//  * DEFLATE Compressed Data Format Specification version 1.3
//    https://tools.ietf.org/html/rfc1951
//  * puff: a simple inflate written to specify the deflate format unambiguously
//    https://github.com/madler/zlib/blob/master/contrib/puff/puff.c
//  * Canonical Huffman code
//    https://en.wikipedia.org/wiki/Canonical_Huffman_code
//  * ZLIB Compressed Data Format Specification version 3.3
//    https://tools.ietf.org/html/rfc1950
//  * An Explanation of the Deflate Algorithm
//    https://www.zlib.net/feldspar.html

const MAX_BITS: usize = 15;
const MAX_L_CODES: usize = 286;
const MAX_D_CODES: usize = 30;
const MAX_CODES: usize = MAX_L_CODES + MAX_D_CODES;
const FIX_L_CODES: usize = 288;

#[derive(Debug)]
pub enum Error {
    HuffmanTableTooBig,
    InvalidBlockCodeHeader,
    InvalidBlockSize,
    InvalidBlockType,
    InvalidDataHeader,
    InvalidDistTooFar,
    InvalidFixedCode,
    MissingEndOfBlockCode,
    OutOfCodes,
    OutOfInput,
    TooManyCodes,
}

// Instead of using a classic Huffman code with a tree datastructure, we will
// be using a more compact one: a canonical Huffman code.
struct HuffmanTable {
    count: [u16; MAX_BITS + 1],
    symbol: [u16; MAX_CODES],
}

impl HuffmanTable {
    // Create the table to decode the canonical Huffman code described by the
    // `length` array
    fn new(length: &[u16]) -> Result<HuffmanTable, Error> {
        let mut table = HuffmanTable {
            count: [0; MAX_BITS + 1],
            symbol: [0; MAX_CODES],
        };

        for len in 0..length.len() {
            table.count[length[len] as usize] += 1;
        }

        // Check if the count is valid (one bit = 2x more codes)
        let mut codes_left = 1;
        for len in 1..(MAX_BITS + 1) {
            codes_left <<= 1;
            codes_left -= table.count[len] as i32;
            if codes_left < 0 {
                return Err(Error::TooManyCodes);
            }
        }

        // Add symbols in sorted order (first by length, then by symbol) by
        // generating an offset table

        let mut offset = [0; MAX_BITS + 1];
        for len in 1..MAX_BITS {
            offset[len + 1] = offset[len] + table.count[len];
        }

        for sym in 0..length.len() {
            let len = length[sym] as usize;
            if len != 0 {
                table.symbol[offset[len] as usize] = sym as u16;
                offset[len as usize] += 1;
            }
        }

        Ok(table)
    }

    fn decode_sym(&self, state: &mut Decoder) -> Result<u16, Error> {
        let mut code = 0;
        let mut first = 0;
        let mut index = 0;
        for bit in 1..(MAX_BITS + 1) {
            code |= state.get_bits(1)?;
            let count = self.count[bit];
            if code < first + count {
                return Ok(self.symbol[(index + (code - first)) as usize]);
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }

        Err(Error::OutOfCodes)
    }
}

pub struct Decoder {
    // We store input data as bytes, but since compressed data blocks are not
    // guaranteed to begin on a byte boundary, we need a buffer to hold unused
    // bits from previous byte.
    input: Vec<u8>,
    input_idx: usize,
    bit_buf: u32,
    bit_cnt: u32,

    pub output: Vec<u8>,
}

impl Decoder {
    pub fn new(input: Vec<u8>) -> Decoder {
        Decoder {
            input: input,
            input_idx: 0,
            bit_buf: 0,
            bit_cnt: 0,
            output: Vec::new(),
        }
    }

    pub fn decompress(&mut self) -> Result<(), Error> {
        // Validate header (CM = 8 CINFO = 7 FCHECK = 1 FDICT = 0 FLEVEL = 0)
        let cmf = self.get_bits(8)?;
        let flg = self.get_bits(8)?;
        if cmf != 0x78 || flg != 1 {
            return Err(Error::InvalidDataHeader);
        }
        loop {
            let end_of_file = self.get_bits(1)?;
            let compress_mode = self.get_bits(2)?;
            match compress_mode {
                0 => self.non_compressed(),
                1 => self.fixed_huffman(),
                2 => self.dynamic_huffman(),
                3 => Err(Error::InvalidBlockType),
                _ => unreachable!(),
            }?;
            if end_of_file == 1 {
                break;
            }
        }

        Ok(())
    }

    fn get_bits(&mut self, need: u32) -> Result<u16, Error> {
        let mut val = self.bit_buf;
        while self.bit_cnt < need {
            if self.input_idx == self.input.len() {
                return Err(Error::OutOfInput);
            }
            // Load a new byte
            let byte = self.input[self.input_idx] as u32;
            self.input_idx += 1;
            val |= byte << self.bit_cnt;
            self.bit_cnt += 8;
        }
        // Keep only unused bits inside the buffer
        self.bit_buf = val >> need;
        self.bit_cnt -= need;
        // Zero out unwanted bits
        Ok((val & ((1 << need) - 1)) as u16)
    }

