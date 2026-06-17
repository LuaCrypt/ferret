pub fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::stable_hash;

    #[test]
    fn hash_is_stable() {
        assert_eq!(stable_hash(b"ferret"), stable_hash(b"ferret"));
        assert_ne!(stable_hash(b"ferret"), stable_hash(b"vm"));
    }
}
