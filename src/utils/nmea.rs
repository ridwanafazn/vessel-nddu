//! Utilitas NMEA 0183: checksum, pembentukan sentence, dan validasi.
//!
//! Format wajib: `$TTSSS,field,...*HH\r\n`
//! - `TT`  = talker ID (2 karakter)
//! - `SSS` = sentence formatter (3 karakter)
//! - `HH`  = checksum XOR (2 digit hex)

/// XOR semua byte pada body (tanpa `$` dan tanpa `*HH`).
pub fn checksum(body: &str) -> u8 {
    body.bytes().fold(0u8, |acc, b| acc ^ b)
}

/// Bentuk sentence lengkap dengan checksum dan CRLF.
pub fn build(body: &str) -> String {
    format!("${}*{:02X}\r\n", body, checksum(body))
}

/// Validasi sentence NMEA 0183.
///
/// Memeriksa: awalan `$`, ada `*HH`, panjang ID minimal 5 karakter,
/// ID hanya huruf besar/angka, dan checksum cocok.
pub fn validate(sentence: &str) -> bool {
    let s = sentence.trim_end_matches(['\r', '\n']);
    if !s.starts_with('$') {
        return false;
    }

    let star = match s.rfind('*') {
        Some(i) => i,
        None => return false,
    };

    // Harus tepat 2 digit hex setelah '*'.
    if star + 3 != s.len() {
        return false;
    }

    let body = &s[1..star];
    let expected = match u8::from_str_radix(&s[star + 1..], 16) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let id = match body.split(',').next() {
        Some(id) => id,
        None => return false,
    };

    if id.len() < 5 {
        return false;
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return false;
    }

    checksum(body) == expected
}

/// Ambil talker ID (2 karakter pertama) dari sentence.
pub fn talker(sentence: &str) -> Option<&str> {
    let s = sentence.strip_prefix('$')?;
    let id = s.split(',').next()?;
    if id.len() < 5 {
        return None;
    }
    Some(&id[..2])
}

/// Ambil formatter (3 karakter setelah talker).
pub fn formatter(sentence: &str) -> Option<&str> {
    let s = sentence.strip_prefix('$')?;
    let id = s.split(',').next()?;
    if id.len() < 5 {
        return None;
    }
    Some(&id[2..5])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_known_value() {
        // Body GPRMC contoh; checksum harus 0x26.
        let body = "GPRMC,122334.35,A,0611.9924,S,10648.0077,E,12.5,45.0,311026,0.6,E,A";
        assert_eq!(checksum(body), 0x26);
    }

    #[test]
    fn build_and_validate_roundtrip() {
        let s = build("IIVHW,0.0,T,0.0,M,15.9,N,29.5,K");
        assert!(validate(&s));
        assert_eq!(talker(&s), Some("II"));
        assert_eq!(formatter(&s), Some("VHW"));
    }

    #[test]
    fn rejects_short_id() {
        let s = build("VHW,0.0,T,0.0,M,15.9,N,29.5,K");
        assert!(!validate(&s));
    }

    #[test]
    fn rejects_bad_checksum() {
        assert!(!validate("$IIVHW,0.0,T,0.0,M,15.9,N,29.5,K*00\r\n"));
    }

    #[test]
    fn rejects_missing_crlf_ok_but_bad_star() {
        assert!(!validate("$IIVHW,0.0,T,0.0,M,15.9,N,29.5,K"));
    }
}
