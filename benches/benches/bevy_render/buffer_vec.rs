use bevy_render::render_resource::{BufferUsages, PollType, RawBufferVec};
use bevy_render::renderer::initialize_renderer;
use bevy_render::settings::{RenderResources, WgpuSettings};
use bevy_tasks::block_on;
use core::hint::black_box;
use criterion::{criterion_group, Criterion};
use std::time::{Duration, Instant};

pub fn buffer_vec_benches(c: &mut Criterion) {
    let settings = WgpuSettings::default();
    let backends = settings.backends.expect("No backends found");

    let RenderResources(device, queue, ..) = block_on(initialize_renderer(
        backends,
        None, // No window needed for buffer tests
        &settings,
        #[cfg(feature = "raw_vulkan_init")]
        Default::default(),
    ));

    c.bench_function("raw_buffer_vec_incremental_write", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::default();

            for _ in 0..iters {
                let start = Instant::now();

                // Simulate incremental writes to the buffer vec, writing after each push
                let mut vec = RawBufferVec::<u32>::new(BufferUsages::STORAGE);
                for i in 0..5000 {
                    vec.push(i);
                    vec.write_buffer(&device, &queue);
                }
                total += Instant::now().duration_since(start);

                black_box(&vec);
                // Cleanup allocated buffers to prevent memory bloat during the benchmark
                queue.submit(None);
                device
                    .poll(PollType::wait_indefinitely())
                    .expect("Failed to poll device");
            }

            total
        });
    });
}

criterion_group!(benches, buffer_vec_benches);
