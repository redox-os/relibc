pub fn split(line: &[u8]) -> [&[u8]; 7] {
    let mut parts: [&[u8]; 7] = [&[]; 7];
    for (i, part) in line.splitn(7, |b| *b == b':').enumerate() {
        parts[i] = part;
    }
    parts
}
