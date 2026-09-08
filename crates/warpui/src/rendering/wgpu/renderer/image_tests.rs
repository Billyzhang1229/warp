use std::sync::mpsc;
use std::time::Duration;

use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::vector::vec2f;
use wgpu::util::{DeviceExt, TextureDataOrder};

use super::{ColorModifier, ImageInstanceData, Pipeline, shader_types};
use crate::rendering::CornerRadius;
use crate::rendering::background_image::BackgroundImageEffects;

#[test]
fn image_and_dither_shader_validate_together() {
    let source = format!(
        "{}\n{}\n{}",
        include_str!("../shaders/image_shader.wgsl"),
        include_str!("../shaders/dither.wgsl"),
        include_str!("../shaders/background_image.wgsl")
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

// Exercise the actual image pipeline, instance layout, texture sampling and fragment output.
// Run explicitly on a machine with a GPU; the default suite still validates WGSL everywhere.
#[test]
#[ignore = "requires a GPU adapter"]
fn background_image_gpu_output() {
    let gpu = futures::executor::block_on(ImageTestGpu::new());
    let pixels: Vec<u8> = (0..64 * 64)
        .flat_map(|index| {
            let x = index % 64;
            let y = index / 64;
            let color = match y / 16 {
                0 => [32, 128, 224],
                1 => [48, 48, 48],
                2 => [128, 128, 128],
                _ => [208, 208, 208],
            };
            [color[0], color[1], color[2], [0, 64, 128, 255][x / 16]]
        })
        .collect();
    let bounds = RectF::new(vec2f(0., 0.), vec2f(64., 64.));
    let plain = gpu.render(&pixels, bounds, [[0.; 4]; 2], [0.; 4], [64, 64]);
    let neutral = gpu.render(
        &pixels,
        bounds,
        [[1., 1., 0., 0.5], [0., 1., 0., 0.]],
        [0.; 4],
        [64, 64],
    );
    assert_eq!(
        plain, neutral,
        "neutral adjustments must match the original image path exactly"
    );
    for (actual, source) in plain.chunks_exact(4).zip(pixels.chunks_exact(4)) {
        assert_eq!(
            actual, source,
            "identity sampling must preserve color and alpha"
        );
    }
    let color_effects = BackgroundImageEffects {
        brightness: 120,
        contrast: 150,
        ..Default::default()
    }
    .parameters();
    let adjusted = gpu.render(&pixels, bounds, color_effects, [0., 0., 8., 1.], [64, 64]);
    let adjusted_off = gpu.render(&pixels, bounds, color_effects, [0.; 4], [64, 64]);
    assert_eq!(
        adjusted, adjusted_off,
        "zero dither keeps other adjustments active"
    );
    for (actual, source) in adjusted.chunks_exact(4).zip(pixels.chunks_exact(4)) {
        for channel in 0..3 {
            let expected = (((f32::from(source[channel]) / 255. * 1.2 - 0.5) * 1.5 + 0.5)
                .clamp(0., 1.)
                * 255.)
                .round() as u8;
            assert!(actual[channel].abs_diff(expected) <= 1);
        }
        assert_eq!(actual[3], source[3]);
        if source[0] == source[1] && source[1] == source[2] {
            assert_eq!(actual[0], actual[1]);
            assert_eq!(actual[1], actual[2], "gray must not acquire a tint");
        }
    }
    let solid = [160, 160, 160, 128].repeat(64 * 64);
    let gradient = BackgroundImageEffects {
        gradient_strength: 80,
        ..Default::default()
    }
    .parameters();
    let vignette = BackgroundImageEffects {
        vignette_strength: 80,
        ..Default::default()
    }
    .parameters();
    // Cover bounds deliberately extend outside the viewport, as with a cropped portrait.
    let portrait = RectF::new(vec2f(0., -64.), vec2f(64., 192.));
    let grad = gpu.render(&solid, portrait, gradient, [0.; 4], [64, 64]);
    let vig = gpu.render(&solid, portrait, vignette, [0.; 4], [64, 64]);
    for y in 1..63 {
        let value = pixel(&grad, 64, 32, y);
        assert_eq!(value[3], 128);
        if y < 32 {
            assert_eq!(value[0], 160, "top half must be unchanged despite crop");
        }
        assert!(value[0] <= pixel(&grad, 64, 32, y - 1)[0]);
    }
    assert!(pixel(&grad, 64, 32, 62)[0] < 36);
    assert_eq!(pixel(&vig, 64, 32, 32), [160, 160, 160, 128]);
    assert!(pixel(&vig, 64, 1, 1)[0] < 40);
    let wide = gpu.render(
        &solid,
        RectF::new(vec2f(-64., 0.), vec2f(256., 64.)),
        vignette,
        [0.; 4],
        [128, 64],
    );
    assert_eq!(pixel(&wide, 128, 64, 32), [160, 160, 160, 128]);
    assert!(pixel(&wide, 128, 1, 1)[0] < 40);
    let no_gradient = gpu.render(
        &pixels,
        bounds,
        BackgroundImageEffects {
            gradient_strength: 100,
            gradient_start: 100,
            ..Default::default()
        }
        .parameters(),
        [0.; 4],
        [64, 64],
    );
    assert_eq!(plain, no_gradient);
    let static_a = gpu.render(&pixels, bounds, color_effects, [0., 0.65, 8., 0.], [64, 64]);
    let static_b = gpu.render(
        &pixels,
        bounds,
        color_effects,
        [30., 0.65, 8., 0.],
        [64, 64],
    );
    assert_eq!(
        static_a, static_b,
        "paused dither is independent of elapsed time"
    );
    assert_ne!(static_a, adjusted);
    let animated = gpu.render(
        &pixels,
        bounds,
        color_effects,
        [30., 0.65, 8., 1.],
        [64, 64],
    );
    assert_ne!(static_a, animated);
    for (actual, source) in animated.chunks_exact(4).zip(pixels.chunks_exact(4)) {
        assert_eq!(actual[3], source[3]);
    }
}

fn pixel(data: &[u8], width: usize, x: usize, y: usize) -> &[u8] {
    &data[(y * width + x) * 4..(y * width + x + 1) * 4]
}

struct ImageTestGpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: Pipeline,
    uniform_layout: wgpu::BindGroupLayout,
    vertices: wgpu::Buffer,
}

impl ImageTestGpu {
    async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = instance
            .request_adapter(&Default::default())
            .await
            .expect("GPU adapter required");
        eprintln!("Image shader output tests: {:?}", adapter.get_info());
        let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();
        let uniform_layout =
            crate::rendering::wgpu::resources::uniforms::create_bind_group_layout(&device);
        let pipeline = Pipeline::new(
            &uniform_layout,
            &device,
            &queue,
            wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            },
        );
        let vertices =
            [[0., 0.], [1., 0.], [0., 1.], [0., 1.], [1., 0.], [1., 1.]].map(|[x, y]| {
                shader_types::Vertex {
                    position: shader_types::vec2f(x, y),
                }
            });
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Self {
            device,
            queue,
            pipeline,
            uniform_layout,
            vertices,
        }
    }

    fn render(
        &self,
        pixels: &[u8],
        bounds: RectF,
        effects: [[f32; 4]; 2],
        dither: [f32; 4],
        viewport: [u32; 2],
    ) -> Vec<u8> {
        let uniforms = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[viewport[0] as f32, viewport[1] as f32, 0., 0.]),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let uniform_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });
        let texture = self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 64,
                    height: 64,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            TextureDataOrder::LayerMajor,
            pixels,
        );
        let texture_view = texture.create_view(&Default::default());
        let texture_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.pipeline.sampler),
                },
            ],
        });
        let instance = ImageInstanceData::new(
            bounds,
            ColorModifier::Image { opacity: 255 },
            CornerRadius::default(),
            dither,
            effects,
        );
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&instance),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let size = wgpu::Extent3d {
            width: viewport[0],
            height: viewport[1],
            depth_or_array_layers: 1,
        };
        let output = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let output_view = output.create_view(&Default::default());
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: u64::from(viewport[0] * viewport[1] * 4),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Background image output test"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline.render_pipeline);
            pass.set_bind_group(0, &uniform_group, &[]);
            pass.set_bind_group(1, &texture_group, &[]);
            pass.set_bind_group(2, &self.pipeline.noise_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.set_vertex_buffer(1, instance_buffer.slice(..));
            pass.draw(0..6, 0..1);
        }
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &output,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(viewport[0] * 4),
                    rows_per_image: None,
                },
            },
            size,
        );
        self.queue.submit([encoder.finish()]);
        let (tx, rx) = mpsc::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(30)),
            })
            .unwrap();
        rx.recv_timeout(Duration::from_secs(30)).unwrap().unwrap();
        let result = buffer.slice(..).get_mapped_range().unwrap().to_vec();
        buffer.unmap();
        result
    }
}
