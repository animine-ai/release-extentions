use alloc::{string::{String,ToString},vec,vec::Vec};
use arex_sdk::*;
use bounded::{decode,encode};
use crate::{error,identity,page,route,html::Document,Result};
fn diagnostic(code:&str)->ObservationDiagnosticV1{ObservationDiagnosticV1{code:code.into(),message:code.into()}}
fn notice_track(line:&str)->Result<Option<ObservationTrack>>{
    let low=line.to_ascii_lowercase();let sub=low.contains("(sub)");let dub=low.contains("(dub)");
    if sub&&dub{return Err(());}Ok(if sub{Some(ObservationTrack::DE_SUB)}else if dub{Some(ObservationTrack::DE_DUB)}else{None})
}
fn context(c:&ExtensionContextV1)->bool{
    identity(&c.extension_id,&c.provider_id)&&!c.source_roles.is_empty()&&
        !c.source_roles.iter().enumerate().any(|(i,r)|c.source_roles.iter().take(i).any(|v|v==r))&&
        !c.targets.iter().enumerate().any(|(i,t)|c.targets.iter().take(i).any(|v|v.target_token==t.target_token))
}
fn target_url(t:&ExtensionTargetV1)->Option<String>{
    if t.installment.kind!=ObservationInstallmentKind::EPISODE||t.track==ObservationTrack::UNKNOWN{return None;}
    let season=t.source_season?;if t.navigation_season.is_some_and(|s|s!=season){return None;}
    let number=t.installment.number.as_deref()?;let constructed=route::exact(&t.provider_series_key,season,number)?;
    if let Some(hint)=&t.provider_url{if route::url(hint)?!=constructed{return None;}}
    Some(constructed)
}
pub(crate) fn release_plan(c:&ExtensionContextV1)->Result<Vec<RequestSpec>>{
    if !context(c){return Err(());}let mut out=vec![];
    for role in &c.source_roles{
        let(id,url)=match role{SourceRole::CALENDAR=>("calendar",route::CALENDAR),SourceRole::RECENT=>("recent",route::RECENT),SourceRole::POSTPONEMENT=>("postponement",route::POSTPONEMENT),SourceRole::DIRECT=>continue};
        out.push(RequestSpec{request_id:id.into(),source_role:role.clone(),url:url.into(),method:ExtensionMethod::GET,target_token:None});
    }
    if c.source_roles.contains(&SourceRole::DIRECT){for(i,t)in c.targets.iter().enumerate(){
        if out.iter().filter(|r|r.source_role==SourceRole::DIRECT).count()==4{break;}
        if let Some(url)=target_url(t){
            out.push(RequestSpec{request_id:route::join(&["direct-",&i.to_string()]),source_role:SourceRole::DIRECT,url,method:ExtensionMethod::GET,target_token:Some(t.target_token.clone())});
        }
    }}Ok(out)
}
pub fn plan(bytes:&[u8])->Vec<u8>{
    let Ok(i)=decode::<PlanInputV1>(bytes,256*1024)else{return error();};let Ok(requests)=release_plan(&i.context)else{return error();};
    let _=abi::diagnostic_text("de.aniworld release plan");encode(&PlanOutputV1{schema_version:1,requests},65536).unwrap_or_else(|_|error())
}
fn observation(c:&ExtensionContextV1,r:&ResponseEnvelope,key:Option<String>,title:String,season:Option<i32>,number:Option<String>,track:ObservationTrack)->ProviderObservationV1{
    ProviderObservationV1{schema_version:1,extension_id:c.extension_id.clone(),provider_id:c.provider_id.clone(),request_id:r.request_id.clone(),source_role:r.source_role.clone(),provider_series_key:key,raw_title:title,source_season:season,navigation_season:season,installment:InstallmentV1{kind:if number.is_some(){ObservationInstallmentKind::EPISODE}else{ObservationInstallmentKind::UNKNOWN},number},track,
        claim_kind:match r.source_role{SourceRole::CALENDAR=>ObservationClaimKind::FORECAST,SourceRole::RECENT=>ObservationClaimKind::RELEASE_LISTING,SourceRole::POSTPONEMENT=>ObservationClaimKind::CORRECTION,SourceRole::DIRECT=>ObservationClaimKind::DIRECT_AVAILABILITY},source_date_text:None,source_time_text:None,source_raw_text:None,parsed_timestamp:None,approximate:false,schedule_marker:ObservationScheduleMarker::NONE,correction_marker:None,source_url:r.final_url.clone().unwrap_or_default(),source_hash:r.source_hash.clone().unwrap_or_default(),diagnostics:vec![]}
}
fn heading(d:&Document,text:&str)->Result<()>{let h=d.unique(1..d.nodes.len(),|n|n.tag=="h1")?;if d.text(h,256)?!=text{return Err(());}Ok(())}
fn push_unique(out:&mut Vec<ProviderObservationV1>,hashes:&mut Vec<u64>,o:ProviderObservationV1)->Result<()>{
    // This hash is only an equality prefilter. A collision still requires full DTO
    // equality, so no observation, language track or correction can be lost to it.
    let mut h=14695981039346656037u64;
    for s in [o.provider_series_key.as_deref().unwrap_or(""),o.installment.number.as_deref().unwrap_or("")]{for b in s.bytes(){h=(h^(b as u64)).wrapping_mul(1099511628211);}h=h.wrapping_mul(1099511628211);}
    h^=(o.source_season.unwrap_or(0) as u64)<<8;h^=o.track.clone() as u64;
    if hashes.iter().zip(out.iter()).any(|(v,p)|*v==h&&p==&o){return Ok(());}
    if out.len()>=512{return Err(());}hashes.push(h);out.push(o);Ok(())
}
fn listing(c:&ExtensionContextV1,r:&ResponseEnvelope,d:&Document)->Result<(Vec<ProviderObservationV1>,bool)>{
    let calendar=r.source_role==SourceRole::CALENDAR;heading(d,if calendar{"Animekalender"}else{"Neue Episoden"})?;
    let list=if calendar{0}else{d.unique(1..d.nodes.len(),|n|n.class("newEpisodeList"))?};
    let mut out=vec![];let mut hashes=vec![];let mut rows=0;let mut partial=false;
    for a in d.descendants(list){
        if d.nodes[a].tag!="a"{continue;}let Some(href)=d.nodes[a].attr("href")else{continue;};
        if !href.contains("/anime/stream/"){continue;}if calendar&&d.nearest_class(a,"calendarList").is_none(){continue;}
        rows+=1;if rows>512{return Err(());}
        let parsed=(||->Result<Vec<ProviderObservationV1>>{
            let route=route::parse(href).ok_or(())?;let season=route.season.ok_or(())?;let number=route.episode.ok_or(())?;
            let row=if calendar{a}else{d.nearest_class(d.nodes[a].parent,"col-md-12").ok_or(())?};
            if !calendar&&d.descendants(row).filter(|i|d.nodes[*i].tag=="a").count()!=1{return Err(());}
            let title_node=d.unique(d.descendants(a),|n|if calendar{n.tag=="h3"&&n.class("seriesTitle")}else{n.tag=="strong"})?;
            let title=d.text(title_node,1024)?;
            let coordinates=if calendar{
                let small=d.descendants(a).find(|i|d.nodes[*i].tag=="small").ok_or(())?;
                let label=d.raw_text(small,256)?;route::label(label.split_whitespace().next().ok_or(())?).ok_or(())?
            }else{let coord=d.unique(d.descendants(a),|n|n.tag=="span"&&n.class("blue2"))?;route::label(&d.text(coord,64)?).ok_or(())?};
            if coordinates!=(season,number.clone()){return Err(());}
            let(date,time,approximate)=if calendar{
                let section=d.nearest_class(a,"calendarList").ok_or(())?;
                let h=d.unique(d.descendants(section),|n|n.tag=="h3"&&!n.class("seriesTitle"))?;let date=route::date(&d.text(h,128)?).ok_or(())?;
                let small=d.descendants(a).filter(|i|d.nodes[*i].tag=="small").nth(1).ok_or(())?;let text=d.text(small,256)?;
                (date,Some(route::time(&text).ok_or(())?),text.contains('~'))
            }else{let n=d.unique(d.descendants(a),|n|n.class("elementFloatRight"))?;(route::date(&d.text(n,128)?).ok_or(())?,None,false)};
            let mut facts=vec![];for track in page::tracks(d,row)?{
                let mut o=observation(c,r,Some(route.key.clone()),title.clone(),Some(season),Some(number.clone()),track);
                o.source_date_text=Some(date.clone());o.source_time_text=time.clone();o.approximate=approximate;
                if o.track==ObservationTrack::UNKNOWN{o.diagnostics.push(diagnostic("UNKNOWN_LANGUAGE_TRACK"));}facts.push(o);
            }Ok(facts)
        })();
        match parsed{Ok(facts)=>for o in facts{push_unique(&mut out,&mut hashes,o)?;},Err(())=>partial=true}
    }
    Ok((out,partial))
}
fn direct(c:&ExtensionContextV1,r:&ResponseEnvelope,d:&Document,p:&RequestSpec)->Result<Vec<ProviderObservationV1>>{
    let t=c.targets.iter().find(|t|p.target_token.as_ref()==Some(&t.target_token)).ok_or(())?;
    let season=t.source_season.ok_or(())?;let number=t.installment.number.as_deref().ok_or(())?;
    let(title,available)=page::episode(d,&p.url,season,number)?;if !available.contains(&t.track){return Ok(vec![]);}
    Ok(vec![observation(c,r,Some(t.provider_series_key.clone()),title,Some(season),Some(number.into()),t.track.clone())])
}
fn postponed(c:&ExtensionContextV1,r:&ResponseEnvelope,d:&Document)->Result<(Vec<ProviderObservationV1>,bool)>{
    let article=d.unique(1..d.nodes.len(),|n|n.class("supportFAQArticle")&&n.class("supportFAQHighlight"))?;
    let h=d.unique(d.descendants(article),|n|n.tag=="h1")?;if !d.text(h,256)?.starts_with("Animeverschiebungen"){return Err(());}
    let p=d.unique(d.descendants(article),|n|n.tag=="p")?;let text=d.raw_text(p,32768)?;let mut out=vec![];let mut hashes=vec![];let mut partial=false;
    for block in text.split("----------------------------------------------------------------------"){
        let lines:Vec<_>=block.lines().map(str::trim).filter(|s|!s.is_empty()).collect();if lines.is_empty(){continue;}
        let title=lines[0].trim_start_matches(['⚠','\u{fe0f}','🚨','ℹ',' ']).trim();if title.is_empty()||title.len()>1024{continue;}
        for(i,line)in lines.iter().enumerate(){
            if !line.starts_with('•'){continue;}let coordinate=line.trim_start_matches('•').trim();let mut words=coordinate.split_whitespace();
            let s=words.next().unwrap_or("");let e=words.next().unwrap_or("");let Some((season,number))=route::label(&route::join(&[s,e]))else{partial=true;continue;};
            let Ok(line_track)=notice_track(line)else{partial=true;continue;};
            let mut seen=false;
            for date_line in lines.iter().skip(i+1).take_while(|s|!s.starts_with('•')){
                let down=date_line.contains('▼');let up=date_line.contains('▲');
                if down&&up{partial=true;continue;}
                let marker=if down{ObservationScheduleMarker::POSTPONED}else if up{ObservationScheduleMarker::RESCHEDULED}else{continue;};seen=true;
                let Ok(date_track)=notice_track(date_line)else{partial=true;continue;};
                if line_track.is_some()&&date_track.is_some()&&line_track!=date_track{partial=true;continue;}
                let track=date_track.or_else(||line_track.clone()).unwrap_or(ObservationTrack::UNKNOWN);
                let mut o=observation(c,r,None,title.into(),Some(season),Some(number.clone()),track);o.source_raw_text=Some(route::join(&[coordinate," ",date_line]));
                if o.source_raw_text.as_ref().is_some_and(|s|s.len()>2048){partial=true;continue;}
                o.schedule_marker=marker;o.correction_marker=Some("POSTPONEMENT_NOTICE_UNBOUND".into());o.diagnostics.push(diagnostic("UNBOUND_PROVIDER_IDENTITY"));push_unique(&mut out,&mut hashes,o)?;
            }if !seen{partial=true;}
        }
    }Ok((out,partial))
}
pub fn parse(bytes:&[u8])->Vec<u8>{
    let Ok(input)=decode::<ParseInputV1>(bytes,4*1024*1024)else{return error();};let Ok(plan)=release_plan(&input.context)else{return error();};
    if input.responses.iter().enumerate().any(|(i,r)|input.responses.iter().take(i).any(|v|v.request_id==r.request_id)){return error();}
    let mut observations=vec![];let mut reports=vec![];
    for r in &input.responses{
        let result=(||->Result<(Vec<ProviderObservationV1>,bool)>{
            let p=plan.iter().find(|p|p.request_id==r.request_id&&p.source_role==r.source_role).ok_or(())?;
            let d=page::page(&r.status,r.http_status,r.final_url.as_deref(),r.body_utf8.as_deref(),r.source_hash.as_deref(),&p.url)?;
            match r.source_role{SourceRole::CALENDAR|SourceRole::RECENT=>listing(&input.context,r,&d),SourceRole::POSTPONEMENT=>postponed(&input.context,r,&d),SourceRole::DIRECT=>Ok((direct(&input.context,r,&d,p)?,false))}
        })();
        let(outcome,diagnostics)=match result{
            Ok((facts,partial))=>{if observations.len()+facts.len()>512{return error();}observations.extend(facts);(if partial{ExtensionReportOutcome::PARTIAL}else{ExtensionReportOutcome::SUCCESS},if partial{vec![diagnostic("AMBIGUOUS_OR_INVALID_ROW")]}else{vec![]})},
            Err(())=>(ExtensionReportOutcome::FAILURE,vec![diagnostic("INVALID_PAGE_OR_PROVENANCE")]),
        };reports.push(ResponseReportV1{request_id:r.request_id.clone(),outcome,diagnostics});
    }
    let _=abi::diagnostic_text("de.aniworld release parse");encode(&ParseOutputV1{schema_version:1,observations,response_reports:reports},1024*1024).unwrap_or_else(|_|error())
}
