#![no_std]
extern crate alloc;
pub mod abi;
pub mod bounded;
pub mod wire;
use alloc::{string::String,vec::Vec};
use wire::{Wire,Value,Fields};

#[derive(Clone,Debug,PartialEq,Eq)]
#[allow(non_camel_case_types)]
pub enum NavigationCapability { OVERVIEW_NAVIGATION,EPISODE_NAVIGATION }
impl Wire for NavigationCapability { fn from_value(v:Value)->Result<Self,bounded::Error>{let s=String::from_value(v)?;match s.as_str(){"OVERVIEW_NAVIGATION"=>Ok(Self::OVERVIEW_NAVIGATION),"EPISODE_NAVIGATION"=>Ok(Self::EPISODE_NAVIGATION),_=>Err(bounded::Error::Json)}} fn to_value(&self)->Value{Value::Text(String::from(match self{Self::OVERVIEW_NAVIGATION=>"OVERVIEW_NAVIGATION",Self::EPISODE_NAVIGATION=>"EPISODE_NAVIGATION",}))} }

#[derive(Clone,Debug,PartialEq,Eq)]
#[allow(non_camel_case_types)]
pub enum SourceRole { CALENDAR,RECENT,POSTPONEMENT,DIRECT }
impl Wire for SourceRole { fn from_value(v:Value)->Result<Self,bounded::Error>{let s=String::from_value(v)?;match s.as_str(){"CALENDAR"=>Ok(Self::CALENDAR),"RECENT"=>Ok(Self::RECENT),"POSTPONEMENT"=>Ok(Self::POSTPONEMENT),"DIRECT"=>Ok(Self::DIRECT),_=>Err(bounded::Error::Json)}} fn to_value(&self)->Value{Value::Text(String::from(match self{Self::CALENDAR=>"CALENDAR",Self::RECENT=>"RECENT",Self::POSTPONEMENT=>"POSTPONEMENT",Self::DIRECT=>"DIRECT",}))} }

#[derive(Clone,Debug,PartialEq,Eq)]
#[allow(non_camel_case_types)]
pub enum ExtensionMethod { GET }
impl Wire for ExtensionMethod { fn from_value(v:Value)->Result<Self,bounded::Error>{let s=String::from_value(v)?;match s.as_str(){"GET"=>Ok(Self::GET),_=>Err(bounded::Error::Json)}} fn to_value(&self)->Value{Value::Text(String::from(match self{Self::GET=>"GET",}))} }

#[derive(Clone,Debug,PartialEq,Eq)]
#[allow(non_camel_case_types)]
pub enum ExtensionResponseStatus { OK,TRANSPORT_FAILURE,BUDGET_DENIED,CANCELLED }
impl Wire for ExtensionResponseStatus { fn from_value(v:Value)->Result<Self,bounded::Error>{let s=String::from_value(v)?;match s.as_str(){"OK"=>Ok(Self::OK),"TRANSPORT_FAILURE"=>Ok(Self::TRANSPORT_FAILURE),"BUDGET_DENIED"=>Ok(Self::BUDGET_DENIED),"CANCELLED"=>Ok(Self::CANCELLED),_=>Err(bounded::Error::Json)}} fn to_value(&self)->Value{Value::Text(String::from(match self{Self::OK=>"OK",Self::TRANSPORT_FAILURE=>"TRANSPORT_FAILURE",Self::BUDGET_DENIED=>"BUDGET_DENIED",Self::CANCELLED=>"CANCELLED",}))} }

#[derive(Clone,Debug,PartialEq,Eq)]
#[allow(non_camel_case_types)]
pub enum ExtensionReportOutcome { SUCCESS,PARTIAL,FAILURE }
impl Wire for ExtensionReportOutcome { fn from_value(v:Value)->Result<Self,bounded::Error>{let s=String::from_value(v)?;match s.as_str(){"SUCCESS"=>Ok(Self::SUCCESS),"PARTIAL"=>Ok(Self::PARTIAL),"FAILURE"=>Ok(Self::FAILURE),_=>Err(bounded::Error::Json)}} fn to_value(&self)->Value{Value::Text(String::from(match self{Self::SUCCESS=>"SUCCESS",Self::PARTIAL=>"PARTIAL",Self::FAILURE=>"FAILURE",}))} }

