use alloc::{borrow::Cow,string::{String,ToString}};
pub fn join(parts:&[&str])->String{let mut out=String::new();for part in parts{out.push_str(part);}out}
pub const ORIGIN:&str="https://aniworld.to";
pub const CALENDAR:&str="https://aniworld.to/animekalender";
pub const RECENT:&str="https://aniworld.to/neue-episoden";
pub const POSTPONEMENT:&str="https://aniworld.to/support/frage/anime-verschiebungen";
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Route { pub key:String,pub season:Option<i32>,pub episode:Option<String> }
pub fn key(value:&str)->bool { !value.is_empty() && value.len()<=128 && value.bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit()||b==b'-') && !value.starts_with('-') && !value.ends_with('-') && !value.contains("--") }
pub fn integer(value:&str,max:i32)->Option<i32>{
    if value.is_empty() || value.len()>4 || !value.bytes().all(|b|b.is_ascii_digit()) {return None;}
    value.parse::<i32>().ok().filter(|v|*v>0 && *v<=max)
}
pub fn episode(value:&str)->Option<String>{
    let n=integer(value,9999)?; if n.to_string()!=value{return None;} Some(value.into())
}
fn path(value:&str)->Option<&str> {
    if value.len()>2048 || value.bytes().any(|b|b<=32||b==127||b"?#%\\".contains(&b)){return None;}
    let path=if let Some(p)=value.strip_prefix(ORIGIN){if !p.starts_with('/'){return None;}p}
        else if value.starts_with('/') && !value.starts_with("//"){value}else{return None;};
    if !path.is_ascii() || path.contains("//") || path.contains("/./") || path.contains("/../"){return None;}
    Some(path)
}
pub fn url(value:&str)->Option<String> {Some(join(&[ORIGIN,path(value)?]))}
pub fn parse(value:&str)->Option<Route>{
    let path=path(value)?.strip_prefix("/anime/stream/")?;
    let mut parts=path.split('/');let series_key=parts.next()?;
    if !key(series_key){return None;}
    let season_part=parts.next();let episode_part=parts.next();if parts.next().is_some(){return None;}
    match (season_part,episode_part) {
        (None,None)=>Some(Route{key:series_key.into(),season:None,episode:None}),
        (Some(raw),None)=>{let raw=raw.strip_prefix("staffel-")?;let season=integer(raw,9999)?;if season.to_string()!=raw{return None;}Some(Route{key:series_key.into(),season:Some(season),episode:None})},
        (Some(raw),Some(e))=>{let raw=raw.strip_prefix("staffel-")?;let season=integer(raw,9999)?;if season.to_string()!=raw{return None;}let e=episode(e.strip_prefix("episode-")?)?;Some(Route{key:series_key.into(),season:Some(season),episode:Some(e)})},
        _=>None,
    }
}
pub fn series(key:&str,season:Option<i32>)->Option<String>{
    if !self::key(key)||season.is_some_and(|s|!(1..=9999).contains(&s)){return None;}
    let base=join(&[ORIGIN,"/anime/stream/",key]);Some(match season {Some(s)=>join(&[&base,"/staffel-",&s.to_string()]),None=>base})
}
pub fn exact(key:&str,season:i32,episode:&str)->Option<String>{
    let base=series(key,Some(season))?;self::episode(episode)?;Some(join(&[&base,"/episode-",episode]))
}
pub fn label(value:&str)->Option<(i32,String)>{
    let label:Cow<'_,str>=if value.chars().any(char::is_whitespace){Cow::Owned(value.chars().filter(|c|!c.is_whitespace()).collect())}else{Cow::Borrowed(value)};
    let rest=label.strip_prefix('S')?;let (s,e)=rest.split_once('E')?;
    Some((integer(s,9999)?,integer(e,9999)?.to_string()))
}
// Wall dates are retained without inventing a timezone/year or DST resolution.
pub fn date(value:&str)->Option<String>{
    let mut dates=value.split_whitespace().filter(|p|p.len()==10 && p.as_bytes().get(2)==Some(&b'.') && p.as_bytes().get(5)==Some(&b'.'));
    let numeric=dates.next()?;if dates.next().is_some(){return None;}
    let parts:alloc::vec::Vec<_>=numeric.split('.').collect();if parts.len()!=3{return None;}
    if parts[0].len()!=2||parts[1].len()!=2||parts[2].len()!=4{return None;}
    let d=integer(parts[0],31)?;let m=integer(parts[1],12)?;let y=integer(parts[2],9999)?;
    let max=match m {4|6|9|11=>30,2=>if y%4==0 && (y%100!=0||y%400==0){29}else{28},_=>31};
    if d>max{return None;}Some(numeric.into())
}
pub fn time(value:&str)->Option<String>{
    let mut times=value.split_whitespace().filter(|s|s.len()==5 && s.as_bytes().get(2)==Some(&b':'));
    let t=times.next()?;if times.next().is_some(){return None;}
    let (h,m)=t.split_once(':')?;if !h.bytes().all(|b|b.is_ascii_digit())||!m.bytes().all(|b|b.is_ascii_digit()){return None;}
    let h=h.parse::<u32>().ok()?;let m=m.parse::<u32>().ok()?;if h>23||m>59{return None;}Some(t.into())
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn route_fail_closed(){for u in ["http://aniworld.to/anime/stream/a","https://aniworld.to.evil/anime/stream/a","//aniworld.to/anime/stream/a","/anime/stream/a/../b","/anime/stream/A","/anime/stream/a/staffel-01","/anime/stream/a/staffel-1/episode-1.5","/anime/stream/a?x=1"]{assert!(parse(u).is_none(),"{u}");}}
    #[test] fn wall_dates(){assert_eq!(date("Mi, 30.09.2026 (heute)"),Some("30.09.2026".into()));for s in ["29.02.2025","32.09.2026","31.04.2026","2026-09-30"]{assert_eq!(date(s),None);}assert_eq!(time("~ 02:30 Uhr"),Some("02:30".into()));assert_eq!(time("25:30 Uhr"),None);}
}
