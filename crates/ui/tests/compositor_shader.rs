#[test]
fn external_shader_validates_against_the_current_gpui_transport() {
    let source = concat!(
        include_str!("../../../vendor/gpui/crates/gpui_wgpu/src/shaders.wgsl"),
        include_str!("../../../vendor/gpui/crates/gpui_wgpu/src/shaders_storage.wgsl"),
        include_str!("../../../vendor/gpui/crates/gpui_wgpu/src/external_compositor.wgsl"),
    );
    let module = naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(source)));
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .unwrap_or_else(|error| panic!("{}", error.emit_to_string(source)));
}