#[derive(Clone,Debug,PartialEq,Eq)]
#[allow(non_camel_case_types)]
pub enum ObservationInstallmentKind { EPISODE,FILM,SPECIAL,UNKNOWN }
impl Wire for ObservationInstallmentKind { fn from_value(v:Value)->Result<Self,bounded::Error>{let s=String::from_value(v)?;match s.as_str(){"EPISODE"=>Ok(Self::EPISODE),"FILM"=>Ok(Self::FILM),"SPECIAL"=>Ok(Self::SPECIAL),"UNKNOWN"=>Ok(Self::UNKNOWN),_=>Err(bounded::Error::Json)}} fn to_value(&self)->Value{Value::Text(String::from(match self{Self::EPISODE=>"EPISODE",Self::FILM=>"FILM",Self::SPECIAL=>"SPECIAL",Self::UNKNOWN=>"UNKNOWN",}))} }

#[derive(Clone,Debug,PartialEq,Eq)]
#[allow(non_camel_case_types)]
pub enum ObservationTrack { DE_SUB,DE_DUB,UNKNOWN }
impl Wire for ObservationTrack { fn from_value(v:Value)->Result<Self,bounded::Error>{let s=String::from_value(v)?;match s.as_str(){"DE_SUB"=>Ok(Self::DE_SUB),"DE_DUB"=>Ok(Self::DE_DUB),"UNKNOWN"=>Ok(Self::UNKNOWN),_=>Err(bounded::Error::Json)}} fn to_value(&self)->Value{Value::Text(String::from(match self{Self::DE_SUB=>"DE_SUB",Self::DE_DUB=>"DE_DUB",Self::UNKNOWN=>"UNKNOWN",}))} }

#[derive(Clone,Debug,PartialEq,Eq)]
#[allow(non_camel_case_types)]
pub enum ObservationClaimKind { FORECAST,RELEASE_LISTING,CORRECTION,DIRECT_AVAILABILITY }
impl Wire for ObservationClaimKind { fn from_value(v:Value)->Result<Self,bounded::Error>{let s=String::from_value(v)?;match s.as_str(){"FORECAST"=>Ok(Self::FORECAST),"RELEASE_LISTING"=>Ok(Self::RELEASE_LISTING),"CORRECTION"=>Ok(Self::CORRECTION),"DIRECT_AVAILABILITY"=>Ok(Self::DIRECT_AVAILABILITY),_=>Err(bounded::Error::Json)}} fn to_value(&self)->Value{Value::Text(String::from(match self{Self::FORECAST=>"FORECAST",Self::RELEASE_LISTING=>"RELEASE_LISTING",Self::CORRECTION=>"CORRECTION",Self::DIRECT_AVAILABILITY=>"DIRECT_AVAILABILITY",}))} }

#[derive(Clone,Debug,PartialEq,Eq)]
#[allow(non_camel_case_types)]
pub enum ObservationScheduleMarker { NONE,POSTPONED,CANCELLED,RESCHEDULED,UNKNOWN }
impl Wire for ObservationScheduleMarker { fn from_value(v:Value)->Result<Self,bounded::Error>{let s=String::from_value(v)?;match s.as_str(){"NONE"=>Ok(Self::NONE),"POSTPONED"=>Ok(Self::POSTPONED),"CANCELLED"=>Ok(Self::CANCELLED),"RESCHEDULED"=>Ok(Self::RESCHEDULED),"UNKNOWN"=>Ok(Self::UNKNOWN),_=>Err(bounded::Error::Json)}} fn to_value(&self)->Value{Value::Text(String::from(match self{Self::NONE=>"NONE",Self::POSTPONED=>"POSTPONED",Self::CANCELLED=>"CANCELLED",Self::RESCHEDULED=>"RESCHEDULED",Self::UNKNOWN=>"UNKNOWN",}))} }

