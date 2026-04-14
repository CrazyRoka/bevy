//! A stress test designed to highlight extraction overhead in `bevy_pbr::light::extract_lights`.
//!
//! Tuned to avoid GPU bottlenecks while maximizing CPU extraction/culling overhead.

use argh::FromArgs;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{PresentMode, WindowResolution},
    winit::WinitSettings,
};
use std::f32::consts::PI;

#[derive(FromArgs, Resource)]
#[argh(description = "Many Shadow Lights Stress Test")]
struct Args {
    /// number of shadow-casting point lights (Default: 150)
    #[argh(option, short = 'l', default = "150")]
    light_count: usize,

    /// grid size of meshes (Default: 40 -> 40x40 = 1,600 meshes)
    #[argh(option, short = 'm', default = "40")]
    mesh_grid_size: usize,

    /// disable moving the lights
    #[argh(switch)]
    static_lights: bool,
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let args: Args = argh::from_env();
    #[cfg(target_arch = "wasm32")]
    let args = Args::from_args(&[], &[]).unwrap();

    println!("Many Shadow Lights Stress Test");
    println!(
        "Spawning {} meshes and {} shadow-casting point lights.",
        args.mesh_grid_size * args.mesh_grid_size,
        args.light_count
    );
    println!("Total shadow views per frame: {}", args.light_count * 6);

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(1920, 1080).with_scale_factor_override(1.0),
                    title: "many_shadow_lights".into(),
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .insert_resource(WinitSettings::continuous())
        // Disable ambient light to make the point lights highly visible
        .insert_resource(GlobalAmbientLight {
            color: Color::BLACK,
            brightness: 0.0,
            ..default()
        })
        .insert_resource(args)
        .add_systems(Startup, setup)
        .add_systems(Update, move_lights)
        .run();
}

#[derive(Component)]
struct MovingLight {
    angle: f32,
    radius: f32,
    speed_multiplier: f32,
}

fn setup(
    mut commands: Commands,
    args: Res<Args>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let n = args.mesh_grid_size;
    let spacing = 2.5;
    let offset = (n as f32 * spacing) / 2.0;
    let light_radius = offset * 0.8;

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, offset * 0.8, offset * 1.2).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Make cubes slightly smaller than spacing to create gaps for shadows
    let mesh_handle = meshes.add(Cuboid::from_size(Vec3::splat(1.5)));
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.8,
        ..default()
    });

    for x in 0..n {
        for z in 0..n {
            commands.spawn((
                Mesh3d(mesh_handle.clone()),
                MeshMaterial3d(material_handle.clone()),
                Transform::from_xyz(
                    x as f32 * spacing - offset,
                    0.0,
                    z as f32 * spacing - offset,
                ),
            ));
        }
    }

    for i in 0..args.light_count {
        let angle = (i as f32) / (args.light_count as f32) * PI * 2.0;
        // Alternate light heights to create dynamic intersecting shadows
        let height = 2.0 + (i % 3) as f32 * 3.0;
        let speed = 0.5 + (i % 5) as f32 * 0.1;

        commands.spawn((
            PointLight {
                intensity: 5_000_000.0, // Cranked up so they are visible from far away
                range: light_radius * 2.5,
                shadow_maps_enabled: true,
                // Give lights different colors to see them clearly
                color: Color::hsl((i as f32 * 137.5) % 360.0, 1.0, 0.5),
                ..default()
            },
            Transform::from_xyz(angle.cos() * light_radius, height, angle.sin() * light_radius),
            MovingLight {
                angle,
                radius: light_radius,
                speed_multiplier: speed,
            },
        ));
    }
}

fn move_lights(time: Res<Time>, args: Res<Args>, mut query: Query<(&mut Transform, &mut MovingLight)>) {
    if args.static_lights {
        return;
    }

    let delta = time.delta_secs();
    for (mut transform, mut light) in &mut query {
        light.angle += delta * light.speed_multiplier;
        transform.translation.x = light.angle.cos() * light.radius;
        transform.translation.z = light.angle.sin() * light.radius;
    }
}
