const MASK31: u32 = 0x7fff_ffff;
const WORD_MUL: u32 = 1_103_515_245;
const WORD_ADD: u32 = 12_345;

pub fn encode_words(words: &[u32], seed: u64) -> Vec<u32> {
    let mut state = (seed as u32 ^ 0x4259_5445) & MASK31;
    words
        .iter()
        .enumerate()
        .map(|(index, word)| {
            state = word_mask(state, index + 1);
            (word ^ state) & MASK31
        })
        .collect()
}

pub fn decode_words(words: &[u32], seed: u64) -> Vec<u32> {
    encode_words(words, seed)
}

pub fn encode_bytes(bytes: &[u8], seed: u64) -> Vec<u8> {
    let mut state = (seed as u32 ^ 0x5354_5247) & 0xff;
    bytes
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            state = byte_mask(state, index + 1);
            let mask = state as u8;
            byte ^ mask
        })
        .collect()
}

fn word_mask(state: u32, index: usize) -> u32 {
    state
        .wrapping_mul(WORD_MUL)
        .wrapping_add(WORD_ADD)
        .wrapping_add((index as u32).wrapping_mul(97))
        & MASK31
}

fn byte_mask(state: u32, index: usize) -> u32 {
    state
        .wrapping_mul(73)
        .wrapping_add(41)
        .wrapping_add((index as u32).wrapping_mul(17))
        & 0xff
}

#[cfg(test)]
mod tests {
    use super::{decode_words, encode_bytes, encode_words};

    #[test]
    fn words_round_trip() {
        let raw = vec![1, 2, 9, 12_345];
        let enc = encode_words(&raw, 4);
        assert_ne!(raw, enc);
        assert_eq!(raw, decode_words(&enc, 4));
    }

    #[test]
    fn bytes_round_trip() {
        let raw = b"secret";
        let enc = encode_bytes(raw, 9);
        assert_ne!(raw, enc.as_slice());
        assert_eq!(raw.to_vec(), encode_bytes(&enc, 9));
    }
}
