use alloc::{collections::BTreeMap,string::{String,ToString},vec,vec::Vec};
use arex_sdk::{ExtensionResponseStatus,ObservationTrack};
use crate::{html::{find_byte,Document,Node},route,Result};
pub fn hash(value:Option<&str>)->bool{value.is_some_and(|s|s.len()==64&&s.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)))}
pub fn page<'a>(status:&ExtensionResponseStatus,http:Option<i32>,url:Option<&str>,body:Option<&'a str>,hash_value:Option<&str>,expected:&str)->Result<Document<'a>>{
    if *status!=ExtensionResponseStatus::OK||!http.is_some_and(|s|(200..300).contains(&s))||url!=Some(expected)||!hash(hash_value){return Err(());}
    let body=body.ok_or(())?;if body.trim().is_empty(){return Err(());}Document::parse(body)
}
pub fn series(d:&Document,expected:&str)->Result<String>{
    let n=d.unique(1..d.nodes.len(),|n|n.tag=="link"&&n.attr("rel").is_some_and(|r|r.split_ascii_whitespace().any(|v|v=="canonical")))?;
    if d.nodes[n].attr("href").and_then(route::url).as_deref()!=Some(expected){return Err(());}
    let head=d.unique(1..d.nodes.len(),|n|n.class("series-title"))?;
    let h=d.unique(d.descendants(head),|n|n.tag=="h1")?;d.text(h,1024)
}
#[derive(Clone)]
enum TrackClass { German(ObservationTrack), Foreign }
const FOREIGN_MARKERS:&[&str]=&["english","englisch","french","franz","spanish","spanisch","italian","italien","portugu","polish","poln","russian","russisch","japanese","japanisch","korean","chinese","chines","dutch","niederl","danish","dän","daen","swedish","schwed","norwegian","norweg","finnish","finn","czech","tschech","turkish","türk","tuerk","romanian","rumän","rumaen","hungarian","ungar","greek","griech","originalton"];
fn has(s:&str,needle:&str)->bool{
    let bytes=s.as_bytes();let pattern=needle.as_bytes();let mut from=0;
    while let Some(at)=find_byte(bytes,from,pattern[0]){
        if bytes.get(at..at+pattern.len())==Some(pattern){return true;}from=at+1;
    }false
}
fn known_foreign(s:&str)->bool{FOREIGN_MARKERS.iter().any(|m|has(s,m))}
fn nearby_track_cue(s:&str,start:usize,end:usize)->bool{
    let cues=["untertitel","subtitle","flagge","flag","sprache","language","auf","on","originalton"];
    let is_cue=|v:&str|cues.iter().any(|cue|v==*cue||v.starts_with(*cue));
    s.get(..start).unwrap_or("").rsplit(|c:char|!c.is_alphanumeric()).filter(|v|!v.is_empty()).take(2).any(&is_cue)||
        s.get(end..).unwrap_or("").split(|c:char|!c.is_alphanumeric()).filter(|v|!v.is_empty()).take(2).any(&is_cue)
}
fn foreign_word(s:&str)->bool{
    let prefixes:&[&str]=match s.as_bytes().first(){
        Some(b'e')=>&["english","englisch"],Some(b'f')=>&["french","franz","finn"],
        Some(b's')=>&["spanish","spanisch","swedish","schwed"],Some(b'i')=>&["italian","italien"],
        Some(b'p')=>&["portugu","polish","poln"],Some(b'r')=>&["russian","russisch","romanian","rumän","rumaen"],
        Some(b'j')=>&["japanese","japanisch"],Some(b'k')=>&["korean"],Some(b'c')=>&["chinese","chines","czech"],
        Some(b'd')=>&["dutch","danish","dän","daen"],Some(b'n')=>&["niederl","norwegian","norweg"],
        Some(b't')=>&["tschech","turkish","türk","tuerk"],Some(b'h')=>&["hungarian"],Some(b'u')=>&["ungar"],
        Some(b'g')=>&["greek","griech"],Some(b'o')=>&["originalton"],_=>&[],
    };prefixes.iter().any(|p|s.starts_with(p))
}
fn explicit_foreign_text(s:&str)->bool{
    let s=s.to_ascii_lowercase();let mut start=None;
    for (at,c) in s.char_indices().chain(core::iter::once((s.len(),' '))){
        if c.is_alphabetic(){if start.is_none(){start=Some(at);}}
        else if let Some(begin)=start.take(){if foreign_word(&s[begin..at])&&nearby_track_cue(&s,begin,at){return true;}}
    }false
}
fn foreign_asset(value:&str)->bool{
    let file=value.to_ascii_lowercase();let file=file.rsplit('/').next().unwrap_or(&file);let file=file.split('?').next().unwrap_or(file);
    let Some(stem)=file.strip_suffix(".svg")else{return false;};
    let language=stem.rsplit('-').next().unwrap_or(stem);foreign_word(language)
}
fn explicit_foreign_marker(n:&Node)->bool{
    ["src","data-src"].iter().any(|name|n.attr(name).is_some_and(foreign_asset))||
        ["alt","title"].iter().any(|name|n.attr(name).is_some_and(explicit_foreign_text))
}
fn track(n:&Node)->Result<TrackClass>{
    let mut markers=String::new();
    for name in ["src","data-src","alt","title"]{if let Some(v)=n.attr(name){markers.push_str(v);markers.push(' ');}}
    markers.make_ascii_lowercase();let s=markers;
    let subtitle=has(&s,"untertitel");let ger_sub=has(&s,"ger-sub");
    let sub=has(&s,"japanese-german.svg")||((subtitle||ger_sub)&&(has(&s,"deutsch")||has(&s,"german")||ger_sub||has(&s,"de untertitel")));
    let dub=has(&s,"/german.svg")||(!subtitle&&!ger_sub&&(has(&s,"auf deutsch")||has(&s,"deutsche flagge")||has(&s,"german flag")));
    if (sub&&dub)||((sub||dub)&&explicit_foreign_marker(n)){return Err(());}
    if sub{return Ok(TrackClass::German(ObservationTrack::DE_SUB));}
    if dub{return Ok(TrackClass::German(ObservationTrack::DE_DUB));}
    if known_foreign(&s){return Ok(TrackClass::Foreign);}
    Err(())
}
pub fn tracks(d:&Document,n:usize)->Result<Vec<ObservationTrack>>{
    tracks_impl(d,n,None)
}
pub struct TrackCache(BTreeMap<u64,Vec<(usize,TrackClass)>>);
impl TrackCache { pub fn new()->Self{Self(BTreeMap::new())} }
fn marker_hash(n:&Node)->u64{
    let mut h=14695981039346656037u64;
    for name in ["src","data-src","alt","title"]{
        let bytes=n.attr(name).unwrap_or("").as_bytes();let mut i=0;
        h=(h^bytes.len() as u64).wrapping_mul(1099511628211);
        while bytes.len()-i>=8{
            // SAFETY: the eight-byte guard covers this unaligned word.
            let word=u64::from_le(unsafe{core::ptr::read_unaligned(bytes.as_ptr().add(i).cast::<u64>())});
            h=(h^word).rotate_left(13).wrapping_mul(1099511628211);i+=8;
        }
        while i<bytes.len(){h=(h^bytes[i] as u64).wrapping_mul(1099511628211);i+=1;}
        h=(h^255).wrapping_mul(1099511628211);
    }h
}
fn same_markers(a:&Node,b:&Node)->bool{
    ["src","data-src","alt","title"].iter().all(|name|a.attr(name)==b.attr(name))
}
fn tracks_impl(d:&Document,n:usize,mut cache:Option<&mut TrackCache>)->Result<Vec<ObservationTrack>>{
    let mut out=vec![];let mut count=0;
    for i in d.descendants(n){if d.nodes[i].tag=="img"&&d.nodes[i].class("flag"){
        count+=1;if count>16{return Err(());}
        let classified=if let Some(cache)=cache.as_deref_mut(){
            let h=marker_hash(&d.nodes[i]);
            if let Some(prior)=cache.0.get(&h).and_then(|items|items.iter().find(|(j,_)|same_markers(&d.nodes[*j],&d.nodes[i]))){prior.1.clone()}
            else{let found=track(&d.nodes[i])?;cache.0.entry(h).or_default().push((i,found.clone()));found}
        }else{track(&d.nodes[i])?};
        if let TrackClass::German(t)=classified{if !out.contains(&t){out.push(t);}}
    }}
    if count==0{return Err(());}Ok(out)
}
pub fn tracks_cached(d:&Document,n:usize,cache:&mut TrackCache)->Result<Vec<ObservationTrack>>{tracks_impl(d,n,Some(cache))}
pub fn episode(d:&Document,expected:&str,season:i32,number:&str)->Result<(String,Vec<ObservationTrack>)>{
    let title=series(d,expected)?;
    let n=d.unique(1..d.nodes.len(),|n|n.class("hosterSiteTitle"))?;
    if d.nodes[n].attr("data-season").and_then(|s|route::integer(s,9999))!=Some(season)||d.nodes[n].attr("data-episode").and_then(route::episode).as_deref()!=Some(number){return Err(());}
    let video=d.unique(1..d.nodes.len(),|n|n.class("hosterSiteVideo"))?;
    let language=d.unique(d.descendants(video),|n|n.class("changeLanguageBox"))?;
    let mut mappings:Vec<(String,Option<ObservationTrack>)>=vec![];
    for i in d.descendants(language){if d.nodes[i].tag=="img"{
        let key=d.nodes[i].attr("data-lang-key").ok_or(())?;
        if key.is_empty()||key.len()>4||!key.bytes().all(|b|b.is_ascii_digit())||mappings.iter().any(|(k,_)|k==key){return Err(());}
        let mapped=match track(&d.nodes[i])?{TrackClass::German(t)=>Some(t),TrackClass::Foreign=>None};
        mappings.push((key.to_string(),mapped));if mappings.len()>16{return Err(());}
    }}
    let mut available=vec![];let mut rows=0;
    for i in d.descendants(video){if d.nodes[i].tag=="li"&&d.nodes[i].attr("data-lang-key").is_some(){
        rows+=1;if rows>256{return Err(());}
        let k=d.nodes[i].attr("data-lang-key").ok_or(())?;
        let Some((_,Some(t)))=mappings.iter().find(|(v,_)|v==k)else{continue;};
        let links:Vec<_>=d.descendants(i).filter(|n|d.nodes[*n].tag=="a"&&d.nodes[*n].class("watchEpisode")).collect();
        if links.len()!=1{continue;}
        let Some(id)=d.nodes[links[0]].attr("href").and_then(|v|v.strip_prefix("/redirect/"))else{continue;};
        if id.is_empty()||id.len()>16||!id.bytes().all(|b|b.is_ascii_digit()){continue;}
        if !available.contains(t){available.push(t.clone());}
    }}
    Ok((title,available))
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn cached_flags_still_reject_conflicting_attributes(){
        let d=Document::parse("<main><a><img class='flag' src='/public/img/japanese-german.svg' alt='Deutscher Untertitel'></a><a><img class='flag' src='/public/img/japanese-german.svg' alt='Deutscher Untertitel'></a><a><img class='flag' src='/public/img/japanese-german.svg' alt='English Subtitle Flag'></a></main>").unwrap();
        let rows:Vec<_>=d.nodes.iter().enumerate().filter(|(_,n)|n.tag=="a").map(|(i,_)|i).collect();
        let mut cache=TrackCache::new();
        assert_eq!(tracks_cached(&d,rows[0],&mut cache).unwrap(),vec![ObservationTrack::DE_SUB]);
        assert_eq!(tracks_cached(&d,rows[1],&mut cache).unwrap(),vec![ObservationTrack::DE_SUB]);
        assert!(tracks_cached(&d,rows[2],&mut cache).is_err());
    }
}
