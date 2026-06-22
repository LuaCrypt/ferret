pub(super) fn level(chars: &[char], cursor: usize) -> Option<usize> {
    if chars.get(cursor) != Some(&'[') {
        return None;
    }
    let mut level = 0;
    while chars.get(cursor + level + 1) == Some(&'=') {
        level += 1;
    }
    (chars.get(cursor + level + 1) == Some(&'[')).then_some(level)
}

pub(super) fn close(chars: &[char], cursor: usize, level: usize) -> bool {
    if chars.get(cursor) != Some(&']') {
        return false;
    }
    for index in 0..level {
        if chars.get(cursor + index + 1) != Some(&'=') {
            return false;
        }
    }
    chars.get(cursor + level + 1) == Some(&']')
}
