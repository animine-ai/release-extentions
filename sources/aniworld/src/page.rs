use alloc::{string::{String,ToString},vec,vec::Vec};
use arex_sdk::{ExtensionResponseStatus,ObservationTrack};
use crate::{html::{Document,Node},route,Result};
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
pub fn track(n:&Node)->Result<ObservationTrack>{
    let mut markers=String::new();
    for name in ["src","data-src","alt","title"]{if let Some(v)=n.attr(name){markers.push_str(v);markers.push(' ');}}
    let s=markers.to_ascii_lowercase();
    let sub=s.contains("japanese-german.svg")||((s.contains("untertitel")||s.contains("ger-sub"))&&(s.contains("deutsch")||s.contains("german")||s.contains("ger-sub")||s.contains("de untertitel")));
    let dub=s.contains("/german.svg")||(!s.contains("untertitel")&&!s.contains("ger-sub")&&(s.contains("auf deutsch")||s.contains("deutsche flagge")||s.contains("german flag")));
    if sub&&dub{return Err(());}Ok(if sub{ObservationTrack::DE_SUB}else if dub{ObservationTrack::DE_DUB}else{ObservationTrack::UNKNOWN})
}
pub fn tracks(d:&Document,n:usize)->Result<Vec<ObservationTrack>>{
    let mut out=vec![];let mut count=0;
    for i in d.descendants(n){if d.nodes[i].tag=="img"&&d.nodes[i].class("flag"){
        count+=1;if count>16{return Err(());}let t=track(&d.nodes[i])?;if !out.contains(&t){out.push(t);}
    }}
    if out.is_empty(){out.push(ObservationTrack::UNKNOWN);}Ok(out)
}
pub fn episode(d:&Document,expected:&str,season:i32,number:&str)->Result<(String,Vec<ObservationTrack>)>{
    let title=series(d,expected)?;
    let n=d.unique(1..d.nodes.len(),|n|n.class("hosterSiteTitle"))?;
    if d.nodes[n].attr("data-season").and_then(|s|route::integer(s,9999))!=Some(season)||d.nodes[n].attr("data-episode").and_then(route::episode).as_deref()!=Some(number){return Err(());}
    let video=d.unique(1..d.nodes.len(),|n|n.class("hosterSiteVideo"))?;
    let language=d.unique(d.descendants(video),|n|n.class("changeLanguageBox"))?;
    let mut mappings:Vec<(String,ObservationTrack)>=vec![];
    for i in d.descendants(language){if d.nodes[i].tag=="img"{
        let key=d.nodes[i].attr("data-lang-key").ok_or(())?;
        if key.is_empty()||key.len()>4||!key.bytes().all(|b|b.is_ascii_digit())||mappings.iter().any(|(k,_)|k==key){return Err(());}
        let t=track(&d.nodes[i])?;mappings.push((key.to_string(),t));if mappings.len()>16{return Err(());}
    }}
    let mut available=vec![];let mut rows=0;
    for i in d.descendants(video){if d.nodes[i].tag=="li"&&d.nodes[i].attr("data-lang-key").is_some(){
        rows+=1;if rows>256{return Err(());}
        let k=d.nodes[i].attr("data-lang-key").ok_or(())?;
        let Some((_,t))=mappings.iter().find(|(v,_)|v==k)else{continue;};
        let links:Vec<_>=d.descendants(i).filter(|n|d.nodes[*n].tag=="a"&&d.nodes[*n].class("watchEpisode")).collect();
        if links.len()!=1{continue;}
        let Some(id)=d.nodes[links[0]].attr("href").and_then(|v|v.strip_prefix("/redirect/"))else{continue;};
        if id.is_empty()||id.len()>16||!id.bytes().all(|b|b.is_ascii_digit()){continue;}
        if !available.contains(t){available.push(t.clone());}
    }}
    Ok((title,available))
}
