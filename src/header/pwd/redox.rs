pub fn split(line: &[u8]) -> [&[u8]; 7] {
    let mut parts: [&[u8]; 7] = [&[]; 7];
    let mut iter = line.split(|b| *b == b';');

    parts[0] = iter.next().unwrap_or(&[]);
    // Skip passwd
    for i in 2..7 {
        parts[i] = iter.next().unwrap_or(&[]);
    }
    parts
}
