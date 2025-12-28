use rand::Rng;
use crate::error::Error;

/// Convert a custom Base64-like string to u64.
///
/// This uses a Little-Endian encoding scheme: the first character represents
/// the least significant 6 bits.
///
/// Max supported length is 10 characters (60 bits).
pub fn str_to_int(s: &str) -> Result<u64, Error> {
    if s.len() > 10 {
        // 10 chars * 6 bits = 60 bits, fitting safely in u64.
        return Err(Error::new(500).with_message("String length cannot exceed 10 characters"));
    }

    let mut result: u64 = 0;
    for (i, c) in s.chars().enumerate() {
        let val = char_to_u8(c)?;
        result |= (val as u64) << (i * 6);
    }
    Ok(result)
}

/// Convert u64 to a custom Base64-like string.
///
/// This produces a Little-Endian string (first char is LSB).
pub fn int_to_str(mut n: u64) -> String {
    if n == 0 {
        return String::new(); // Or maybe "A"? Original code returns "" for 0.
    }

    let mut result = String::with_capacity(11);
    while n != 0 {
        let val = (n & 0x3F) as u8;
        // u6_to_char always returns Some for val < 64
        if let Some(c) = u6_to_char(val) {
            result.push(c);
        }
        n >>= 6;
    }
    result
}

fn char_to_u8(c: char) -> Result<u8, Error> {
    match c {
        'A'..='Z' => Ok(c as u8 - b'A'),
        'a'..='z' => Ok(c as u8 - b'a' + 26),
        '0'..='9' => Ok(c as u8 - b'0' + 52),
        '+' => Ok(62),
        '/' => Ok(63),
        _ => Err(Error::new(500).with_message(format!("Unsupported character: {}", c))),
    }
}

fn u6_to_char(n: u8) -> Option<char> {
    match n {
        0..=25 => Some((b'A' + n) as char),
        26..=51 => Some((b'a' + (n - 26)) as char),
        52..=61 => Some((b'0' + (n - 52)) as char),
        62 => Some('+'),
        63 => Some('/'),
        _ => None,
    }
}

/// Generate a random API Key of specified length.
///
/// Uses alphanumeric characters (A-Z, a-z, 0-9).
pub fn generate_api_key(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();

    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    // 引入模块中的所有内容
    use super::*;

    #[test]
    fn test_api_key_length() {
        let key_length = 32;
        let api_key = generate_api_key(key_length);
        println!("Generated API Key: {}", api_key);
        // 验证生成的 API Key 长度是否符合预期
        assert_eq!(
            api_key.len(),
            key_length,
            "生成的 API Key 长度应为 {}",
            key_length
        );
    }

    #[test]
    fn test_api_key_randomness() {
        let key_length = 32;
        let key1 = generate_api_key(key_length);
        let key2 = generate_api_key(key_length);
        // 检查两次生成的 API Key 是否不同（极低概率下可能相同，但几乎可以忽略）
        assert_ne!(key1, key2, "两次生成的 API Key 不应相同");
    }
}
