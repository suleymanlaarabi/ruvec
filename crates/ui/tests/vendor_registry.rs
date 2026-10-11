use gpui::{DevicePixels, Size};

// Run the fork's own lifecycle tests without resolving Zed's full application
// workspace. This includes the exact source compiled into our GPUI dependency.
#[allow(dead_code)]
#[path = "../../../vendor/gpui/crates/gpui/src/external_compositor.rs"]
mod external_compositor;
