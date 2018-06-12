const BLOCK_SIZE: usize = 512 / 8;
const ROUND_SIZE: usize = 80;

fn convert_u64_to_u8(x: u64) -> [u8; 8] {
    [
        ((x >> 56) & 0xff) as u8,
        ((x >> 48) & 0xff) as u8,
        ((x >> 40) & 0xff) as u8,
        ((x >> 32) & 0xff) as u8,
        ((x >> 24) & 0xff) as u8,
        ((x >> 16) & 0xff) as u8,
        ((x >> 8) & 0xff) as u8,
        (x & 0xff) as u8,
    ]
}

fn convert_u8_to_u32(b1: u8, b2: u8, b3: u8, b4: u8) -> u32 {
    (b1 as u32) << 24 | (b2 as u32) << 16 | (b3 as u32) << 8 | (b4 as u32)
}

// Format: input (as bytes) + padding + 64-bit message length (in bits)
fn sha1_format(input: &str) -> Vec<u8> {
    let mut fmt_input: Vec<u8> = Vec::new();
    let input_size: usize = input.len();

    fmt_input.extend(input.as_bytes());
    fmt_input.push(0x80);
    fmt_input.extend(vec![0; 63 - ((input_size + 8) % 64)]);
    fmt_input.extend_from_slice(&convert_u64_to_u8(8 * (input_size as u64)));

    fmt_input
}

pub fn sha1(input: &str) -> String {
    // Initial states
    let mut states = [
        0x67452301u32,
        0xefcdab89u32,
        0x98badcfeu32,
        0x10325476u32,
        0xc3d2e1f0u32,
    ];
    let mut w = [0u32; ROUND_SIZE];

    for block in sha1_format(input).chunks(BLOCK_SIZE) {
        for i in 0..16 {
            w[i] = convert_u8_to_u32(
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            );
        }
        for i in 16..80 {
            w[i] = w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16];
            w[i] = w[i].rotate_left(1);
        }

        let mut a = states[0];
        let mut b = states[1];
        let mut c = states[2];
        let mut d = states[3];
        let mut e = states[4];

        for i in 0..80 {
            let (k, f) = match i {
                0...19 => (0x5a827999, (b & c) | (!b & d)),
                20...39 => (0x6ed9eba1, (b ^ c ^ d)),
                40...59 => (0x8f1bbcdc, ((b & c) | (b & d) | c & d)),
                60...79 => (0xca62c1d6, (b ^ c ^ d)),
                _ => unreachable!(),
            };

            let tmp = a.rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = tmp;
        }

        states[0] = states[0].wrapping_add(a);
        states[1] = states[1].wrapping_add(b);
        states[2] = states[2].wrapping_add(c);
        states[3] = states[3].wrapping_add(d);
        states[4] = states[4].wrapping_add(e);
    }

    // Final hash formated as a hex string
    states.iter().map(|b| format!("{:02x}", b)).collect()
}
