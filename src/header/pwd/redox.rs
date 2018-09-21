pub fn split(line: &[u8]) -> [&[u8]; 7] {
    let mut parts: [&[u8]; 7] = [&[]; 7];
    let mut iter = line.split(|b| *b == b';');

    parts[0] = iter.next().unwrap_or(&[]);
    // Skip passwd
    for i in 0..2 {
        parts[2 + i] = iter.next().unwrap_or(&[]);
    }
    // Skip gecos
    for i in 0..2 {
        parts[5 + i] = iter.next().unwrap_or(&[]);
    }
    parts
}
