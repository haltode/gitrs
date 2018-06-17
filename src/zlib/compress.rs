// Careful with the bits order, we want little endian format
// (see RFC 1951 - Section 3.1)
fn convert_u16_to_u8(x: u16) -> [u8; 2] {
    [(x & 0xff) as u8, ((x >> 8) & 0xff) as u8]
}

// Big endian this time
fn convert_u32_to_u8(x: u32) -> [u8; 4] {
    [
        ((x >> 24) & 0xff) as u8,
        ((x >> 16) & 0xff) as u8,
        ((x >> 8) & 0xff) as u8,
        (x & 0xff) as u8,
    ]
}

#[derive(Debug)]
pub enum Error {
    OutOfInput,
}

pub struct Encoder {
    input: Vec<u8>,
    input_idx: usize,

    pub output: Vec<u8>,
}

impl Encoder {
    pub fn new(input: Vec<u8>) -> Encoder {
        Encoder {
            input: input,
            input_idx: 0,
            output: Vec::new(),
        }
    }

    pub fn compress(&mut self) -> Result<(), Error> {
        self.write_header();

        // :D
        let nb_bytes = self.input.len();
        self.non_compressed(nb_bytes)?;

        self.add_checksum();
        return Ok(());
    }

    fn write_header(&mut self) {
        // CM = 8 CINFO = 7 FCHECK = 1 FDICT = 0 FLEVEL = 0
        self.output.push(0x78);
        self.output.push(1);
    }

    fn non_compressed(&mut self, nb_bytes: usize) -> Result<(), Error> {
        // TODO: temporary block header
        self.output.push(1);

        let start = self.input_idx;
        let end = start + nb_bytes;
        if end > self.input.len() {
            return Err(Error::OutOfInput);
        }

        let mut header = Vec::new();
        header.extend_from_slice(&convert_u16_to_u8(nb_bytes as u16));
        header.extend_from_slice(&convert_u16_to_u8(!nb_bytes as u16));

        let data = &self.input[start..end];

        self.output.extend(header);
        self.output.extend(data);

        return Ok(());
    }

    fn fixed_huffman(&mut self) -> Result<(), Error> {
        return Ok(());
    }

    fn dynamic_huffman(&mut self) -> Result<(), Error> {
        return Ok(());
    }

    // Adler-32 checksum
    fn add_checksum(&mut self) {
        let mut a: u32 = 1;
        let mut b: u32 = 0;

        for byte in 0..self.input.len() {
            a = (a + self.input[byte] as u32) % 65521;
            b = (b + a) % 65521;
        }

        let res = (b << 16) | a;
        self.output.extend_from_slice(&convert_u32_to_u8(res));
    }
}
