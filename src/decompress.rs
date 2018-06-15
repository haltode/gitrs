const MAX_BITS: usize = 15;
const MAX_L_CODES: usize = 286;
const MAX_D_CODES: usize = 30;
const MAX_CODES: usize = MAX_L_CODES + MAX_D_CODES;
const FIX_L_CODES: usize = 288;

// TODO: properly handle errors (custom types, ? operator, etc.)
// remove all temporary panic!()
enum Error {

}

struct HuffmanTable {
    count: [u16; MAX_BITS + 1],
    symbol: [u16; MAX_CODES],
}

impl HuffmanTable {
    fn new(length: &[u16]) -> HuffmanTable {
        let mut table = HuffmanTable {
            count: [0; MAX_BITS + 1],
            symbol: [0; MAX_CODES],
        };

        for len in 0..length.len() {
            table.count[length[len] as usize] += 1;
        }

        let mut left = 1;
        for len in 1..(MAX_BITS + 1) {
            left <<= 1;
            left -= table.count[len] as i32;
            if left < 0 {
                panic!()
            }
        }

        let mut offset = [0; MAX_BITS + 1];
        for len in 1..MAX_BITS {
            offset[len + 1] = offset[len] + table.count[len];
        }

        for symbol in 0..length.len() {
            if length[symbol] != 0 {
                let len = length[symbol];
                table.symbol[offset[len as usize] as usize] = symbol as u16;
                offset[len as usize] += 1;
            }
        }

        return table;
    }

    fn decode_char(&self, state: &mut State) -> u16 {
        let mut code = 0;
        let mut first = 0;
        let mut index = 0;
        for len in 1..(MAX_BITS + 1) {
            code |= state.get_bits(1);
            let count = self.count[len];
            if code < first + count {
                return self.symbol[(index + (code - first)) as usize];
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        panic!()
    }
}

struct State {
    input: Vec<u8>,
    input_idx: usize,
    bit_buf: u32,
    bit_cnt: u32,

    output: Vec<u8>,
}

impl State {
    fn new(input: Vec<u8>) -> State {
        State {
            input: input,
            input_idx: 0,
            bit_buf: 0,
            bit_cnt: 0,
            output: Vec::new(),
        }
    }

    fn get_bits(&mut self, need: u32) -> u16 {
        let mut val = self.bit_buf;
        while self.bit_cnt < need {
            let byte = self.input[self.input_idx] as u32;
            self.input_idx += 1;
            val |= byte << self.bit_cnt;
            self.bit_cnt += 8;
        }
        self.bit_buf = val >> need;
        self.bit_cnt -= need;
        return (val & ((1 << need) - 1)) as u16;
    }

    fn decompress(&mut self) {
        // TODO: check header
        self.get_bits(8);
        self.get_bits(8);
        loop {
            let end_of_file = self.get_bits(1);
            let compress_mode = self.get_bits(2);
            match compress_mode {
                0 => self.non_compressed(),
                1 => self.fixed_huffman(),
                2 => self.dynamic_huffman(),
                3 => panic!(),
                _ => unreachable!(),
            };
            if end_of_file == 1 {
                break;
            }
        }
    }

    fn non_compressed(&mut self) {
        self.bit_buf = 0;
        self.bit_cnt = 0;
        let len = self.get_bits(16);
        let nlen = self.get_bits(16);
        if !nlen != len {
            panic!();
        }
        for _ in 0..len {
            let byte = self.get_bits(8) as u8;
            self.output.push(byte);
        }
    }

    fn decompress_block(&mut self, len_table: &HuffmanTable, dist_table: &HuffmanTable) {
        const LENS: [u16; 29] = [
            3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99,
            115, 131, 163, 195, 227, 258,
        ];
        const LEXT: [u16; 29] = [
            0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0
        ];
        const DISTS: [u16; 30] = [
            1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025,
            1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
        ];
        const DEXT: [u16; 30] = [
            0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12,
            12, 13, 13,
        ];

        loop {
            let mut symbol = len_table.decode_char(self);
            if symbol < 256 {
                self.output.push(symbol as u8);
            } else if symbol == 256 {
                break;
            } else if symbol < 290 {
                symbol -= 257;
                if symbol as usize > LENS.len() {
                    panic!()
                }
                let mut len = LENS[symbol as usize] + self.get_bits(LEXT[symbol as usize] as u32);

                symbol = dist_table.decode_char(self);
                let dist = DISTS[symbol as usize] + self.get_bits(DEXT[symbol as usize] as u32);
                while len > 0 {
                    let prev = self.output[self.output.len() - dist as usize];
                    self.output.push(prev);
                    len -= 1;
                }
            } else {
                panic!();
            }
        }
    }

    fn fixed_huffman(&mut self) {
        // TODO: find proper way to initialize things just once (not at every call)
        let mut length = [0u16; FIX_L_CODES];
        for sym in 0..FIX_L_CODES {
            length[sym] = match sym {
                0...143 => 8,
                144...255 => 9,
                256...279 => 7,
                280...287 => 8, // TODO: use constant in ..
                _ => unreachable!()
            };
        }

        let dist = [5u16; MAX_D_CODES];

        let len_table = HuffmanTable::new(&length);
        let dist_table = HuffmanTable::new(&dist);
        // TODO: check tables content

        self.decompress_block(&len_table, &dist_table);
    }

    fn dynamic_huffman(&mut self) {
        let nlen: usize = self.get_bits(5) as usize + 257;
        let ndist: usize = self.get_bits(5) as usize + 1;
        let ncode: usize = self.get_bits(4) as usize + 4;
        if nlen > MAX_L_CODES || ndist > MAX_D_CODES {
            panic!();
        }

        // TODO: static?
        const ORDER: [usize; 19] = [
            16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15
        ];
        let mut length = [0; MAX_CODES];
        for idx in 0..ncode {
            length[ORDER[idx]] = self.get_bits(3);
        }

        let len_table = HuffmanTable::new(&length);

        let mut idx: usize = 0;
        while idx < nlen + ndist {
            let mut symbol = len_table.decode_char(self);
            if symbol < 16 {
                length[idx] = symbol;
                idx += 1;
            } else {
                let mut len = 0;
                // TODO: use match?
                if symbol == 16 {
                    if idx == 0 {
                        panic!();
                    }
                    len = length[idx - 1];
                    symbol = 3 + self.get_bits(2);
                } else if symbol == 17 {
                    symbol = 3 + self.get_bits(3);
                } else {
                    symbol = 11 + self.get_bits(7);
                }

                if idx + symbol as usize > nlen + ndist {
                    panic!();
                }
                while symbol > 0 {
                    length[idx] = len;
                    idx += 1;
                    symbol -= 1;
                }
            }
        }

        if length[256] == 0 {
            panic!();
        }

        let len_table = HuffmanTable::new(&length[..nlen]);
        let dist_table = HuffmanTable::new(&length[nlen..]);

        self.decompress_block(&len_table, &dist_table);
    }
}

pub fn decompress(input: Vec<u8>) -> Vec<u8> {
    let mut state = State::new(input);
    state.decompress();
    return state.output;
}
