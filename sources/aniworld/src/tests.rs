use super::*;
use alloc::{string::ToString,vec};
use arex_sdk::*;
use bounded::{decode,encode};
const RECENT:&str=include_str!("../fixtures/recent.html");
const CALENDAR:&str=include_str!("../fixtures/calendar.html");
const EPISODE:&str=include_str!("../fixtures/episode.html");
const SERIES:&str=include_str!("../fixtures/series.html");
const POSTPONEMENT:&str=include_str!("../fixtures/postponement.html");
fn context(roles:Vec<SourceRole>)->ExtensionContextV1{ExtensionContextV1{extension_id:"de.aniworld".into(),provider_id:"aniworld".into(),source_roles:roles,observed_at:"2026-09-30T12:00:00Z".into(),targets:vec![]}}
fn response(role:SourceRole,body:&str)->ResponseEnvelope{
    let(id,url)=match role{SourceRole::RECENT=>("recent",route::RECENT),SourceRole::CALENDAR=>("calendar",route::CALENDAR),SourceRole::POSTPONEMENT=>("postponement",route::POSTPONEMENT),SourceRole::DIRECT=>("direct-0","https://aniworld.to/anime/stream/fixture-series/staffel-1/episode-1")};
    ResponseEnvelope{request_id:id.into(),source_role:role,status:ExtensionResponseStatus::OK,http_status:Some(200),final_url:Some(url.into()),body_utf8:Some(body.into()),source_hash:Some("a".repeat(64))}
}
fn parse_input(c:ExtensionContextV1,responses:Vec<ResponseEnvelope>)->Vec<u8>{encode(&ParseInputV1{schema_version:1,context:c,responses},4*1024*1024).unwrap()}
fn listing(role:SourceRole,body:&str)->ParseOutputV1{decode(&parse(&parse_input(context(vec![role.clone()]),vec![response(role,body)])),1024*1024).unwrap()}
fn direct_result(body:&str)->ParseOutputV1{let mut c=context(vec![SourceRole::DIRECT]);c.targets.push(target());decode(&parse(&parse_input(c,vec![response(SourceRole::DIRECT,body)])),1024*1024).unwrap()}
fn nav(kind:NavigationTargetKind)->NavigationContextV1{let ep=kind==NavigationTargetKind::EPISODE;NavigationContextV1{schema_version:1,extension_id:"de.aniworld".into(),provider_id:"aniworld".into(),observed_at:"2026-09-30T12:00:00Z".into(),target_kind:kind,target_token:"n1".into(),provider_series_key:"fixture-series".into(),provider_route_hint:None,source_season:if ep{Some(1)}else{None},provider_episode:if ep{Some("1".into())}else{None},track:if ep{Some(ObservationTrack::DE_SUB)}else{None}}}
fn navigate(c:NavigationContextV1,body:&str)->NavigationParseOutputV1{
    let expected=navigation::navigation_url(&c).unwrap_or_else(||"https://aniworld.to/invalid".into());
    let i=NavigationParseInputV1{schema_version:1,context:c,responses:vec![NavigationResponseEnvelopeV1{request_id:"navigation".into(),status:ExtensionResponseStatus::OK,http_status:Some(200),final_url:Some(expected),body_utf8:Some(body.into()),source_hash:Some("a".repeat(64))}]};
    decode(&nav_parse(&encode(&i,4*1024*1024).unwrap()),65536).unwrap()
}
#[test]fn release_plan_all_roles_and_direct(){let mut c=context(vec![SourceRole::CALENDAR,SourceRole::RECENT,SourceRole::POSTPONEMENT,SourceRole::DIRECT]);c.targets.push(target());let out:PlanOutputV1=decode(&plan(&encode(&PlanInputV1{schema_version:1,context:c},262144).unwrap()),65536).unwrap();assert_eq!(out.requests.len(),4);assert_eq!(out.requests[3].target_token.as_deref(),Some("t1"));}
fn target()->ExtensionTargetV1{ExtensionTargetV1{target_token:"t1".into(),provider_series_key:"fixture-series".into(),provider_url:None,source_season:Some(1),navigation_season:Some(1),installment:InstallmentV1{kind:ObservationInstallmentKind::EPISODE,number:Some("1".into())},track:ObservationTrack::DE_SUB}}
#[test]fn plan_never_guesses_direct(){let mut c=context(vec![SourceRole::DIRECT]);let mut t=target();t.source_season=None;c.targets.push(t);assert!(release::release_plan(&c).unwrap().is_empty());c.targets[0]=target();c.targets[0].provider_url=Some("https://evil.test/1".into());assert!(release::release_plan(&c).unwrap().is_empty());}
#[test]fn direct_cap_and_track_dedup(){let mut c=context(vec![SourceRole::DIRECT]);for i in 1..=8{let mut t=target();t.target_token=format!("t{i}");t.installment.number=Some(i.to_string());c.targets.push(t);}assert_eq!(release::release_plan(&c).unwrap().len(),4);}
#[test]fn recent_sub_and_dub_separate(){let o=listing(SourceRole::RECENT,RECENT);assert_eq!(o.observations.len(),2);assert_eq!(o.observations[0].track,ObservationTrack::DE_SUB);assert_eq!(o.observations[1].track,ObservationTrack::DE_DUB);assert!(o.observations.iter().all(|o|o.claim_kind==ObservationClaimKind::RELEASE_LISTING&&o.provider_series_key.as_deref()==Some("fixture-series")));}
#[test]fn forecast_never_confirms_even_online(){let body=CALENDAR.replace("~ 12:00 Uhr","~ 12:00 Uhr <span title='Stream online!'>online</span>");let o=listing(SourceRole::CALENDAR,&body);assert_eq!(o.observations.len(),2);assert!(o.observations.iter().all(|o|o.claim_kind==ObservationClaimKind::FORECAST&&o.approximate&&o.parsed_timestamp.is_none()));}
#[test]fn postponements_are_unbound_facts(){let o=listing(SourceRole::POSTPONEMENT,POSTPONEMENT);assert_eq!(o.observations.len(),2);assert!(o.observations.iter().all(|o|o.provider_series_key.is_none()&&o.claim_kind==ObservationClaimKind::CORRECTION&&o.parsed_timestamp.is_none()&&o.schedule_marker==ObservationScheduleMarker::POSTPONED));assert_ne!(o.observations[0].track,o.observations[1].track);}
#[test]fn combined_postponement_track_expands_to_sub_and_dub(){let body=POSTPONEMENT.replace("(Sub)","(Sub+Dub)");let o=listing(SourceRole::POSTPONEMENT,&body);assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::SUCCESS);assert_eq!(o.observations.len(),3);assert!(o.observations.iter().any(|o|o.track==ObservationTrack::DE_SUB));assert!(o.observations.iter().filter(|o|o.track==ObservationTrack::DE_DUB).count()>=2);}
#[test]fn conflicting_notice_markers_do_not_choose_a_track_or_direction(){
    for body in [POSTPONEMENT.replace("(Sub)","(Sub) (Dub)").replace("(Dub)","(Sub) (Dub)"),POSTPONEMENT.replace("▼","▼ ▲")]{
        let o=listing(SourceRole::POSTPONEMENT,&body);assert!(o.observations.is_empty());assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::PARTIAL);
    }
}
#[test]fn multiple_wall_dates_or_times_are_ambiguous(){
    let recent=listing(SourceRole::RECENT,&RECENT.replace("30.09.2026","30.09.2026 01.10.2026"));assert!(recent.observations.is_empty());
    let calendar=listing(SourceRole::CALENDAR,&CALENDAR.replace(" Uhr"," 13:00 Uhr"));assert!(calendar.observations.is_empty());
}
#[test]fn missing_episode_metadata_is_failure_not_absence(){
    let mut c=context(vec![SourceRole::DIRECT]);c.targets.push(target());let o:ParseOutputV1=decode(&parse(&parse_input(c,vec![response(SourceRole::DIRECT,include_str!("../fixtures/missing-episode.html"))])),1048576).unwrap();
    assert!(o.observations.is_empty());assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::FAILURE);
}
#[test]fn direct_requires_actual_track_and_hoster(){let mut c=context(vec![SourceRole::DIRECT]);c.targets.push(target());let o:ParseOutputV1=decode(&parse(&parse_input(c.clone(),vec![response(SourceRole::DIRECT,EPISODE)])),1048576).unwrap();assert_eq!(o.observations.len(),1);assert_eq!(o.observations[0].claim_kind,ObservationClaimKind::DIRECT_AVAILABILITY);c.targets[0].track=ObservationTrack::DE_DUB;let o:ParseOutputV1=decode(&parse(&parse_input(c,vec![response(SourceRole::DIRECT,EPISODE)])),1048576).unwrap();assert!(o.observations.is_empty());}
#[test]fn contradictory_language_markers_fail_closed_without_poisoning_other_recent_rows(){
    let body=RECENT.replace("title=\"Episode 1 mit deutschen Untertiteln\"","title=\"Episode 1 mit englischen Untertiteln\"");
    let o=listing(SourceRole::RECENT,&body);assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::PARTIAL);assert_eq!(o.observations.len(),1);assert_eq!(o.observations[0].track,ObservationTrack::DE_DUB);
    let body=EPISODE.replace("title=\"mit Untertitel Deutsch\"","title=\"mit Untertitel Englisch\"");
    let o=direct_result(&body);assert!(o.observations.is_empty());assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::FAILURE);
}
#[test]fn repeated_foreign_data_lang_key_fails_closed(){
    let body=EPISODE.replace("<img data-lang-key=\"2\" title=\"mit Untertitel Englisch\" alt=\"English subtitles\">","<img data-lang-key=\"2\" title=\"mit Untertitel Englisch\" alt=\"English subtitles\"><img data-lang-key=\"2\" title=\"auf Englisch\" alt=\"English dub\">");
    let o=direct_result(&body);assert!(o.observations.is_empty());assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::FAILURE);
}
#[test]fn distinct_foreign_data_lang_keys_do_not_poison_german_direct_track(){
    let body=EPISODE.replace("<img data-lang-key=\"2\" title=\"mit Untertitel Englisch\" alt=\"English subtitles\">","<img data-lang-key=\"2\" title=\"mit Untertitel Englisch\" alt=\"English subtitles\"><img data-lang-key=\"4\" title=\"auf Französisch\" alt=\"French dub\">");
    let o=direct_result(&body);assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::SUCCESS);assert_eq!(o.observations.len(),1);assert_eq!(o.observations[0].track,ObservationTrack::DE_SUB);
}
#[test]fn japanese_german_subtitle_asset_is_not_a_conflicting_foreign_track(){
    let body=EPISODE.replace("<img data-lang-key=\"3\" title=\"mit Untertitel Deutsch\" alt=\"German subtitles\">","<img src=\"/public/img/japanese-german.svg\" data-lang-key=\"3\" title=\"mit Untertitel Deutsch\" alt=\"Ger-Sub, Deutscher Untertitel, Flagge, Sprache\">");
    let o=direct_result(&body);assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::SUCCESS);assert_eq!(o.observations.len(),1);assert_eq!(o.observations[0].track,ObservationTrack::DE_SUB);
}
#[test]fn split_cour_direct_uses_navigation_season_but_preserves_source_season(){let mut c=context(vec![SourceRole::DIRECT]);let mut t=target();t.source_season=Some(1);t.navigation_season=Some(2);t.installment.number=Some("15".into());c.targets.push(t);let plan=release::release_plan(&c).unwrap();assert_eq!(plan[0].url,"https://aniworld.to/anime/stream/fixture-series/staffel-2/episode-15");let body=EPISODE.replace("staffel-1/episode-1","staffel-2/episode-15").replace("data-season=\"1\"","data-season=\"2\"").replace("data-episode=\"1\"","data-episode=\"15\"");let mut r=response(SourceRole::DIRECT,&body);r.final_url=Some(plan[0].url.clone());let o:ParseOutputV1=decode(&parse(&parse_input(c,vec![r])),1048576).unwrap();assert_eq!(o.observations.len(),1);assert_eq!(o.observations[0].source_season,Some(1));assert_eq!(o.observations[0].navigation_season,Some(2));}
#[test]fn empty_truncated_irrelevant_antibot_fail_closed(){for body in ["","<html><body><h1>Neue Episoden</h1>","<html><body><h1>Shop</h1></body></html>","<html><body><h1>Checking your browser</h1></body></html>"]{let o=listing(SourceRole::RECENT,body);assert!(o.observations.is_empty());assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::FAILURE);}}
#[test]fn malformed_dates_do_not_release(){for date in ["31.02.2026","99.09.2026","30.13.2026","30.09.XXXX"]{let o=listing(SourceRole::RECENT,&RECENT.replace("30.09.2026",date));assert!(o.observations.is_empty());assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::PARTIAL);}}
#[test]fn malformed_coordinates_do_not_release(){for marker in ["S01 E02","S01 E1-3","S? E01","S01 E01.5"]{let o=listing(SourceRole::RECENT,&RECENT.replace("S01 E01",marker));assert!(o.observations.is_empty());}}
#[test]fn unknown_tracks_fail_closed_without_release_authority(){let body=RECENT.replace("Episode 1 mit deutschen Untertiteln","unknown").replace("Deutsche Untertitel Flagge, German Subtitle Flag","unknown").replace("Episode 1 auf Deutsch","unknown").replace("Deutsche Flagge, German Flag","unknown");let o=listing(SourceRole::RECENT,&body);assert!(o.observations.is_empty());assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::PARTIAL);}
#[test]fn known_foreign_tracks_are_ignored_without_poisoning_german_rows(){let body=RECENT.replace("Episode 1 mit deutschen Untertiteln","Episode 1 auf Englisch").replace("Deutsche Untertitel Flagge, German Subtitle Flag","Englische Flagge, English Flag");let o=listing(SourceRole::RECENT,&body);assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::SUCCESS);assert_eq!(o.observations.len(),1);assert_eq!(o.observations[0].track,ObservationTrack::DE_DUB);}
#[test]fn duplicate_entries_collapse(){let o=listing(SourceRole::RECENT,include_str!("../fixtures/duplicate-row.html"));assert_eq!(o.observations,listing(SourceRole::RECENT,RECENT).observations);}
#[test]fn dedup_preserves_same_coordinates_with_distinct_wall_dates(){
    let duplicate=include_str!("../fixtures/duplicate-row.html");let split=duplicate.replacen("30.09.2026","01.10.2026",1);
    let o=listing(SourceRole::RECENT,&split);assert_eq!(o.observations.len(),3);
    assert_eq!(o.observations.iter().filter(|v|v.track==ObservationTrack::DE_SUB).count(),2);
}
#[test]fn response_provenance_and_transport_fail_closed(){let c=context(vec![SourceRole::RECENT]);for variant in 0..4{let mut r=response(SourceRole::RECENT,RECENT);match variant{0=>r.final_url=Some(route::CALENDAR.into()),1=>r.source_hash=Some("X".repeat(64)),2=>r.http_status=Some(403),_=>r.status=ExtensionResponseStatus::CANCELLED};let o:ParseOutputV1=decode(&parse(&parse_input(c.clone(),vec![r])),1048576).unwrap();assert!(o.observations.is_empty());assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::FAILURE);}}
#[test]fn overview_exact_and_bound(){let o=navigate(nav(NavigationTargetKind::OVERVIEW),SERIES);assert_eq!(o.targets.len(),1);assert_eq!(o.targets[0].url,"https://aniworld.to/anime/stream/fixture-series");assert_eq!(o.targets[0].request_id.as_deref(),Some("navigation"));}
#[test]fn episode_exact_no_overview_fallback(){let c=nav(NavigationTargetKind::EPISODE);let o=navigate(c.clone(),EPISODE);assert_eq!(o.targets.len(),1);assert_eq!(o.targets[0].provider_episode.as_deref(),Some("1"));assert!(navigate(c,SERIES).targets.is_empty());}
#[test]fn split_cour_uses_supplied_provider_coordinates(){let mut c=nav(NavigationTargetKind::EPISODE);c.source_season=Some(2);c.provider_episode=Some("15".into());let body=EPISODE.replace("staffel-1/episode-1","staffel-2/episode-15").replace("data-season=\"1\"","data-season=\"2\"").replace("data-episode=\"1\"","data-episode=\"15\"");let o=navigate(c,&body);assert_eq!(o.targets.len(),1);assert_eq!(o.targets[0].source_season,Some(2));assert_eq!(o.targets[0].provider_episode.as_deref(),Some("15"));}
#[test]fn missing_episode_or_dub_never_returns_target(){let mut c=nav(NavigationTargetKind::EPISODE);c.track=Some(ObservationTrack::DE_DUB);assert!(navigate(c,EPISODE).targets.is_empty());let body=EPISODE.replace("watchEpisode","unavailable");assert!(navigate(nav(NavigationTargetKind::EPISODE),&body).targets.is_empty());}
#[test]fn malformed_routes_plan_nothing(){for hint in ["https://evil.test/a","/anime/stream/other","/anime/stream/fixture-series/staffel-2","/anime/stream/fixture-series/staffel-1/episode-2"]{let mut c=nav(NavigationTargetKind::EPISODE);c.provider_route_hint=Some(hint.into());let o:NavigationPlanOutputV1=decode(&nav_plan(&encode(&c,65536).unwrap()),65536).unwrap();assert!(o.requests.is_empty());}}
#[test]fn canonical_or_final_redirect_mismatch_fails(){let body=EPISODE.replace("rel=\"canonical\"","rel=\"other\"");assert!(navigate(nav(NavigationTargetKind::EPISODE),&body).targets.is_empty());let body=EPISODE.replace("https://aniworld.to/anime/stream/fixture-series/staffel-1/episode-1","https://aniworld.to/anime/stream/other/staffel-1/episode-1");assert!(navigate(nav(NavigationTargetKind::EPISODE),&body).targets.is_empty());}
#[test]fn hostile_bytes_do_not_panic(){for body in ["<a href='unterminated>","<!-- unterminated","<main><div></main>","<img data-lang-key='3' data-lang-key='1'>","<script><h1>fake</h1>","😀<"]{let o=listing(SourceRole::RECENT,body);assert!(o.observations.is_empty());}}
#[test]fn oversized_logical_list_fails_whole_response(){let row="<div class='col-md-12'><a href='/anime/stream/fixture-series/staffel-1/episode-1'><strong>A</strong><span class='blue2'>S01 E01</span><span class='elementFloatRight'>30.09.2026</span></a><img class='flag' title='auf Deutsch'></div>";let body=format!("<h1>Neue Episoden</h1><div class='newEpisodeList'>{}</div>",row.repeat(513));let o=listing(SourceRole::RECENT,&body);assert!(o.observations.is_empty());assert_eq!(o.response_reports[0].outcome,ExtensionReportOutcome::FAILURE);}
#[test]fn shuffled_row_fields_and_flags_keep_semantics(){let body=RECENT.replace("class=\"flag\" title=", "title=").replace("alt=\"Deutsche Flagge, German Flag\"", "class=\"flag\" alt=\"Deutsche Flagge, German Flag\"").replace("alt=\"Deutsche Untertitel Flagge, German Subtitle Flag\"", "class=\"flag\" alt=\"Deutsche Untertitel Flagge, German Subtitle Flag\"");assert_eq!(listing(SourceRole::RECENT,&body).observations,listing(SourceRole::RECENT,RECENT).observations);}
#[test]fn invalid_json_schema_identity_rejected(){for b in [b"{}".as_slice(),b"\xff",b"{\"schemaVersion\":2}",b"{\"schemaVersion\":1,\"schemaVersion\":1}"]{assert_eq!(plan(b),error());assert_eq!(parse(b),error());assert_eq!(nav_plan(b),error());assert_eq!(nav_parse(b),error());}let mut c=context(vec![SourceRole::RECENT]);c.provider_id="other".into();assert!(release::release_plan(&c).is_err());}

#[test]fn calendar_series_root_preserves_forecast_coordinates_without_navigation_guess(){
    let o=listing(SourceRole::CALENDAR,CALENDAR);
    assert_eq!(o.observations.len(),2);
    assert!(o.observations.iter().all(|v|v.claim_kind==ObservationClaimKind::FORECAST&&v.source_season==Some(1)&&v.navigation_season.is_none()&&v.installment.number.as_deref()==Some("1")));
    let explicit=CALENDAR.replace("href=\"/anime/stream/fixture-series\"","href=\"/anime/stream/fixture-series/staffel-2/episode-1\"");
    assert!(listing(SourceRole::CALENDAR,&explicit).observations.iter().all(|v|v.source_season==Some(1)&&v.navigation_season==Some(2)));
    let mismatch=CALENDAR.replace("href=\"/anime/stream/fixture-series\"","href=\"/anime/stream/fixture-series/staffel-2/episode-2\"");
    assert!(listing(SourceRole::CALENDAR,&mismatch).observations.is_empty());
}
