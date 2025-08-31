mod build;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rusty_rays_core;

#[napi]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
