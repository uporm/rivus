#![allow(unused)]
use anyhow::anyhow;
use rand::Rng;

pub fn str_to_int(s: &str) -> anyhow::Result<u64> {
    if s.len() > 10 {
        // 修改为10，因为10*6=60，仍在u64范围内
        return Err(anyhow!("字符串长度不能超过10个字符"));
    }

    let mut result: u64 = 0;
    for (i, c) in s.chars().enumerate() {
        if i * 6 >= 64 {
            // 添加额外检查，确保不会溢出
            return Err(anyhow!("位移操作将导致溢出"));
        }
        let val = char_to_u8(c)?;
        result |= (val as u64) << (i * 6);
    }
    Ok(result)
}

pub fn int_to_str(n: u64) -> String {
    let mut result = String::new();
    let mut n = n;
    while n != 0 {
        let val = (n & 0x3F) as u8;
        if let Some(c) = u6_to_char(val) {
            result.push(c);
        }
        n >>= 6;
    }
    result
}

fn char_to_u8(c: char) -> anyhow::Result<u8> {
    match c {
        'A'..='Z' => Ok(c as u8 - b'A'),
        'a'..='z' => Ok(c as u8 - b'a' + 26),
        '0'..='9' => Ok(c as u8 - b'0' + 52),
        '+' => Ok(62),
        '/' => Ok(63),
        _ => Err(anyhow!("不支持的字符")),
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

// 生成 API Key
fn generate_api_key(length: usize) -> String {
    // 定义 API Key 可用的字符集
    let charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
    let mut rng = rand::rng();

    // 随机选取字符生成 API Key
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..charset.len());
            charset[idx]
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
