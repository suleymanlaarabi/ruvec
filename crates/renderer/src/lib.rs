mod gpu;
mod vello;

pub use gpu::GpuRenderer;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("canvas dimensions exceed the device or Vello limits: {width} × {height}")]
    InvalidSize { width: u32, height: u32 },
    #[error("Vello rendering failed: {0}")]
    Backend(String),
}
