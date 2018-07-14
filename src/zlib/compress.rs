use utils::bits::{big_endian, little_endian};

pub struct Encoder {
    input: Vec<u8>,
    input_idx: usize,

    pub output: Vec<u8>,
}

#[derive(Debug)]
pub enum Error {
    OutOfInput,
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

        self.add_adler32_checksum();
        Ok(())
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
        header.extend_from_slice(&little_endian::u16_to_u8(nb_bytes as u16));
        header.extend_from_slice(&little_endian::u16_to_u8(!nb_bytes as u16));

        let data = &self.input[start..end];

        self.output.extend(header);
        self.output.extend(data);

        Ok(())
    }

    #[allow(dead_code)]
    fn fixed_huffman(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    #[allow(dead_code)]
    fn dynamic_huffman(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    fn add_adler32_checksum(&mut self) {
        let mut a: u32 = 1;
        let mut b: u32 = 0;

        for byte in 0..self.input.len() {
            a = (a + self.input[byte] as u32) % 65521;
            b = (b + a) % 65521;
        }

        let res = (b << 16) | a;
        self.output.extend_from_slice(&big_endian::u32_to_u8(res));
    }
}
