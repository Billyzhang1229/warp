pub mod fonts;
pub mod platform;
pub mod rendering;

/// Whether this build can render WGSL background effects.
pub const SUPPORTS_BACKGROUND_SHADERS: bool = cfg!(wgpu);
pub mod windowing;

// Re-export everything from the core crate.
pub use warpui_core::*;