    // RFC 1951 - Section 3.2.4
    fn non_compressed(&mut self) -> Result<(), Error> {
        // Ignore bits in buffer until next byte boundary (these data blocks
        // are byte-aligned)
        self.bit_buf = 0;
        self.bit_cnt = 0;

        let len = self.get_bits(16)?;
        let nlen = self.get_bits(16)?;
        if !nlen != len {
            return Err(Error::InvalidBlockSize);
        }
        // Non-compressed mode is as simple as reading `len` bytes
        for _ in 0..len {
            let byte = self.get_bits(8)? as u8;
            self.output.push(byte);
        }

        Ok(())
    }

    // RFC 1951 - Section 3.2.5
    fn decompress_block(
        &mut self,
        len_table: &HuffmanTable,
        dist_table: &HuffmanTable,
    ) -> Result<(), Error> {
        const EXTRA_LEN: [u16; 29] = [
            3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99,
            115, 131, 163, 195, 227, 258,
        ];
        const EXTRA_BITS: [u16; 29] = [
            0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0
        ];
        const EXTRA_DIST: [u16; 30] = [
            1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025,
            1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
        ];
        const EXTRA_DBITS: [u16; 30] = [
            0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12,
            12, 13, 13,
        ];

        loop {
            let mut symbol = len_table.decode_sym(self)?;
            if symbol == 256 {
                // End of block
                break;
            } else if symbol < 256 {
                // Literal
                self.output.push(symbol as u8);
            } else if symbol < 290 {
                // Length/distance pair

                // Get length
                symbol -= 257;
                if symbol as usize > EXTRA_LEN.len() {
                    return Err(Error::InvalidFixedCode);
                }
                let len =
                    EXTRA_LEN[symbol as usize] + self.get_bits(EXTRA_BITS[symbol as usize] as u32)?;

                // Get distance
                symbol = dist_table.decode_sym(self)?;
                let dist = EXTRA_DIST[symbol as usize]
                    + self.get_bits(EXTRA_DBITS[symbol as usize] as u32)?;

                // Copy `len` bytes from `dist` bytes back
                let dist = dist as usize;
                if dist > self.output.len() {
                    return Err(Error::InvalidDistTooFar);
                }
                for _ in 0..len {
                    let prev = self.output[self.output.len() - dist];
                    self.output.push(prev);
                }
            }
        }

        Ok(())
    }

    // RFC 1951 - Section 3.2.6
    fn fixed_huffman(&mut self) -> Result<(), Error> {
        let mut length = [0u16; FIX_L_CODES];
        for sym in 0..FIX_L_CODES {
            length[sym] = match sym {
                0...143 => 8,
                144...255 => 9,
                256...279 => 7,
                280...287 => 8,
                _ => unreachable!(),
            };
        }

        let dist = [5u16; MAX_D_CODES];

        let len_table = HuffmanTable::new(&length)?;
        let dist_table = HuffmanTable::new(&dist)?;
        self.decompress_block(&len_table, &dist_table)?;

        Ok(())
    }

    // RFC 1951 - Section 3.2.7
    fn dynamic_huffman(&mut self) -> Result<(), Error> {
        // Lengths of each table
        let nlen: usize = self.get_bits(5)? as usize + 257;
        let ndist: usize = self.get_bits(5)? as usize + 1;
        let ncode: usize = self.get_bits(4)? as usize + 4;
        if nlen > MAX_L_CODES || ndist > MAX_D_CODES {
            return Err(Error::HuffmanTableTooBig);
        }

        // Build temporary table to read literal/length/distance afterwards
        const ORDER: [usize; 19] = [
            16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15
        ];
        let mut length = [0; MAX_CODES];
        for idx in 0..ncode {
            length[ORDER[idx]] = self.get_bits(3)?;
        }
        let len_table = HuffmanTable::new(&length)?;

        // Get literal and length/distance
        let mut idx: usize = 0;
        while idx < nlen + ndist {
            let mut symbol = len_table.decode_sym(self)?;
            if symbol < 16 {
                length[idx] = symbol;
                idx += 1;
            } else {
                let mut len = 0;
                if symbol == 16 {
                    if idx == 0 {
                        return Err(Error::InvalidBlockCodeHeader);
                    }
                    len = length[idx - 1];
                    symbol = 3 + self.get_bits(2)?;
                } else if symbol == 17 {
                    symbol = 3 + self.get_bits(3)?;
                } else {
                    symbol = 11 + self.get_bits(7)?;
                }

                if idx + symbol as usize > nlen + ndist {
                    return Err(Error::InvalidBlockCodeHeader);
                }
                for _ in 0..symbol {
                    length[idx] = len;
                    idx += 1;
                }
            }
        }

        if length[256] == 0 {
            return Err(Error::MissingEndOfBlockCode);
        }

        let len_table = HuffmanTable::new(&length[..nlen])?;
        let dist_table = HuffmanTable::new(&length[nlen..])?;
        self.decompress_block(&len_table, &dist_table)?;

        Ok(())
    }
}
