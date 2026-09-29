#![no_std]
extern crate alloc;
pub mod abi;
pub mod bounded;
use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

// Required nullable fields must be present as explicit null on the v1 wire.
fn required_nullable<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where D: serde::Deserializer<'de>, T: Deserialize<'de> { Option::<T>::deserialize(d) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum NavigationCapability { OVERVIEW_NAVIGATION, EPISODE_NAVIGATION }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum SourceRole { CALENDAR, RECENT, POSTPONEMENT, DIRECT }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum ExtensionMethod { GET }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum ExtensionResponseStatus { OK, TRANSPORT_FAILURE, BUDGET_DENIED, CANCELLED }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum ExtensionReportOutcome { SUCCESS, PARTIAL, FAILURE }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum ObservationInstallmentKind { EPISODE, FILM, SPECIAL, UNKNOWN }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum ObservationTrack { DE_SUB, DE_DUB, UNKNOWN }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum ObservationClaimKind { FORECAST, RELEASE_LISTING, CORRECTION, DIRECT_AVAILABILITY }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum ObservationScheduleMarker { NONE, POSTPONED, CANCELLED, RESCHEDULED, UNKNOWN }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum ExtensionGuestErrorCode { INVALID_INPUT, UNSUPPORTED_SCHEMA, PARSE_FAILED, UNSUPPORTED_ROLE }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum NavigationTargetKind { OVERVIEW, EPISODE }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstallmentV1 {
    pub kind: ObservationInstallmentKind,
    #[serde(deserialize_with = "required_nullable")]
    pub number: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionTargetV1 {
    pub target_token: String,
    pub provider_series_key: String,
    #[serde(deserialize_with = "required_nullable")]
    pub provider_url: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub source_season: Option<i32>,
    #[serde(deserialize_with = "required_nullable")]
    pub navigation_season: Option<i32>,
    pub installment: InstallmentV1,
    pub track: ObservationTrack,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionContextV1 {
    pub extension_id: String,
    pub provider_id: String,
    pub source_roles: Vec<SourceRole>,
    pub observed_at: String,
    pub targets: Vec<ExtensionTargetV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanInputV1 {
    pub schema_version: i32,
    pub context: ExtensionContextV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequestSpec {
    pub request_id: String,
    pub source_role: SourceRole,
    pub url: String,
    pub method: ExtensionMethod,
    #[serde(deserialize_with = "required_nullable")]
    pub target_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanOutputV1 {
    pub schema_version: i32,
    pub requests: Vec<RequestSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResponseEnvelope {
    pub request_id: String,
    pub source_role: SourceRole,
    pub status: ExtensionResponseStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub http_status: Option<i32>,
    #[serde(deserialize_with = "required_nullable")]
    pub final_url: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub body_utf8: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub source_hash: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParseInputV1 {
    pub schema_version: i32,
    pub context: ExtensionContextV1,
    pub responses: Vec<ResponseEnvelope>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObservationDiagnosticV1 {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderObservationV1 {
    pub schema_version: i32,
    pub extension_id: String,
    pub provider_id: String,
    pub request_id: String,
    pub source_role: SourceRole,
    #[serde(deserialize_with = "required_nullable")]
    pub provider_series_key: Option<String>,
    pub raw_title: String,
    #[serde(deserialize_with = "required_nullable")]
    pub source_season: Option<i32>,
    #[serde(deserialize_with = "required_nullable")]
    pub navigation_season: Option<i32>,
    pub installment: InstallmentV1,
    pub track: ObservationTrack,
    pub claim_kind: ObservationClaimKind,
    #[serde(deserialize_with = "required_nullable")]
    pub source_date_text: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub source_time_text: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub source_raw_text: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub parsed_timestamp: Option<String>,
    pub approximate: bool,
    pub schedule_marker: ObservationScheduleMarker,
    #[serde(deserialize_with = "required_nullable")]
    pub correction_marker: Option<String>,
    pub source_url: String,
    pub source_hash: String,
    pub diagnostics: Vec<ObservationDiagnosticV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResponseReportV1 {
    pub request_id: String,
    pub outcome: ExtensionReportOutcome,
    pub diagnostics: Vec<ObservationDiagnosticV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParseOutputV1 {
    pub schema_version: i32,
    pub observations: Vec<ProviderObservationV1>,
    pub response_reports: Vec<ResponseReportV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NavigationContextV1 {
    pub schema_version: i32,
    pub extension_id: String,
    pub provider_id: String,
    pub observed_at: String,
    pub target_kind: NavigationTargetKind,
    pub target_token: String,
    pub provider_series_key: String,
    #[serde(deserialize_with = "required_nullable")]
    pub provider_route_hint: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub source_season: Option<i32>,
    #[serde(deserialize_with = "required_nullable")]
    pub provider_episode: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub track: Option<ObservationTrack>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NavigationRequestSpecV1 {
    pub request_id: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NavigationPlanOutputV1 {
    pub schema_version: i32,
    pub requests: Vec<NavigationRequestSpecV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NavigationResponseEnvelopeV1 {
    pub request_id: String,
    pub status: ExtensionResponseStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub http_status: Option<i32>,
    #[serde(deserialize_with = "required_nullable")]
    pub final_url: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub body_utf8: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub source_hash: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderNavigationTargetV1 {
    pub schema_version: i32,
    pub extension_id: String,
    pub provider_id: String,
    pub target_kind: NavigationTargetKind,
    pub provider_series_key: String,
    #[serde(deserialize_with = "required_nullable")]
    pub source_season: Option<i32>,
    #[serde(deserialize_with = "required_nullable")]
    pub provider_episode: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub track: Option<ObservationTrack>,
    pub url: String,
    #[serde(deserialize_with = "required_nullable")]
    pub request_id: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub source_hash: Option<String>,
    pub diagnostics: Vec<ObservationDiagnosticV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NavigationParseOutputV1 {
    pub schema_version: i32,
    pub targets: Vec<ProviderNavigationTargetV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NavigationParseInputV1 {
    pub schema_version: i32,
    pub context: NavigationContextV1,
    pub responses: Vec<NavigationResponseEnvelopeV1>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GuestError {
    pub code: ExtensionGuestErrorCode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ErrorOutputV1 {
    pub schema_version: i32,
    pub error: GuestError,
}

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
&self.installment.validate()?;

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
&self.context.validate()?;
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
&self.context.validate()?;
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
&self.installment.validate()?;


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
if let Some(v) = &self.track {  }
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
if let Some(v) = &self.track {  }
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
&self.context.validate()?;
bounded::ensure(self.responses.len() <= 7)?;
for v in &self.responses { v.validate()?; }
Ok(()) } }
impl Validate for GuestError { fn validate(&self) -> Result<(), bounded::Error> {

Ok(()) } }
impl Validate for ErrorOutputV1 { fn validate(&self) -> Result<(), bounded::Error> {
bounded::ensure(*&self.schema_version == 1)?;
&self.error.validate()?;
Ok(()) } }
