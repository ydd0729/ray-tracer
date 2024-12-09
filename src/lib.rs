use cfg_if::cfg_if;
use lazy_static::lazy_static;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub(crate) use web_time as time;
    } else {
        pub(crate) use std::time;
    }
}

pub mod app;
mod math;
pub mod render;

lazy_static! {
    static ref FONT: &'static [u8] = include_bytes!("../asset/font/SourceSans3-Medium.otf");
    static ref FONT_NAME: &'static str = "SourceSans3-Medium";
    static ref RAY_TRACING_SHADER: &'static str = include_str!("shader/ray_tracing.wgsl");
}
