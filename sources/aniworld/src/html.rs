//! Bounded, non-executing HTML reader. This is not a browser or a forgiving HTML5
//! repair engine: ambiguous nesting/attributes and exhausted budgets fail closed.
use alloc::{string::{String, ToString}, vec, vec::Vec};

pub const MAX_NODES: usize = 16384;
pub struct Node<'a> {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub text: &'a str,
    pub parent: usize,
    pub end: usize,
}
impl Node<'_> {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }
    pub fn class(&self, name: &str) -> bool {
        self.attr("class").is_some_and(|v| v.split_ascii_whitespace().any(|c| c == name))
    }
}
pub struct Document<'a> { pub nodes: Vec<Node<'a>> }
type Result<T> = core::result::Result<T, ()>;
fn name_byte(b: u8) -> bool { b.is_ascii_alphanumeric() || b"-_:".contains(&b) }
fn space(b: u8) -> bool { b.is_ascii_whitespace() }
fn void(tag: &str) -> bool {
    matches!(tag, "area"|"base"|"br"|"col"|"embed"|"hr"|"img"|"input"|"link"|"meta"|"param"|"source"|"track"|"wbr")
}
pub fn entities(value: &str) -> Result<String> {
    let mut out = String::new(); let mut rest = value;
    while let Some(at) = rest.find('&') {
        out.push_str(rest.get(..at).ok_or(())?); rest = rest.get(at..).ok_or(())?;
        let Some(end) = rest.find(';').filter(|n| *n <= 16) else {
            out.push('&'); rest = rest.get(1..).ok_or(())?; continue;
        };
        let entity = rest.get(1..end).ok_or(())?;
        let c = match entity {
            "amp" => Some('&'), "lt" => Some('<'), "gt" => Some('>'),
            "quot" => Some('"'), "apos" => Some('\''), "nbsp" => Some(' '),
            _ => {
                let number = if let Some(hex) = entity.strip_prefix("#x").or_else(||entity.strip_prefix("#X")) {
                    u32::from_str_radix(hex, 16).ok()
                } else if let Some(decimal) = entity.strip_prefix('#') { decimal.parse::<u32>().ok() }
                else { None };
                number.and_then(char::from_u32)
            }
        };
        if let Some(c) = c { if c.is_control() { return Err(()); } out.push(c); }
        else { out.push_str(rest.get(..end+1).ok_or(())?); }
        rest = rest.get(end+1..).ok_or(())?;
    }
    out.push_str(rest); Ok(out)
}
impl<'a> Document<'a> {
    pub fn parse(input: &'a str) -> Result<Self> {
        if input.is_empty() || input.len() > 2*1024*1024 { return Err(()); }
        let bytes = input.as_bytes(); let mut i = 0;
        let mut nodes = vec![Node { tag: "root".into(), attrs: vec![], text: "", parent: 0, end: 0 }];
        let mut stack = vec![0usize];
        while i < bytes.len() {
            if nodes.len() >= MAX_NODES { return Err(()); }
            if bytes[i] != b'<' {
                let start = i; while i < bytes.len() && bytes[i] != b'<' { i += 1; }
                let text = input.get(start..i).ok_or(())?;
                if !text.trim().is_empty() { let n=nodes.len(); nodes.push(Node { tag: "#text".into(), attrs: vec![], text, parent: *stack.last().ok_or(())?, end: n+1 }); }
                continue;
            }
            if input.get(i..).ok_or(())?.starts_with("<!--") {
                let n = input.get(i+4..).ok_or(())?.find("-->").ok_or(())?; i += n+7; continue;
            }
            if bytes.get(i+1) == Some(&b'!') {
                let n=input.get(i..).ok_or(())?.find('>').ok_or(())?; i+=n+1; continue;
            }
            i += 1; let closing = bytes.get(i)==Some(&b'/'); if closing { i += 1; }
            let start=i; while i<bytes.len() && name_byte(bytes[i]) { i+=1; }
            if start==i { return Err(()); }
            let tag=input.get(start..i).ok_or(())?.to_ascii_lowercase();
            let mut attrs: Vec<(String,String)>=vec![]; let mut self_closed=false;
            loop {
                while i<bytes.len() && space(bytes[i]) { i+=1; }
                if bytes.get(i)==Some(&b'>') { i+=1; break; }
                if bytes.get(i)==Some(&b'/') && bytes.get(i+1)==Some(&b'>') { self_closed=true; i+=2; break; }
                if closing || attrs.len()>=32 { return Err(()); }
                let a=i; while i<bytes.len() && name_byte(bytes[i]) { i+=1; }
                if a==i { return Err(()); }
                let key=input.get(a..i).ok_or(())?.to_ascii_lowercase();
                if attrs.iter().any(|(k,_)|k==&key) { return Err(()); }
                while i<bytes.len() && space(bytes[i]) { i+=1; }
                let mut value=String::new();
                if bytes.get(i)==Some(&b'=') {
                    i+=1; while i<bytes.len() && space(bytes[i]) { i+=1; }
                    let quoted=bytes.get(i).copied().filter(|b|*b==b'\'' || *b==b'"');
                    if quoted.is_some() { i+=1; }
                    let v=i;
                    if let Some(q)=quoted { while i<bytes.len() && bytes[i]!=q { i+=1; } if i==bytes.len() { return Err(()); } }
                    else { while i<bytes.len() && !space(bytes[i]) && bytes[i]!=b'>' { i+=1; } }
                    if i-v>8192 { return Err(()); }
                    value=entities(input.get(v..i).ok_or(())?)?; if quoted.is_some() { i+=1; }
                }
                attrs.push((key,value));
            }
            if closing {
                if void(&tag) { continue; }
                // Current provider series/episode pages omit only the outer #wrapper
                // close before </body>. Close that one exact layout node at this explicit
                // boundary. All other mismatches and incomplete documents still reject.
                if tag=="body"&&stack.len()==4{
                    let n=stack[3];let parent=stack[2];
                    if nodes[n].tag=="div"&&nodes[n].attr("id")==Some("wrapper")&&nodes[parent].tag=="body"{
                        nodes[n].end=nodes.len();stack.pop();
                    }
                }
                let current=*stack.last().ok_or(())?;
                if current==0 || nodes[current].tag!=tag { return Err(()); }
                nodes[current].end=nodes.len(); stack.pop(); continue;
            }
            // Raw content cannot influence selectors or observations.
            if matches!(tag.as_str(),"script"|"style"|"noscript") && !self_closed {
                let mut end=i;
                while end<bytes.len() {
                    if bytes[end]==b'<' && bytes.get(end+1)==Some(&b'/') &&
                        input.get(end+2..end+2+tag.len()).is_some_and(|s|s.eq_ignore_ascii_case(&tag)) { break; }
                    end+=1;
                }
                if end==bytes.len() { return Err(()); }
                let close=input.get(end..).ok_or(())?.find('>').ok_or(())?; i=end+close+1; continue;
            }
            let n=nodes.len();nodes.push(Node { tag, attrs, text:"", parent:*stack.last().ok_or(())?,end:n+1 });
            if !self_closed && !void(&nodes[n].tag) { stack.push(n); if stack.len()>64 { return Err(()); } }
        }
        if stack.len()!=1 { return Err(()); }
        nodes[0].end=nodes.len(); Ok(Self { nodes })
    }
    pub fn descendants(&self, n: usize) -> core::ops::Range<usize> { n+1..self.nodes[n].end }
    pub fn text(&self, n: usize, cap: usize) -> Result<String> {
        let raw=self.raw_text(n,cap)?;
        let result=raw.split_whitespace().collect::<Vec<_>>().join(" ");
        if result.is_empty() || result.len()>cap { return Err(()); } Ok(result)
    }
    pub fn raw_text(&self,n:usize,cap:usize)->Result<String>{
        let mut out=String::new();
        for i in self.descendants(n) {
            let node=&self.nodes[i];
            if node.tag=="#text" {out.push_str(&entities(node.text)?);out.push(' ');}
            if node.tag=="br" {out.push('\n');}
            if out.len()>cap {return Err(());}
        }
        Ok(out)
    }
    pub fn unique<F:Fn(&Node)->bool>(&self, range:core::ops::Range<usize>, test:F)->Result<usize>{
        let mut found=None;
        for i in range { if test(&self.nodes[i]) {if found.is_some(){return Err(());}found=Some(i);} }
        found.ok_or(())
    }
    pub fn nearest_class(&self,mut n:usize,class:&str)->Option<usize>{
        for _ in 0..64 { if self.nodes[n].class(class) {return Some(n);} if n==0{return None;}n=self.nodes[n].parent; }
        None
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn bounds_and_truncation(){assert!(Document::parse("<a>").is_err());assert!(Document::parse("<a href='x' href='y'></a>").is_err());assert!(Document::parse(&"<div>".repeat(65)).is_err());}
    #[test] fn entities_and_inert_bytes(){let d=Document::parse("<main><script><a href='bad'>fake</a></script><h1>A &amp; B &#039; &#x1f600;</h1></main>").unwrap();assert_eq!(d.text(1,128).unwrap(),"A & B ' 😀");assert!(entities("&#0;").is_err());}
    #[test] fn observed_outer_wrapper_omission_is_narrow(){
        assert!(Document::parse("<html><body><div id='wrapper'><h1>A</h1></body></html>").is_ok());
        for s in ["<html><body><div id='other'><h1>A</h1></body></html>","<html><body><div id='wrapper'><div></body></html>","<html><body><div id='wrapper'><h1>A</h1>"]{assert!(Document::parse(s).is_err());}
    }
}
