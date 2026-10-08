pub fn parse_size(s: &str) -> Option<usize> {
    let i = s.len().checked_sub(1)?;
    let (number, unit) = s.split_at_checked(i)?;
    let number = number.parse::<usize>().ok()?;
    let mult = match unit {
        "K" | "k" => 1 << 10,
        "M" | "m" => 1 << 20,
        "G" | "g" => 1 << 30,
        _ => return None,
    };
    number.checked_mul(mult)
}
