#![cfg_attr(target_arch="wasm32",no_std)]
extern crate alloc;
use alloc::{string::String, vec, vec::Vec};
use arex_sdk::*;
use bounded::{decode,encode};
fn error()->Vec<u8> { br#"{"schemaVersion":1,"error":{"code":"INVALID_INPUT"}}"#.to_vec() }
fn identity(e:&str,p:&str)->bool { e=="fixture.release" && p=="fixture" }
pub fn plan(bytes:&[u8])->Vec<u8> {
    let Ok(input)=decode::<PlanInputV1>(bytes,256*1024) else { return error(); };
    if !identity(&input.context.extension_id,&input.context.provider_id) || input.context.source_roles!=[SourceRole::CALENDAR] { return error(); }
    let _=abi::diagnostic_text("fixture plan");
    encode(&PlanOutputV1 { schema_version:1, requests:vec![RequestSpec { request_id:String::from("calendar-1"),source_role:SourceRole::CALENDAR,url:String::from("https://example.org/calendar"),method:ExtensionMethod::GET,target_token:None }] },65536).unwrap_or_else(|_|error())
}
pub fn parse(bytes:&[u8])->Vec<u8> {
    let Ok(input)=decode::<ParseInputV1>(bytes,4*1024*1024) else { return error(); };
    if !identity(&input.context.extension_id,&input.context.provider_id) || input.responses.len()!=1 { return error(); }
    let r=&input.responses[0];
    if r.request_id!="calendar-1" || r.source_role!=SourceRole::CALENDAR { return error(); }
    let ok=r.status==ExtensionResponseStatus::OK && r.body_utf8.as_deref()==Some("fixture-release-v1") && r.http_status.map(|s|(200..300).contains(&s)).unwrap_or(false) && r.final_url.as_deref()==Some("https://example.org/calendar") && r.source_hash.as_ref().map(|s|s.len()==64).unwrap_or(false);
    let observations=if ok {vec![ProviderObservationV1 { schema_version:1,extension_id:input.context.extension_id.clone(),provider_id:input.context.provider_id.clone(),request_id:r.request_id.clone(),source_role:r.source_role.clone(),provider_series_key:Some(String::from("series-1")),raw_title:String::from("Fixture"),source_season:Some(2),navigation_season:Some(2),installment:InstallmentV1 {kind:ObservationInstallmentKind::EPISODE,number:Some(String::from("1"))},track:ObservationTrack::DE_SUB,claim_kind:ObservationClaimKind::FORECAST,source_date_text:None,source_time_text:None,source_raw_text:Some(String::from("fixture-release-v1")),parsed_timestamp:None,approximate:false,schedule_marker:ObservationScheduleMarker::NONE,correction_marker:None,source_url:r.final_url.clone().unwrap(),source_hash:r.source_hash.clone().unwrap(),diagnostics:vec![] }]}else{vec![]};
    let _=abi::diagnostic_text("fixture parse");
    encode(&ParseOutputV1 {schema_version:1,observations,response_reports:vec![ResponseReportV1 {request_id:r.request_id.clone(),outcome:if ok {ExtensionReportOutcome::SUCCESS}else{ExtensionReportOutcome::FAILURE},diagnostics:vec![]}]},1024*1024).unwrap_or_else(|_|error())
}
fn coordinates(c:&NavigationContextV1)->bool {
    identity(&c.extension_id,&c.provider_id) && c.provider_series_key=="series-1" && c.source_season==Some(2) &&
    match c.target_kind { NavigationTargetKind::OVERVIEW=>c.provider_episode.is_none() && c.track.is_none(), NavigationTargetKind::EPISODE=>c.provider_episode.as_deref()==Some("15") && c.track==Some(ObservationTrack::DE_SUB) }
}
pub fn nav_plan(bytes:&[u8])->Vec<u8> {
    let Ok(c)=decode::<NavigationContextV1>(bytes,65536) else{return error();};if !coordinates(&c){return error();}
    encode(&NavigationPlanOutputV1 {schema_version:1,requests:vec![NavigationRequestSpecV1{request_id:String::from("nav-1"),url:String::from("https://example.org/nav-source")}]},65536).unwrap_or_else(|_|error())
}
pub fn nav_parse(bytes:&[u8])->Vec<u8> {
    let Ok(input)=decode::<NavigationParseInputV1>(bytes,4*1024*1024) else{return error();};
    let c=input.context;if !coordinates(&c) || input.responses.len()!=1{return error();}let r=&input.responses[0];
    if r.request_id!="nav-1" || r.status!=ExtensionResponseStatus::OK || r.body_utf8.as_deref()!=Some("fixture-nav-v1") || r.final_url.as_deref()!=Some("https://example.org/nav-source") || r.http_status!=Some(200) || r.source_hash.as_ref().map(|s|s.len()!=64).unwrap_or(true){return error();}
    let url=match c.target_kind{NavigationTargetKind::OVERVIEW=>"https://example.org/series/1",NavigationTargetKind::EPISODE=>"https://example.org/series/1/episode/15"};
    let _=abi::diagnostic_text("fixture navigation");
    encode(&NavigationParseOutputV1{schema_version:1,targets:vec![ProviderNavigationTargetV1{schema_version:1,extension_id:c.extension_id,provider_id:c.provider_id,target_kind:c.target_kind,provider_series_key:c.provider_series_key,source_season:c.source_season,provider_episode:c.provider_episode,track:c.track,url:String::from(url),request_id:Some(r.request_id.clone()),source_hash:r.source_hash.clone(),diagnostics:vec![]}]},65536).unwrap_or_else(|_|error())
}
#[cfg(target_arch="wasm32")]
mod guest {
    use core::alloc::{GlobalAlloc,Layout};
    use core::sync::atomic::{AtomicUsize,Ordering};
    struct Arena;
    #[repr(align(16))] struct Heap([u8;8*1024*1024]);
    static mut HEAP:Heap=Heap([0;8*1024*1024]);
    static USED:AtomicUsize=AtomicUsize::new(0);
    unsafe impl GlobalAlloc for Arena {
        unsafe fn alloc(&self,l:Layout)->*mut u8 {
            let mut old=USED.load(Ordering::Relaxed);
            let base=core::ptr::addr_of_mut!(HEAP.0).cast::<u8>() as usize;
            loop {let Some(address)=base.checked_add(old).and_then(|v|v.checked_add(l.align()-1)) else{return core::ptr::null_mut();};let aligned=(address&!(l.align()-1))-base;let Some(end)=aligned.checked_add(l.size()) else{return core::ptr::null_mut();};if end>8*1024*1024{return core::ptr::null_mut();}
                match USED.compare_exchange_weak(old,end,Ordering::Relaxed,Ordering::Relaxed){Ok(_)=>return unsafe{core::ptr::addr_of_mut!(HEAP.0).cast::<u8>().add(aligned)},Err(v)=>old=v}
            }
        }
        unsafe fn dealloc(&self,_p:*mut u8,_l:Layout) {}
    }
    // Each host operation uses a fresh instance; the bounded arena dies with it.
    #[global_allocator] static ALLOC:Arena=Arena;
    #[panic_handler] fn panic(_: &core::panic::PanicInfo)->!{core::arch::wasm32::unreachable()}
    arex_sdk::export_v1!(super::plan,super::parse,super::nav_plan,super::nav_parse);
}
#[cfg(test)]
mod tests {
    #[test] fn rejects_wrong_schema() { assert_eq!(super::plan(b"{}"),super::error()); }
    #[test] fn transport_failure_is_not_success() {
        let input=br#"{"schemaVersion":1,"context":{"extensionId":"fixture.release","providerId":"fixture","sourceRoles":["CALENDAR"],"observedAt":"2026-09-29T12:00:00Z","targets":[]},"responses":[{"requestId":"calendar-1","sourceRole":"CALENDAR","status":"TRANSPORT_FAILURE","httpStatus":null,"finalUrl":null,"bodyUtf8":null,"sourceHash":null}]}"#;
        let output=super::parse(input);let parsed:arex_sdk::ParseOutputV1=arex_sdk::bounded::decode(&output,4096).unwrap();
        assert!(parsed.observations.is_empty());assert_eq!(parsed.response_reports[0].outcome,arex_sdk::ExtensionReportOutcome::FAILURE);
    }
}
