use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub use web_time as time;
    } else {
        pub use std::time;
    }
}

pub mod app;
mod wgpu_context;
mod ui;
mod color;