#[derive(Clone,Debug,PartialEq,Eq)]
#[allow(non_camel_case_types)]
pub enum ExtensionGuestErrorCode { INVALID_INPUT,UNSUPPORTED_SCHEMA,PARSE_FAILED,UNSUPPORTED_ROLE }
impl Wire for ExtensionGuestErrorCode { fn from_value(v:Value)->Result<Self,bounded::Error>{let s=String::from_value(v)?;match s.as_str(){"INVALID_INPUT"=>Ok(Self::INVALID_INPUT),"UNSUPPORTED_SCHEMA"=>Ok(Self::UNSUPPORTED_SCHEMA),"PARSE_FAILED"=>Ok(Self::PARSE_FAILED),"UNSUPPORTED_ROLE"=>Ok(Self::UNSUPPORTED_ROLE),_=>Err(bounded::Error::Json)}} fn to_value(&self)->Value{Value::Text(String::from(match self{Self::INVALID_INPUT=>"INVALID_INPUT",Self::UNSUPPORTED_SCHEMA=>"UNSUPPORTED_SCHEMA",Self::PARSE_FAILED=>"PARSE_FAILED",Self::UNSUPPORTED_ROLE=>"UNSUPPORTED_ROLE",}))} }

#[derive(Clone,Debug,PartialEq,Eq)]
#[allow(non_camel_case_types)]
pub enum NavigationTargetKind { OVERVIEW,EPISODE }
impl Wire for NavigationTargetKind { fn from_value(v:Value)->Result<Self,bounded::Error>{let s=String::from_value(v)?;match s.as_str(){"OVERVIEW"=>Ok(Self::OVERVIEW),"EPISODE"=>Ok(Self::EPISODE),_=>Err(bounded::Error::Json)}} fn to_value(&self)->Value{Value::Text(String::from(match self{Self::OVERVIEW=>"OVERVIEW",Self::EPISODE=>"EPISODE",}))} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct InstallmentV1 {
pub kind: ObservationInstallmentKind,
pub number: Option<String>,
}
impl Wire for InstallmentV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["kind","number"])?;Ok(Self{kind:f.take("kind")?,number:f.take("number")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("kind"),self.kind.to_value()),(String::from("number"),self.number.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ExtensionTargetV1 {
pub target_token: String,
pub provider_series_key: String,
pub provider_url: Option<String>,
pub source_season: Option<i32>,
pub navigation_season: Option<i32>,
pub installment: InstallmentV1,
pub track: ObservationTrack,
}
impl Wire for ExtensionTargetV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["targetToken","providerSeriesKey","providerUrl","sourceSeason","navigationSeason","installment","track"])?;Ok(Self{target_token:f.take("targetToken")?,provider_series_key:f.take("providerSeriesKey")?,provider_url:f.take("providerUrl")?,source_season:f.take("sourceSeason")?,navigation_season:f.take("navigationSeason")?,installment:f.take("installment")?,track:f.take("track")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("targetToken"),self.target_token.to_value()),(String::from("providerSeriesKey"),self.provider_series_key.to_value()),(String::from("providerUrl"),self.provider_url.to_value()),(String::from("sourceSeason"),self.source_season.to_value()),(String::from("navigationSeason"),self.navigation_season.to_value()),(String::from("installment"),self.installment.to_value()),(String::from("track"),self.track.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ExtensionContextV1 {
pub extension_id: String,
pub provider_id: String,
pub source_roles: Vec<SourceRole>,
pub observed_at: String,
pub targets: Vec<ExtensionTargetV1>,
}
impl Wire for ExtensionContextV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["extensionId","providerId","sourceRoles","observedAt","targets"])?;Ok(Self{extension_id:f.take("extensionId")?,provider_id:f.take("providerId")?,source_roles:f.take("sourceRoles")?,observed_at:f.take("observedAt")?,targets:f.take("targets")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("extensionId"),self.extension_id.to_value()),(String::from("providerId"),self.provider_id.to_value()),(String::from("sourceRoles"),self.source_roles.to_value()),(String::from("observedAt"),self.observed_at.to_value()),(String::from("targets"),self.targets.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct PlanInputV1 {
pub schema_version: i32,
pub context: ExtensionContextV1,
}
impl Wire for PlanInputV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["schemaVersion","context"])?;Ok(Self{schema_version:f.take("schemaVersion")?,context:f.take("context")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("schemaVersion"),self.schema_version.to_value()),(String::from("context"),self.context.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RequestSpec {
pub request_id: String,
pub source_role: SourceRole,
pub url: String,
pub method: ExtensionMethod,
pub target_token: Option<String>,
}
impl Wire for RequestSpec { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["requestId","sourceRole","url","method","targetToken"])?;Ok(Self{request_id:f.take("requestId")?,source_role:f.take("sourceRole")?,url:f.take("url")?,method:f.take("method")?,target_token:f.take("targetToken")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("requestId"),self.request_id.to_value()),(String::from("sourceRole"),self.source_role.to_value()),(String::from("url"),self.url.to_value()),(String::from("method"),self.method.to_value()),(String::from("targetToken"),self.target_token.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct PlanOutputV1 {
pub schema_version: i32,
pub requests: Vec<RequestSpec>,
}
impl Wire for PlanOutputV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["schemaVersion","requests"])?;Ok(Self{schema_version:f.take("schemaVersion")?,requests:f.take("requests")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("schemaVersion"),self.schema_version.to_value()),(String::from("requests"),self.requests.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ResponseEnvelope {
pub request_id: String,
pub source_role: SourceRole,
pub status: ExtensionResponseStatus,
pub http_status: Option<i32>,
pub final_url: Option<String>,
pub body_utf8: Option<String>,
pub source_hash: Option<String>,
}
impl Wire for ResponseEnvelope { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["requestId","sourceRole","status","httpStatus","finalUrl","bodyUtf8","sourceHash"])?;Ok(Self{request_id:f.take("requestId")?,source_role:f.take("sourceRole")?,status:f.take("status")?,http_status:f.take("httpStatus")?,final_url:f.take("finalUrl")?,body_utf8:f.take("bodyUtf8")?,source_hash:f.take("sourceHash")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("requestId"),self.request_id.to_value()),(String::from("sourceRole"),self.source_role.to_value()),(String::from("status"),self.status.to_value()),(String::from("httpStatus"),self.http_status.to_value()),(String::from("finalUrl"),self.final_url.to_value()),(String::from("bodyUtf8"),self.body_utf8.to_value()),(String::from("sourceHash"),self.source_hash.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ParseInputV1 {
pub schema_version: i32,
pub context: ExtensionContextV1,
pub responses: Vec<ResponseEnvelope>,
}
impl Wire for ParseInputV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["schemaVersion","context","responses"])?;Ok(Self{schema_version:f.take("schemaVersion")?,context:f.take("context")?,responses:f.take("responses")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("schemaVersion"),self.schema_version.to_value()),(String::from("context"),self.context.to_value()),(String::from("responses"),self.responses.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ObservationDiagnosticV1 {
pub code: String,
pub message: String,
}
impl Wire for ObservationDiagnosticV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["code","message"])?;Ok(Self{code:f.take("code")?,message:f.take("message")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("code"),self.code.to_value()),(String::from("message"),self.message.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ProviderObservationV1 {
pub schema_version: i32,
pub extension_id: String,
pub provider_id: String,
pub request_id: String,
pub source_role: SourceRole,
pub provider_series_key: Option<String>,
pub raw_title: String,
pub source_season: Option<i32>,
pub navigation_season: Option<i32>,
pub installment: InstallmentV1,
pub track: ObservationTrack,
pub claim_kind: ObservationClaimKind,
pub source_date_text: Option<String>,
pub source_time_text: Option<String>,
pub source_raw_text: Option<String>,
pub parsed_timestamp: Option<String>,
pub approximate: bool,
pub schedule_marker: ObservationScheduleMarker,
pub correction_marker: Option<String>,
pub source_url: String,
pub source_hash: String,
pub diagnostics: Vec<ObservationDiagnosticV1>,
}
impl Wire for ProviderObservationV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["schemaVersion","extensionId","providerId","requestId","sourceRole","providerSeriesKey","rawTitle","sourceSeason","navigationSeason","installment","track","claimKind","sourceDateText","sourceTimeText","sourceRawText","parsedTimestamp","approximate","scheduleMarker","correctionMarker","sourceUrl","sourceHash","diagnostics"])?;Ok(Self{schema_version:f.take("schemaVersion")?,extension_id:f.take("extensionId")?,provider_id:f.take("providerId")?,request_id:f.take("requestId")?,source_role:f.take("sourceRole")?,provider_series_key:f.take("providerSeriesKey")?,raw_title:f.take("rawTitle")?,source_season:f.take("sourceSeason")?,navigation_season:f.take("navigationSeason")?,installment:f.take("installment")?,track:f.take("track")?,claim_kind:f.take("claimKind")?,source_date_text:f.take("sourceDateText")?,source_time_text:f.take("sourceTimeText")?,source_raw_text:f.take("sourceRawText")?,parsed_timestamp:f.take("parsedTimestamp")?,approximate:f.take("approximate")?,schedule_marker:f.take("scheduleMarker")?,correction_marker:f.take("correctionMarker")?,source_url:f.take("sourceUrl")?,source_hash:f.take("sourceHash")?,diagnostics:f.take("diagnostics")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("schemaVersion"),self.schema_version.to_value()),(String::from("extensionId"),self.extension_id.to_value()),(String::from("providerId"),self.provider_id.to_value()),(String::from("requestId"),self.request_id.to_value()),(String::from("sourceRole"),self.source_role.to_value()),(String::from("providerSeriesKey"),self.provider_series_key.to_value()),(String::from("rawTitle"),self.raw_title.to_value()),(String::from("sourceSeason"),self.source_season.to_value()),(String::from("navigationSeason"),self.navigation_season.to_value()),(String::from("installment"),self.installment.to_value()),(String::from("track"),self.track.to_value()),(String::from("claimKind"),self.claim_kind.to_value()),(String::from("sourceDateText"),self.source_date_text.to_value()),(String::from("sourceTimeText"),self.source_time_text.to_value()),(String::from("sourceRawText"),self.source_raw_text.to_value()),(String::from("parsedTimestamp"),self.parsed_timestamp.to_value()),(String::from("approximate"),self.approximate.to_value()),(String::from("scheduleMarker"),self.schedule_marker.to_value()),(String::from("correctionMarker"),self.correction_marker.to_value()),(String::from("sourceUrl"),self.source_url.to_value()),(String::from("sourceHash"),self.source_hash.to_value()),(String::from("diagnostics"),self.diagnostics.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ResponseReportV1 {
pub request_id: String,
pub outcome: ExtensionReportOutcome,
pub diagnostics: Vec<ObservationDiagnosticV1>,
}
impl Wire for ResponseReportV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["requestId","outcome","diagnostics"])?;Ok(Self{request_id:f.take("requestId")?,outcome:f.take("outcome")?,diagnostics:f.take("diagnostics")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("requestId"),self.request_id.to_value()),(String::from("outcome"),self.outcome.to_value()),(String::from("diagnostics"),self.diagnostics.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ParseOutputV1 {
pub schema_version: i32,
pub observations: Vec<ProviderObservationV1>,
pub response_reports: Vec<ResponseReportV1>,
}
impl Wire for ParseOutputV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["schemaVersion","observations","responseReports"])?;Ok(Self{schema_version:f.take("schemaVersion")?,observations:f.take("observations")?,response_reports:f.take("responseReports")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("schemaVersion"),self.schema_version.to_value()),(String::from("observations"),self.observations.to_value()),(String::from("responseReports"),self.response_reports.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct NavigationContextV1 {
pub schema_version: i32,
pub extension_id: String,
pub provider_id: String,
pub observed_at: String,
pub target_kind: NavigationTargetKind,
pub target_token: String,
pub provider_series_key: String,
pub provider_route_hint: Option<String>,
pub source_season: Option<i32>,
pub provider_episode: Option<String>,
pub track: Option<ObservationTrack>,
}
impl Wire for NavigationContextV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["schemaVersion","extensionId","providerId","observedAt","targetKind","targetToken","providerSeriesKey","providerRouteHint","sourceSeason","providerEpisode","track"])?;Ok(Self{schema_version:f.take("schemaVersion")?,extension_id:f.take("extensionId")?,provider_id:f.take("providerId")?,observed_at:f.take("observedAt")?,target_kind:f.take("targetKind")?,target_token:f.take("targetToken")?,provider_series_key:f.take("providerSeriesKey")?,provider_route_hint:f.take("providerRouteHint")?,source_season:f.take("sourceSeason")?,provider_episode:f.take("providerEpisode")?,track:f.take("track")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("schemaVersion"),self.schema_version.to_value()),(String::from("extensionId"),self.extension_id.to_value()),(String::from("providerId"),self.provider_id.to_value()),(String::from("observedAt"),self.observed_at.to_value()),(String::from("targetKind"),self.target_kind.to_value()),(String::from("targetToken"),self.target_token.to_value()),(String::from("providerSeriesKey"),self.provider_series_key.to_value()),(String::from("providerRouteHint"),self.provider_route_hint.to_value()),(String::from("sourceSeason"),self.source_season.to_value()),(String::from("providerEpisode"),self.provider_episode.to_value()),(String::from("track"),self.track.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct NavigationRequestSpecV1 {
pub request_id: String,
pub url: String,
}
impl Wire for NavigationRequestSpecV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["requestId","url"])?;Ok(Self{request_id:f.take("requestId")?,url:f.take("url")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("requestId"),self.request_id.to_value()),(String::from("url"),self.url.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct NavigationPlanOutputV1 {
pub schema_version: i32,
pub requests: Vec<NavigationRequestSpecV1>,
}
impl Wire for NavigationPlanOutputV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["schemaVersion","requests"])?;Ok(Self{schema_version:f.take("schemaVersion")?,requests:f.take("requests")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("schemaVersion"),self.schema_version.to_value()),(String::from("requests"),self.requests.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct NavigationResponseEnvelopeV1 {
pub request_id: String,
pub status: ExtensionResponseStatus,
pub http_status: Option<i32>,
pub final_url: Option<String>,
pub body_utf8: Option<String>,
pub source_hash: Option<String>,
}
impl Wire for NavigationResponseEnvelopeV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["requestId","status","httpStatus","finalUrl","bodyUtf8","sourceHash"])?;Ok(Self{request_id:f.take("requestId")?,status:f.take("status")?,http_status:f.take("httpStatus")?,final_url:f.take("finalUrl")?,body_utf8:f.take("bodyUtf8")?,source_hash:f.take("sourceHash")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("requestId"),self.request_id.to_value()),(String::from("status"),self.status.to_value()),(String::from("httpStatus"),self.http_status.to_value()),(String::from("finalUrl"),self.final_url.to_value()),(String::from("bodyUtf8"),self.body_utf8.to_value()),(String::from("sourceHash"),self.source_hash.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ProviderNavigationTargetV1 {
pub schema_version: i32,
pub extension_id: String,
pub provider_id: String,
pub target_kind: NavigationTargetKind,
pub provider_series_key: String,
pub source_season: Option<i32>,
pub provider_episode: Option<String>,
pub track: Option<ObservationTrack>,
pub url: String,
pub request_id: Option<String>,
pub source_hash: Option<String>,
pub diagnostics: Vec<ObservationDiagnosticV1>,
}
impl Wire for ProviderNavigationTargetV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["schemaVersion","extensionId","providerId","targetKind","providerSeriesKey","sourceSeason","providerEpisode","track","url","requestId","sourceHash","diagnostics"])?;Ok(Self{schema_version:f.take("schemaVersion")?,extension_id:f.take("extensionId")?,provider_id:f.take("providerId")?,target_kind:f.take("targetKind")?,provider_series_key:f.take("providerSeriesKey")?,source_season:f.take("sourceSeason")?,provider_episode:f.take("providerEpisode")?,track:f.take("track")?,url:f.take("url")?,request_id:f.take("requestId")?,source_hash:f.take("sourceHash")?,diagnostics:f.take("diagnostics")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("schemaVersion"),self.schema_version.to_value()),(String::from("extensionId"),self.extension_id.to_value()),(String::from("providerId"),self.provider_id.to_value()),(String::from("targetKind"),self.target_kind.to_value()),(String::from("providerSeriesKey"),self.provider_series_key.to_value()),(String::from("sourceSeason"),self.source_season.to_value()),(String::from("providerEpisode"),self.provider_episode.to_value()),(String::from("track"),self.track.to_value()),(String::from("url"),self.url.to_value()),(String::from("requestId"),self.request_id.to_value()),(String::from("sourceHash"),self.source_hash.to_value()),(String::from("diagnostics"),self.diagnostics.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct NavigationParseOutputV1 {
pub schema_version: i32,
pub targets: Vec<ProviderNavigationTargetV1>,
}
impl Wire for NavigationParseOutputV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["schemaVersion","targets"])?;Ok(Self{schema_version:f.take("schemaVersion")?,targets:f.take("targets")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("schemaVersion"),self.schema_version.to_value()),(String::from("targets"),self.targets.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct NavigationParseInputV1 {
pub schema_version: i32,
pub context: NavigationContextV1,
pub responses: Vec<NavigationResponseEnvelopeV1>,
}
impl Wire for NavigationParseInputV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["schemaVersion","context","responses"])?;Ok(Self{schema_version:f.take("schemaVersion")?,context:f.take("context")?,responses:f.take("responses")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("schemaVersion"),self.schema_version.to_value()),(String::from("context"),self.context.to_value()),(String::from("responses"),self.responses.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct GuestError {
pub code: ExtensionGuestErrorCode,
}
impl Wire for GuestError { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["code"])?;Ok(Self{code:f.take("code")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("code"),self.code.to_value()),])} }

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ErrorOutputV1 {
pub schema_version: i32,
pub error: GuestError,
}
impl Wire for ErrorOutputV1 { fn from_value(v:Value)->Result<Self,bounded::Error>{let mut f=Fields::new(v,&["schemaVersion","error"])?;Ok(Self{schema_version:f.take("schemaVersion")?,error:f.take("error")?,})} fn to_value(&self)->Value{Value::Object(alloc::vec![(String::from("schemaVersion"),self.schema_version.to_value()),(String::from("error"),self.error.to_value()),])} }

pub trait Validate { fn validate(&self) -> Result<(), bounded::Error>; }
impl Validate for InstallmentV1 { fn validate(&self) -> Result<(), bounded::Error> {

if let Some(v) = &self.number { bounded::text(v, 32)?; }
Ok(()) } }
impl Validate for ExtensionTargetV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::text(&self.target_token, 128)?;
bounded::text(&self.provider_series_key, 512)?;
if let Some(v) = &self.provider_url { bounded::text(v, 2048)?; }
if let Some(v) = &self.source_season { bounded::ensure((0..=9999).contains(v))?; }
if let Some(v) = &self.navigation_season { bounded::ensure((0..=9999).contains(v))?; }
self.installment.validate()?;

Ok(()) } }
impl Validate for ExtensionContextV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::id(&self.extension_id)?;
bounded::id(&self.provider_id)?;
bounded::ensure(self.source_roles.len() <= 4)?;
bounded::text(&self.observed_at, 64)?;
bounded::ensure(self.targets.len() <= 256)?;
for v in &self.targets { v.validate()?; }
Ok(()) } }
impl Validate for PlanInputV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::ensure(*&self.schema_version == 1)?;
self.context.validate()?;
Ok(()) } }
impl Validate for RequestSpec { fn validate(&self) -> Result<(), bounded::Error> {
bounded::text(&self.request_id, 64)?;

bounded::text(&self.url, 2048)?;

if let Some(v) = &self.target_token { bounded::text(v, 128)?; }
Ok(()) } }
impl Validate for PlanOutputV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::ensure(*&self.schema_version == 1)?;
bounded::ensure(self.requests.len() <= 7)?;
for v in &self.requests { v.validate()?; }
Ok(()) } }
impl Validate for ResponseEnvelope { fn validate(&self) -> Result<(), bounded::Error> {
bounded::text(&self.request_id, 64)?;


if let Some(v) = &self.http_status { bounded::ensure((100..=599).contains(v))?; }
if let Some(v) = &self.final_url { bounded::text(v, 2048)?; }
if let Some(v) = &self.body_utf8 { bounded::text(v, 2097152)?; }
if let Some(v) = &self.source_hash { bounded::text(v, 64)?; }
Ok(()) } }
impl Validate for ParseInputV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::ensure(*&self.schema_version == 1)?;
self.context.validate()?;
bounded::ensure(self.responses.len() <= 7)?;
for v in &self.responses { v.validate()?; }
Ok(()) } }
impl Validate for ObservationDiagnosticV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::text(&self.code, 64)?;
bounded::text(&self.message, 256)?;
Ok(()) } }
impl Validate for ProviderObservationV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::ensure(*&self.schema_version == 1)?;
bounded::id(&self.extension_id)?;
bounded::id(&self.provider_id)?;
bounded::text(&self.request_id, 64)?;

if let Some(v) = &self.provider_series_key { bounded::text(v, 512)?; }
bounded::text(&self.raw_title, 1024)?;
if let Some(v) = &self.source_season { bounded::ensure((0..=9999).contains(v))?; }
if let Some(v) = &self.navigation_season { bounded::ensure((0..=9999).contains(v))?; }
self.installment.validate()?;


if let Some(v) = &self.source_date_text { bounded::text(v, 256)?; }
if let Some(v) = &self.source_time_text { bounded::text(v, 256)?; }
if let Some(v) = &self.source_raw_text { bounded::text(v, 2048)?; }
if let Some(v) = &self.parsed_timestamp { bounded::text(v, 64)?; }


if let Some(v) = &self.correction_marker { bounded::text(v, 512)?; }
bounded::text(&self.source_url, 2048)?;
bounded::text(&self.source_hash, 64)?;
bounded::ensure(self.diagnostics.len() <= 16)?;
for v in &self.diagnostics { v.validate()?; }
Ok(()) } }
impl Validate for ResponseReportV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::text(&self.request_id, 64)?;

bounded::ensure(self.diagnostics.len() <= 16)?;
for v in &self.diagnostics { v.validate()?; }
Ok(()) } }
impl Validate for ParseOutputV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::ensure(*&self.schema_version == 1)?;
bounded::ensure(self.observations.len() <= 512)?;
for v in &self.observations { v.validate()?; }
bounded::ensure(self.response_reports.len() <= 7)?;
for v in &self.response_reports { v.validate()?; }
Ok(()) } }
impl Validate for NavigationContextV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::ensure(*&self.schema_version == 1)?;
bounded::id(&self.extension_id)?;
bounded::id(&self.provider_id)?;
bounded::text(&self.observed_at, 64)?;

bounded::text(&self.target_token, 128)?;
bounded::text(&self.provider_series_key, 512)?;
if let Some(v) = &self.provider_route_hint { bounded::text(v, 2048)?; }
if let Some(v) = &self.source_season { bounded::ensure((0..=9999).contains(v))?; }
if let Some(v) = &self.provider_episode { bounded::text(v, 32)?; }
Ok(()) } }
impl Validate for NavigationRequestSpecV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::text(&self.request_id, 64)?;
bounded::text(&self.url, 2048)?;
Ok(()) } }
impl Validate for NavigationPlanOutputV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::ensure(*&self.schema_version == 1)?;
bounded::ensure(self.requests.len() <= 7)?;
for v in &self.requests { v.validate()?; }
Ok(()) } }
impl Validate for NavigationResponseEnvelopeV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::text(&self.request_id, 64)?;

