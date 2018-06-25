use bits::big_endian;

fn format_input(input: &str) -> Vec<u8> {
    let mut fmt_input = Vec::new();
    let input_size = input.len();

    fmt_input.extend(input.as_bytes());
    fmt_input.push(0x80);

    let padding = vec![0; 63 - ((input_size + 8) % 64)];
    fmt_input.extend(padding);

    let input_size_bits = 8 * input_size as u64;
    fmt_input.extend_from_slice(&big_endian::u64_to_u8(input_size_bits));

    fmt_input
}

pub fn sha1(data: &str) -> String {
    let mut states = [
        0x67452301u32,
        0xefcdab89u32,
        0x98badcfeu32,
        0x10325476u32,
        0xc3d2e1f0u32,
    ];
    let mut w = [0u32; 80];

    for block in format_input(data).chunks(64) {
        for i in 0..16 {
            w[i] = big_endian::u8_to_u32([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
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
                20...39 => (0x6ed9eba1, b ^ c ^ d),
                40...59 => (0x8f1bbcdc, (b & c) | (b & d) | (c & d)),
                60...79 => (0xca62c1d6, b ^ c ^ d),
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

    u32_hash_to_hex_str(&states)
}

pub fn u32_hash_to_hex_str(hash: &[u32; 5]) -> String {
    hash.iter().map(|b| format!("{:08x}", b)).collect()
}

pub fn u8_slice_hash_to_hex_str(hash: &[u8]) -> String {
    let mut states = [0u32; 5];
    let mut idx = 0;
    for s in 0..5 {
        states[s] = big_endian::u8_slice_to_u32(&hash[idx..]);
        idx += 4;
    }
    u32_hash_to_hex_str(&states)
}

#[cfg(test)]
mod tests {
    use sha1::sha1;

    #[test]
    fn test_sha1() {
        assert_eq!(sha1("abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(sha1(""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(
            sha1("The quick brown fox jumps over the lazy dog"),
            "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12"
        );
        assert_eq!(
            sha1("The quick brown fox jumps over the lazy cog"),
            "de9f2c7fd25e1b3afad3e85a0bd17d9b100db4b3"
        );

        assert_eq!(
            sha1(
                "A4j1pcn9Z8l0jzETQk9hVJjWE5dki7hd4Tk69B2aG60OGdifYMm1BNJ2PnDXz0\
                 D5XwT7QzFZ9JLtKaxl0cMndNPbzStb3YRb4lnR94BAlapbQsRqoZBYyctywtx0\
                 rkOYPbXboNusdd7PupOR3u1Mu71qNuMTgGO3xbO3YAhG4V8eyGGEBQxlObi0m6\
                 jSZ3lNPghdsPDhsuIKfHCUpoSGK0YscCpk6T8zuVadR4KC4vXOfERglIh5ya3r\
                 IxKNCXiborW7tLwhCQlqDmKvVG9fyK1fbwxif0R0h8pJQYo64FvF2Ev5EaPBZy\
                 p9gfPfW1rvgyiKYHFaVes3cs7HDLku64JmaYk79mSpv8XrQoECOmkXhMjIL8U1\
                 gCgpl7ruzkNICaKOc0FoQq5sSyCvH45Cm2qyIrviWVamf4aQ1nE3r7oM2LEpAw\
                 l9d46b6x0XvA3lsdw5lHSpJ9nK0xCD95MXkgFJwT2RaNDxYJesQzHJJJcDz1u6\
                 41znwx4K5onTObfaxMLfZe2LHtnvS9uhqD3gbRRVG9DefFxKnr3ZzJfXhVpsJu\
                 qiogP96YEUXM6Le1UdUQqghG3fJiNmK4K0Llp1ocMHn7RzCPZpyQJydXMZxTsi\
                 Rg9lk1nyTaTeYJVsw375YpMRuV45ZZxMk7RvGEyFhJHYcMEqkzSTh1KVqeUywS\
                 RxQBFp4vB3aWAEU2dejEXIbmLrT4dAqcuSs7T9SsgXVzAmVNSmyCB4vtFaRh6o\
                 OGseV0gqTNzUNcwTPDCQETlkuq0s3VD9j8m4IQymJ4T8EPgF5oAUgWviOiNwr7\
                 JT0GGsNpCa5o1qZy2AbiL8NXRxExhj9aQ5x647O3w2QnylDtbYjCHQpM14obeF\
                 OwThnKbKOHfMUuNoOuFYIcadRVjD0tJwTGOiwb4aDH70aFd6eN4Fnu0wHG62UN\
                 EtEihhkQZfhohShVWcUO23LuLZj4aBIgY5hGJPZO7IImEYtb49rrZ1687EcvTA\
                 LyhMMxMD"
            ),
            "32f3f879a843b8792f1574110d02d66aa04701ad"
        );

        assert_eq!(
            sha1(
                "This is a multi-line string litteral
used as a test file sample!\n"
            ),
            "6bf58217d47b728b777fa2ea1545787587186fff"
        );
    }
}
