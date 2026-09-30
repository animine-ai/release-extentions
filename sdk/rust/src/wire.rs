//! Deterministic integer-only ABI JSON. No dynamic formatters or function tables.
use alloc::{string::String, vec::Vec};
use crate::bounded::{Error,ensure};
#[derive(Clone)]
pub enum Value { Null, Bool(bool), Int(i32), Text(String), Array(Vec<Value>), Object(Vec<(String,Value)>) }
pub trait Wire:Sized { fn from_value(value:Value)->Result<Self,Error>; fn to_value(&self)->Value; }
#[inline(always)]
fn has_zero(v:u64)->bool{v.wrapping_sub(0x0101010101010101)&!v&0x8080808080808080!=0}
// UTF-8 is validated at parse entry. Word scans only locate ASCII JSON boundaries.
#[inline(always)]
fn text_span(bytes:&[u8],mut pos:usize)->usize{
    while bytes.len()-pos>=8{
        let b=&bytes[pos..pos+8];let word=u64::from_le_bytes([b[0],b[1],b[2],b[3],b[4],b[5],b[6],b[7]]);
        if has_zero(word^0x2222222222222222)||has_zero(word^0x5c5c5c5c5c5c5c5c)||has_zero(word&0xe0e0e0e0e0e0e0e0){break;}
        pos+=8;
    }
    while pos<bytes.len()&&bytes[pos]>=32&&bytes[pos]!=b'"'&&bytes[pos]!=b'\\'{pos+=1;}pos
}
impl Wire for String {
    fn from_value(v:Value)->Result<Self,Error>{if let Value::Text(s)=v{Ok(s)}else{Err(Error::Json)}}
    fn to_value(&self)->Value{Value::Text(self.clone())}
}
impl Wire for i32 {
    fn from_value(v:Value)->Result<Self,Error>{if let Value::Int(n)=v{Ok(n)}else{Err(Error::Json)}}
    fn to_value(&self)->Value{Value::Int(*self)}
}
impl Wire for bool {
    fn from_value(v:Value)->Result<Self,Error>{if let Value::Bool(b)=v{Ok(b)}else{Err(Error::Json)}}
    fn to_value(&self)->Value{Value::Bool(*self)}
}
impl<T:Wire> Wire for Option<T> {
    fn from_value(v:Value)->Result<Self,Error>{if let Value::Null=v{Ok(None)}else{Ok(Some(T::from_value(v)?))}}
    fn to_value(&self)->Value{match self{Some(v)=>v.to_value(),None=>Value::Null}}
}
impl<T:Wire> Wire for Vec<T> {
    fn from_value(v:Value)->Result<Self,Error>{if let Value::Array(a)=v{a.into_iter().map(T::from_value).collect()}else{Err(Error::Json)}}
    fn to_value(&self)->Value{Value::Array(self.iter().map(Wire::to_value).collect())}
}
pub struct Fields(Vec<(String,Value)>);
impl Fields {
    pub fn new(v:Value,names:&[&str])->Result<Self,Error>{
        if let Value::Object(fields)=v{
            ensure(fields.len()==names.len() && fields.iter().all(|(n,_)|names.contains(&n.as_str())))?;
            Ok(Self(fields))
        }else{Err(Error::Json)}
    }
    pub fn take<T:Wire>(&mut self,name:&str)->Result<T,Error>{
        let i=self.0.iter().position(|(n,_)|n==name).ok_or(Error::Json)?;
        let last=self.0.len()-1;
        self.0.swap(i,last);
        T::from_value(self.0.pop().ok_or(Error::Json)?.1)
    }
}
struct Parser<'a>{text:&'a str,pos:usize}
impl<'a> Parser<'a>{
    fn peek(&self)->Option<u8>{self.text.as_bytes().get(self.pos).copied()}
    fn ws(&mut self){while self.peek().is_some_and(|b|b" \t\r\n".contains(&b)){self.pos+=1;}}
    fn byte(&mut self,b:u8)->Result<(),Error>{ensure(self.peek()==Some(b))?;self.pos+=1;Ok(())}
    fn literal(&mut self,token:&str)->Result<(),Error>{ensure(self.text.get(self.pos..).ok_or(Error::Json)?.starts_with(token))?;self.pos+=token.len();Ok(())}
    fn hex4(&mut self)->Result<u32,Error>{
        let mut n=0;for _ in 0..4{let b=self.peek().ok_or(Error::Json)?;self.pos+=1;let d=match b{b'0'..=b'9'=>b-b'0',b'a'..=b'f'=>b-b'a'+10,b'A'..=b'F'=>b-b'A'+10,_=>return Err(Error::Json)};n=n*16+d as u32;}Ok(n)
    }
    fn text(&mut self)->Result<String,Error>{
        self.byte(b'"')?;let mut s=String::new();
        loop{let b=self.peek().ok_or(Error::Json)?;
            if b==b'"'{self.pos+=1;return Ok(s);}
            if b==b'\\'{self.pos+=1;let escaped=self.peek().ok_or(Error::Json)?;self.pos+=1;
                match escaped{b'"'=>s.push('"'),b'\\'=>s.push('\\'),b'/'=>s.push('/'),b'b'=>s.push('\x08'),b'f'=>s.push('\x0c'),b'n'=>s.push('\n'),b'r'=>s.push('\r'),b't'=>s.push('\t'),b'u'=>{
                    let mut n=self.hex4()?;
                    if (0xd800..=0xdbff).contains(&n){self.byte(b'\\')?;self.byte(b'u')?;let low=self.hex4()?;ensure((0xdc00..=0xdfff).contains(&low))?;n=0x10000+((n-0xd800)<<10)+(low-0xdc00);}
                    else{ensure(!(0xdc00..=0xdfff).contains(&n))?;}
                    s.push(char::from_u32(n).ok_or(Error::Json)?);
                },_=>return Err(Error::Json)}
            }else{
                // The complete input is already UTF-8 validated. Copy a whole unescaped
                // span once instead of decoding and growing the output per character.
                // JSON quotes/backslashes are ASCII boundaries, so this preserves UTF-8.
                ensure(b>=32)?;let start=self.pos;self.pos=text_span(self.text.as_bytes(),self.pos);
                s.push_str(self.text.get(start..self.pos).ok_or(Error::Json)?);
            }
        }
    }
    fn value(&mut self,depth:u32)->Result<Value,Error>{
        ensure(depth<=16)?;self.ws();match self.peek().ok_or(Error::Json)?{
            b'"'=>Ok(Value::Text(self.text()?)),b'n'=>{self.literal("null")?;Ok(Value::Null)},b't'=>{self.literal("true")?;Ok(Value::Bool(true))},b'f'=>{self.literal("false")?;Ok(Value::Bool(false))},
            b'['=>{ensure(depth<16)?;self.pos+=1;self.ws();let mut a=Vec::new();if self.peek()==Some(b']'){self.pos+=1;return Ok(Value::Array(a));}
                loop{ensure(a.len()<512)?;a.push(self.value(depth+1)?);self.ws();if self.peek()==Some(b']'){self.pos+=1;break;}self.byte(b',')?;}
                Ok(Value::Array(a))},
            b'{'=>{ensure(depth<16)?;self.pos+=1;self.ws();let mut fields:Vec<(String,Value)>=Vec::new();if self.peek()==Some(b'}'){self.pos+=1;return Ok(Value::Object(fields));}
                loop{self.ws();ensure(fields.len()<256)?;let name=self.text()?;ensure(!fields.iter().any(|(n,_)|n==&name))?;self.ws();self.byte(b':')?;let value=self.value(depth+1)?;fields.push((name,value));self.ws();if self.peek()==Some(b'}'){self.pos+=1;break;}self.byte(b',')?;}
                Ok(Value::Object(fields))},
            b'-'|b'0'..=b'9'=>{let start=self.pos;if self.peek()==Some(b'-'){self.pos+=1;}
                if self.peek()==Some(b'0'){self.pos+=1;ensure(!self.peek().is_some_and(|b|b.is_ascii_digit()))?;}
                else{ensure(self.peek().is_some_and(|b|matches!(b,b'1'..=b'9')))?;while self.peek().is_some_and(|b|b.is_ascii_digit()){self.pos+=1;}}
                let n=self.text.get(start..self.pos).ok_or(Error::Json)?.parse::<i32>().map_err(|_|Error::Json)?;Ok(Value::Int(n))},
            _=>Err(Error::Json)
        }
    }
}
pub fn parse(bytes:&[u8])->Result<Value,Error>{
    let text=core::str::from_utf8(bytes).map_err(|_|Error::Json)?;let mut p=Parser{text,pos:0};let value=p.value(0)?;p.ws();ensure(p.pos==text.len())?;Ok(value)
}
fn write_text(s:&str,out:&mut Vec<u8>){
    out.push(b'"');let bytes=s.as_bytes();let mut pos=0;
    while pos<bytes.len(){
        let end=text_span(bytes,pos);out.extend_from_slice(&bytes[pos..end]);pos=end;
        if pos==bytes.len(){break;}let b=bytes[pos];pos+=1;
        match b{b'"'=>out.extend_from_slice(b"\\\""),b'\\'=>out.extend_from_slice(b"\\\\"),b'\n'=>out.extend_from_slice(b"\\n"),b'\r'=>out.extend_from_slice(b"\\r"),b'\t'=>out.extend_from_slice(b"\\t"),8=>out.extend_from_slice(b"\\b"),12=>out.extend_from_slice(b"\\f"),n=>{out.extend_from_slice(b"\\u00");out.push(b"0123456789abcdef"[(n>>4) as usize]);out.push(b"0123456789abcdef"[(n&15) as usize]);}}
    }out.push(b'"');
}
fn write(v:&Value,out:&mut Vec<u8>){
    match v{Value::Null=>out.extend_from_slice(b"null"),Value::Bool(b)=>out.extend_from_slice(if *b{b"true"}else{b"false"}),Value::Text(s)=>write_text(s,out),Value::Int(n)=>{
        let mut n=*n as i64;if n<0{out.push(b'-');n=-n;}let mut buf=[0u8;10];let mut i=10;loop{i-=1;*buf.get_mut(i).unwrap()=b'0'+(n%10) as u8;n/=10;if n==0{break;}}out.extend_from_slice(buf.get(i..).unwrap());},
        Value::Array(a)=>{out.push(b'[');for(i,v)in a.iter().enumerate(){if i>0{out.push(b',');}write(v,out);}out.push(b']');},
        Value::Object(fields)=>{out.push(b'{');for(i,(n,v))in fields.iter().enumerate(){if i>0{out.push(b',');}write_text(n,out);out.push(b':');write(v,out);}out.push(b'}');}
    }
}
pub fn serialize(v:&Value)->Vec<u8>{let mut out=Vec::new();write(v,&mut out);out}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn strict_json(){for b in [b"{\"x\":0,\"x\":1}".as_slice(),b"1.0",b"01",b"\"\\ud800\"",b"[1,]",b"true false"]{assert!(parse(b).is_err());}}
    #[test]fn unicode_and_escape_roundtrip(){let s=String::from("😀\n\0\\\"");let encoded=serialize(&Value::Text(s.clone()));let Value::Text(back)=parse(&encoded).unwrap()else{panic!()};assert_eq!(s,back);let Value::Text(pair)=parse(br#""\ud83d\ude00""#).unwrap()else{panic!()};assert_eq!(pair,"😀");}
    #[test]fn long_utf8_spans_preserve_boundaries_and_reject_raw_controls(){
        let s="<section>日本語 😀 & text</section>".repeat(16384);let encoded=serialize(&Value::Text(s.clone()));
        let Value::Text(back)=parse(&encoded).unwrap()else{panic!()};assert_eq!(s,back);
        assert!(parse(b"\"before\x01after\"").is_err());assert!(parse(b"\"unterminated").is_err());
    }
    #[test]fn parser_bounds_empty_container_depth(){
        for leaf in ["0","[]","{}"]{
            let valid=alloc::format!("{}{}{}","[".repeat(15),leaf,"]".repeat(15));assert!(parse(valid.as_bytes()).is_ok());
            let invalid=alloc::format!("{}{}{}","[".repeat(17),leaf,"]".repeat(17));assert!(parse(invalid.as_bytes()).is_err());
        }
    }
    #[test]fn word_scans_preserve_all_ascii_escapes_at_every_alignment(){
        for offset in 0..16{
            let mut s="日".repeat(offset);for c in 0u8..=127{s.push(c as char);}s.push_str("😀tail");
            let Value::Text(back)=parse(&serialize(&Value::Text(s.clone()))).unwrap()else{panic!()};assert_eq!(s,back);
        }
    }
}
