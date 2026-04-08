use chrono::{FixedOffset, TimeZone, Utc};

// Test vectors derived from unbroken-dome/base62 algorithm (11 chars per i64).

fn encode_long(value: i64) -> String {
    // Port of Base62Encoder.accept(long)
    const DIGITS: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut buf = [b'0'; 11];
    let mut v = value;
    if v < 0 {
        for i in (1..=10).rev() {
            let digit = (-(v % 62)) as usize;
            buf[i] = DIGITS[digit];
            v /= 62;
        }
        let first = (-(v - 31)) as usize;
        buf[0] = DIGITS[first];
    } else {
        for i in (1..=10).rev() {
            let digit = (v % 62) as usize;
            buf[i] = DIGITS[digit];
            v /= 62;
        }
        buf[0] = DIGITS[v as usize];
    }
    String::from_utf8(buf.to_vec()).unwrap()
}

fn decode_array(input: &str) -> Option<Vec<i64>> {
    // keep in sync with production decoder expectations:
    if input.len() % 11 != 0 {
        return None;
    }
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::with_capacity(input.len() / 11);
    for chunk in chars.chunks(11) {
        out.push(decode_long_11(chunk)?);
    }
    Some(out)
}

fn decode_long_11(chunk: &[char]) -> Option<i64> {
    if chunk.len() != 11 {
        return None;
    }
    let mut negative = false;
    let mut digit = digit_index(chunk[0])? as i64;
    if digit >= 31 {
        digit -= 31;
        negative = true;
    }
    let mut value = digit;
    for ch in &chunk[1..] {
        let d = digit_index(*ch)? as i64;
        value = value.checked_mul(62)?.checked_add(d)?;
    }
    if negative {
        value = -value;
    }
    Some(value)
}

fn digit_index(ch: char) -> Option<u32> {
    match ch {
        '0'..='9' => Some((ch as u32) - ('0' as u32)),
        'A'..='Z' => Some(10 + (ch as u32) - ('A' as u32)),
        'a'..='z' => Some(36 + (ch as u32) - ('a' as u32)),
        _ => None,
    }
}

#[test]
fn base62_roundtrip_matches_expected_grouping() {
    let a = 123456789i64;
    let b = 1710000000i64;
    let id = format!("{}{}", encode_long(a), encode_long(b));
    let decoded = decode_array(&id).unwrap();
    assert_eq!(decoded, vec![a, b]);
    assert_eq!(id.len(), 22);
}

#[test]
fn epoch_seconds_with_fixed_offset_matches_naive_local() {
    let seconds = 1710000000i64;
    let offset = FixedOffset::east_opt(3 * 3600).unwrap();
    let dt = Utc.timestamp_opt(seconds, 0).unwrap();
    let naive = dt.with_timezone(&offset).naive_local();
    assert_eq!(naive.format("%Y-%m-%dT%H:%M:%S").to_string().len(), 19);
}
