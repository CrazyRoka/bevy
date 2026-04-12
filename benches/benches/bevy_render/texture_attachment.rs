use bevy_render::render_resource::{Extent3d, StoreOp, TextureFormat, TextureUsages};
use bevy_render::renderer::RenderDevice;
use bevy_render::texture::{
    CachedTexture, ColorAttachment, DepthAttachment, OutputColorAttachment,
};
use bevy_tasks::block_on;
use core::hint::black_box;
use criterion::{criterion_group, Criterion};
use wgpu::{
    Color, DeviceDescriptor, Instance, RequestAdapterOptions, TextureDescriptor, TextureDimension,
    TextureViewDescriptor,
};

fn texture_attachment(c: &mut Criterion) {
    // Setup WGPU device to produce valid texture views for benchmarks
    let instance = Instance::default();
    let Ok(adapter) = block_on(instance.request_adapter(&RequestAdapterOptions::default())) else {
        eprintln!("No WGPU adapter found, skipping texture_attachment benchmark");
        return;
    };

    let (device, _) = block_on(adapter.request_device(&DeviceDescriptor::default())).unwrap();
    let render_device = RenderDevice::from(device);

    // Create a dummy 1x1 texture and view
    let texture = render_device.create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&TextureViewDescriptor::default());

    let cached_texture = CachedTexture {
        texture: texture.clone(),
        default_view: view.clone(),
    };

    // Initialize the target attachment structures
    let color_attachment = ColorAttachment::new(cached_texture.clone(), None, None, None);
    let depth_attachment = DepthAttachment::new(view.clone(), None);
    let output_attachment = OutputColorAttachment::new(view.clone(), TextureFormat::Rgba8Unorm);

    let mut group = c.benchmark_group("texture_attachment");
    group.bench_function("color_attachment_get", |b| {
        b.iter(|| {
            black_box(color_attachment.get_attachment());
        });
    });

    group.bench_function("depth_attachment_get", |b| {
        b.iter(|| {
            black_box(depth_attachment.get_attachment(StoreOp::Store));
        });
    });

    group.bench_function("output_color_attachment_get", |b| {
        b.iter(|| {
            black_box(output_attachment.get_attachment(None));
        });
    });

    group.bench_function("color_attachment_new_and_get", |b| {
        b.iter(|| {
            let att = ColorAttachment::new(
                black_box(cached_texture.clone()),
                None,
                None,
                Some(black_box(Color::BLACK)),
            );
            black_box(att.get_attachment());
        });
    });

    group.finish();
}

criterion_group!(benches, texture_attachment);
