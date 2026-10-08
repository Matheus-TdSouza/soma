use hash_benchmark::size::parse_size;

#[test]
fn parse_size_gigabytes() {
    assert_eq!(parse_size("4G"), Some(4 << 30));
}

#[test]
fn parse_size_megabytes() {
    assert_eq!(parse_size("512m"), Some(512 << 20));
}

#[test]
fn parse_size_kilobytes() {
    assert_eq!(parse_size("256k"), Some(256 << 10));
}

#[test]
fn parse_size_zerobytes() {
    assert_eq!(parse_size("0K"), Some(0));
}

#[test]
fn parse_size_rejects_invalid_unit() {
    assert_eq!(parse_size("4é"), None);
}

#[test]
fn parse_size_rejects_num_only() {
    assert_eq!(parse_size("4096"), None);
}

#[test]
fn parse_size_rejects_letter_only() {
    assert_eq!(parse_size("G"), None);
}

#[test]
fn parse_size_rejects_invalid_format() {
    assert_eq!(parse_size("4GB"), None);
}

#[test]
fn parse_size_rejects_signed_format() {
    assert_eq!(parse_size("-1G"), None);
}

#[test]
fn parse_size_rejects_overflow() {
    assert_eq!(parse_size("20000000000G"), None);
}

#[test]
fn parse_size_rejects_empty() {
    assert_eq!(parse_size(""), None);
}
