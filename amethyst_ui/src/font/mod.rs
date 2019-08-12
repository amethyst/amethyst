pub mod default;

#[cfg(not(target_arch = "wasm32"))]
pub mod systemfont;
