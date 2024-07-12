use anyhow::{bail, Result};

pub fn unhex(s: &str) -> Result<Vec<u8>> {
    if s.len() % 2 != 0 {
        bail!("invalid hex string length {}", s.len());
    }

    let mut bytes = Vec::with_capacity(s.len() / 2);

    for i in 0..s.len() / 2 {
        let first_byte = u8::from_str_radix(&s[i * 2..i * 2 + 1], 16)?;
        let second_byte = u8::from_str_radix(&s[i * 2 + 1..i * 2 + 2], 16)?;

        bytes.push(first_byte << 4 | second_byte);
    }

    Ok(bytes)
}
