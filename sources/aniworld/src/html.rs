//! Bounded, non-executing HTML reader. This is not a browser or a forgiving HTML5
//! repair engine: ambiguous nesting/attributes and exhausted budgets fail closed.
use alloc::{borrow::Cow,string::String, vec, vec::Vec};

pub const MAX_NODES: usize = 16384;
pub struct Node<'a> {
    pub tag: Cow<'a,str>,
    pub attrs: Vec<(Cow<'a,str>, Cow<'a,str>)>,
    pub text: &'a str,
    pub parent: usize,
    pub end: usize,
}
impl Node<'_> {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_ref())
    }
    pub fn class(&self, name: &str) -> bool {
        self.attr("class").is_some_and(|v| v.split_ascii_whitespace().any(|c| c == name))
    }
}
pub struct Document<'a> { pub nodes: Vec<Node<'a>> }
type Result<T> = core::result::Result<T, ()>;
fn name_byte(b: u8) -> bool { b.is_ascii_alphanumeric() || b"-_:".contains(&b) }
fn space(b: u8) -> bool { b.is_ascii_whitespace() }
fn lower(value:&str)->Cow<'_,str>{if value.bytes().any(|b|b.is_ascii_uppercase()){Cow::Owned(value.to_ascii_lowercase())}else{Cow::Borrowed(value)}}
#[inline(always)]
fn find_byte(bytes:&[u8],mut pos:usize,needle:u8)->Option<usize>{
    let pattern=(needle as u64)*0x0101010101010101;
    while bytes.len()-pos>=8{
        let b=&bytes[pos..pos+8];let word=u64::from_le_bytes([b[0],b[1],b[2],b[3],b[4],b[5],b[6],b[7]])^pattern;
        if word.wrapping_sub(0x0101010101010101)&!word&0x8080808080808080!=0{break;}pos+=8;
    }
    while pos<bytes.len(){if bytes[pos]==needle{return Some(pos);}pos+=1;}None
}
fn comment_end(bytes:&[u8],mut pos:usize)->Option<usize>{
    while let Some(at)=find_byte(bytes,pos,b'-'){
        if bytes.get(at..at+3)==Some(b"-->"){return Some(at+3);}pos=at+1;
    }None
}
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
                let start = i; i=find_byte(bytes,i,b'<').unwrap_or(bytes.len());
                let text = input.get(start..i).ok_or(())?;
                if !text.trim().is_empty() { let n=nodes.len(); nodes.push(Node { tag: "#text".into(), attrs: vec![], text, parent: *stack.last().ok_or(())?, end: n+1 }); }
                continue;
            }
            if input.get(i..).ok_or(())?.starts_with("<!--") {
                i=comment_end(bytes,i+4).ok_or(())?;continue;
            }
            if bytes.get(i+1) == Some(&b'!') {
                i=find_byte(bytes,i,b'>').ok_or(())?+1;continue;
            }
            i += 1; let closing = bytes.get(i)==Some(&b'/'); if closing { i += 1; }
            let start=i; while i<bytes.len() && name_byte(bytes[i]) { i+=1; }
            if start==i { return Err(()); }
            let tag=lower(input.get(start..i).ok_or(())?);
            let mut attrs: Vec<(Cow<'a,str>,Cow<'a,str>)>=vec![]; let mut self_closed=false;
            loop {
                while i<bytes.len() && space(bytes[i]) { i+=1; }
                if bytes.get(i)==Some(&b'>') { i+=1; break; }
                if bytes.get(i)==Some(&b'/') && bytes.get(i+1)==Some(&b'>') { self_closed=true; i+=2; break; }
                if closing || attrs.len()>=32 { return Err(()); }
                let a=i; while i<bytes.len() && name_byte(bytes[i]) { i+=1; }
                if a==i { return Err(()); }
                let key=lower(input.get(a..i).ok_or(())?);
                if attrs.iter().any(|(k,_)|k==&key) { return Err(()); }
                while i<bytes.len() && space(bytes[i]) { i+=1; }
                let mut value=Cow::Borrowed("");
                if bytes.get(i)==Some(&b'=') {
                    i+=1; while i<bytes.len() && space(bytes[i]) { i+=1; }
                    let quoted=bytes.get(i).copied().filter(|b|*b==b'\'' || *b==b'"');
                    if quoted.is_some() { i+=1; }
                    let v=i;
                    if let Some(q)=quoted { i=find_byte(bytes,i,q).ok_or(())?; }
                    else { while i<bytes.len() && !space(bytes[i]) && bytes[i]!=b'>' { i+=1; } }
                    if i-v>8192 { return Err(()); }
                    let raw=input.get(v..i).ok_or(())?;
                    value=if raw.contains('&'){Cow::Owned(entities(raw)?)}else{Cow::Borrowed(raw)}; if quoted.is_some() { i+=1; }
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
            if matches!(tag.as_ref(),"script"|"style"|"noscript") && !self_closed {
                let mut end=i;
                while end<bytes.len() {
                    end=find_byte(bytes,end,b'<').unwrap_or(bytes.len());
                    if end==bytes.len(){break;}
                    if bytes.get(end+1)==Some(&b'/') &&
                        input.get(end+2..end+2+tag.len()).is_some_and(|s|s.eq_ignore_ascii_case(&tag)) &&
                        bytes.get(end+2+tag.len()).is_some_and(|b|*b==b'>'||space(*b)){ break; }
                    end+=1;
                }
                if end==bytes.len() { return Err(()); }
                i=find_byte(bytes,end,b'>').ok_or(())?+1;continue;
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
    #[test] fn byte_search_covers_every_boundary_and_utf8(){
        for n in 0..32{let text=alloc::format!("{}😀<{}", "a".repeat(n),"b".repeat(32));assert_eq!(find_byte(text.as_bytes(),0,b'<'),text.find('<'));assert_eq!(find_byte(text.as_bytes(),0,b'z'),None);}
        assert!(Document::parse("<main><!-- -- x --> <h1>A</h1></main>").is_ok());
        assert!(Document::parse("<main><!-- -- x").is_err());
    }
    #[test] fn observed_outer_wrapper_omission_is_narrow(){
        assert!(Document::parse("<html><body><div id='wrapper'><h1>A</h1></body></html>").is_ok());
        for s in ["<html><body><div id='other'><h1>A</h1></body></html>","<html><body><div id='wrapper'><div></body></html>","<html><body><div id='wrapper'><h1>A</h1>"]{assert!(Document::parse(s).is_err());}
    }
    #[test] fn mixed_case_entities_and_raw_close_boundaries(){
        let d=Document::parse("<MAIN CLASS='safe&amp;bound'><SCRIPT></scriptX><h1>fake</h1></SCRIPT><H1>Real</H1></MAIN>").unwrap();
        assert_eq!(d.nodes[1].attr("class"),Some("safe&bound"));assert_eq!(d.text(1,128).unwrap(),"Real");
    }
}
