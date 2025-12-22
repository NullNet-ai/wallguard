/// Masks a string by keeping the first `visible_front` and last `visible_back`
/// characters visible, replacing the rest with `*`.
pub fn mask_string(s: &str, visible_front: usize, visible_back: usize) -> String {
    let len = s.len();

    if visible_front + visible_back >= len {
        return s.to_string();
    }

    let front = &s[..visible_front];
    let back = &s[len - visible_back..];

    let mask_len = len - visible_front - visible_back;
    let mask = "*".repeat(mask_len);

    format!("{front}{mask}{back}")
}
