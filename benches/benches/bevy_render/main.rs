use criterion::criterion_main;

mod compute_normals;
mod render_layers;
mod texture_attachment;
mod torus;

criterion_main!(
    render_layers::benches,
    compute_normals::benches,
    texture_attachment::benches,
    torus::benches
);
