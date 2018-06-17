mod compress;
mod decompress;

pub fn compress(input: Vec<u8>) -> Vec<u8> {
    let mut state = compress::Encoder::new(input);
    if let Err(why) = state.compress() {
        panic!("Error while compressing: {:?}", why);
    }
    return state.output;
}

pub fn decompress(input: Vec<u8>) -> Vec<u8> {
    let mut state = decompress::Decoder::new(input);
    if let Err(why) = state.decompress() {
        panic!("Error while decompressing: {:?}", why);
    }
    return state.output;
}
