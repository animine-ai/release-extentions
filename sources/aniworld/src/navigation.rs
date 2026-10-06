use alloc::{string::String,vec,vec::Vec};
use arex_sdk::*;
use bounded::{decode,encode};
use crate::{error,identity,page,route,Result};
pub(crate) fn navigation_url(c:&NavigationContextV1)->Option<String>{
    if !identity(&c.extension_id,&c.provider_id){return None;}
    let url=match c.target_kind{
        NavigationTargetKind::OVERVIEW=>{if c.provider_episode.is_some()||c.track.is_some(){return None;}route::series(&c.provider_series_key,c.source_season)?},
        NavigationTargetKind::EPISODE=>{if !matches!(c.track,Some(ObservationTrack::DE_SUB|ObservationTrack::DE_DUB)){return None;}route::exact(&c.provider_series_key,c.source_season?,c.provider_episode.as_deref()?)?},
    };
    if let Some(hint)=&c.provider_route_hint{
        let r=route::parse(hint)?;
        if r.key!=c.provider_series_key||r.season.is_some_and(|s|Some(s)!=c.source_season)||r.episode.as_ref().is_some_and(|e|Some(e)!=c.provider_episode.as_ref()){return None;}
    }
    Some(url)
}
pub fn nav_plan(bytes:&[u8])->Vec<u8>{
    let Ok(c)=decode::<NavigationContextV1>(bytes,65536)else{return error();};
    let requests=navigation_url(&c).map(|url|vec![NavigationRequestSpecV1{request_id:"navigation".into(),url}]).unwrap_or_default();
    let _=abi::diagnostic_text("de.aniworld navigation plan");
    encode(&NavigationPlanOutputV1{schema_version:1,requests},65536).unwrap_or_else(|_|error())
}
pub fn nav_parse(bytes:&[u8])->Vec<u8>{
    let Ok(i)=decode::<NavigationParseInputV1>(bytes,4*1024*1024)else{return error();};
    let target=(||->Result<ProviderNavigationTargetV1>{
        let c=&i.context;let expected=navigation_url(c).ok_or(())?;
        if i.responses.len()!=1{return Err(());}let r=&i.responses[0];if r.request_id!="navigation"{return Err(());}
        let d=page::page(&r.status,r.http_status,r.final_url.as_deref(),r.body_utf8.as_deref(),r.source_hash.as_deref(),&expected)?;
        match c.target_kind{
            NavigationTargetKind::OVERVIEW=>{page::series(&d,&expected)?;},
            NavigationTargetKind::EPISODE=>{let(_,tracks)=page::episode(&d,&expected,c.source_season.ok_or(())?,c.provider_episode.as_deref().ok_or(())?)?;if !tracks.contains(c.track.as_ref().ok_or(())?){return Err(());}},
        }
        Ok(ProviderNavigationTargetV1{schema_version:1,extension_id:c.extension_id.clone(),provider_id:c.provider_id.clone(),target_kind:c.target_kind.clone(),provider_series_key:c.provider_series_key.clone(),source_season:c.source_season,provider_episode:c.provider_episode.clone(),track:c.track.clone(),url:expected,request_id:Some(r.request_id.clone()),source_hash:r.source_hash.clone(),diagnostics:vec![]})
    })();
    let _=abi::diagnostic_text("de.aniworld navigation parse");
    encode(&NavigationParseOutputV1{schema_version:1,targets:target.map(|t|vec![t]).unwrap_or_default()},65536).unwrap_or_else(|_|error())
}
