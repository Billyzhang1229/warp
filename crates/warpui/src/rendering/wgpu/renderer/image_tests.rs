#[test]
fn image_and_dither_shader_validate_together() {
    let source = format!(
        "{}\n{}",
        include_str!("../shaders/image_shader.wgsl"),
        include_str!("../shaders/dither.wgsl")
    );
    let module = wgpu::naga::front::wgsl::parse_str(&source)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
    wgpu::naga::valid::Validator::new(
        wgpu::naga::valid::ValidationFlags::all(),
        wgpu::naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .unwrap();
}
