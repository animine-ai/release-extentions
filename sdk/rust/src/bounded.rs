use alloc::vec::Vec;
use crate::Validate;
use crate::wire::{Wire,parse,serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error { InvalidInput, SizeLimit, Json }
pub fn ensure(ok: bool) -> Result<(), Error> { if ok { Ok(()) } else { Err(Error::InvalidInput) } }
pub fn text(value: &str, max: usize) -> Result<(), Error> { ensure(value.len() <= max) }
pub fn id(value: &str) -> Result<(), Error> {
    ensure(!value.is_empty() && value.len() <= 128 && value.as_bytes()[0].is_ascii_alphanumeric() &&
        value.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b)))
}
#[cfg(test)]
fn depth(bytes: &[u8]) -> Result<(), Error> {
    let (mut level, mut quoted, mut escaped) = (0u32, false, false);
    for b in bytes {
        if quoted {
            if escaped { escaped=false; } else if *b==b'\\' { escaped=true; } else if *b==b'"' { quoted=false; }
        } else if *b==b'"' { quoted=true; }
        else if *b==b'{' || *b==b'[' { level+=1; ensure(level<=16)?; }
        else if *b==b'}' || *b==b']' { ensure(level>0)?; level-=1; }
    }
    ensure(level==0 && !quoted)
}
pub fn decode<T: Wire + Validate>(bytes: &[u8], cap: usize) -> Result<T, Error> {
    // wire::Parser enforces the same 16-container depth while parsing, including
    // empty containers. Avoid a redundant full-body scan before UTF-8/JSON validation.
    ensure(!bytes.is_empty() && bytes.len()<=cap)?;
    let value=T::from_value(parse(bytes)?)?;
    value.validate()?; Ok(value)
}
pub fn encode<T: Wire + Validate>(value: &T, cap: usize) -> Result<Vec<u8>, Error> {
    value.validate()?;
    let bytes=serialize(&value.to_value());
    ensure(bytes.len()<=cap)?; Ok(bytes)
}
#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::RequestSpec;
    #[test] fn strict_nullable_and_duplicates() {
        let good=br#"{"requestId":"r","sourceRole":"CALENDAR","url":"https://example.org","method":"GET","targetToken":null}"#;
        assert!(decode::<RequestSpec>(good,1024).is_ok());
        assert!(decode::<RequestSpec>(br#"{"requestId":"r","sourceRole":"CALENDAR","url":"u","method":"GET"}"#,1024).is_err());
        assert!(decode::<RequestSpec>(br#"{"requestId":"r","requestId":"b","sourceRole":"CALENDAR","url":"u","method":"GET","targetToken":null}"#,1024).is_err());
    }
    #[test] fn unicode_byte_and_depth_bounds() { assert!(text("😀",3).is_err()); assert!(depth(b"[[[[[[[[[[[[[[[[[0]]]]]]]]]]]]]]]]]").is_err()); }
}