if let Some(v) = &self.http_status { bounded::ensure((100..=599).contains(v))?; }
if let Some(v) = &self.final_url { bounded::text(v, 2048)?; }
if let Some(v) = &self.body_utf8 { bounded::text(v, 2097152)?; }
if let Some(v) = &self.source_hash { bounded::text(v, 64)?; }
Ok(()) } }
impl Validate for ProviderNavigationTargetV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::ensure(*&self.schema_version == 1)?;
bounded::id(&self.extension_id)?;
bounded::id(&self.provider_id)?;

bounded::text(&self.provider_series_key, 512)?;
if let Some(v) = &self.source_season { bounded::ensure((0..=9999).contains(v))?; }
if let Some(v) = &self.provider_episode { bounded::text(v, 32)?; }
bounded::text(&self.url, 2048)?;
if let Some(v) = &self.request_id { bounded::text(v, 64)?; }
if let Some(v) = &self.source_hash { bounded::text(v, 64)?; }
bounded::ensure(self.diagnostics.len() <= 16)?;
for v in &self.diagnostics { v.validate()?; }
Ok(()) } }
impl Validate for NavigationParseOutputV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::ensure(*&self.schema_version == 1)?;
bounded::ensure(self.targets.len() <= 1)?;
for v in &self.targets { v.validate()?; }
Ok(()) } }
impl Validate for NavigationParseInputV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::ensure(*&self.schema_version == 1)?;
self.context.validate()?;
bounded::ensure(self.responses.len() <= 7)?;
for v in &self.responses { v.validate()?; }
Ok(()) } }
impl Validate for GuestError { fn validate(&self) -> Result<(), bounded::Error> {

Ok(()) } }
impl Validate for ErrorOutputV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::ensure(*&self.schema_version == 1)?;
self.error.validate()?;
Ok(()) } }
