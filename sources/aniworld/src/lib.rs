#![cfg_attr(target_arch="wasm32", no_std)]
extern crate alloc;
mod html;
mod route;
mod release;
mod navigation;
mod page;
pub use release::{plan,parse};
pub use navigation::{nav_plan,nav_parse};
use alloc::vec::Vec;
fn error()->Vec<u8>{br#"{"schemaVersion":1,"error":{"code":"INVALID_INPUT"}}"#.to_vec()}
fn identity(e:&str,p:&str)->bool{e=="de.aniworld" && p=="aniworld"}
type Result<T> = core::result::Result<T, ()>;
#[cfg(target_arch="wasm32")] mod guest;
#[cfg(test)] mod tests;
