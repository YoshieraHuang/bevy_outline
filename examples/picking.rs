use bevy::prelude::*;
use bevy_mod_picking::{PickableBundle, PickingCameraBundle};
use bevy_obj::ObjPlugin;
use bevy_outline::{
    picking::{DefaultPickingPlugins, HoverOutline, PressedOutline, SelectedOutline},
    OutlineMaterial,
};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(ObjPlugin)
        .add_startup_system(set_picking_outlines)
        .add_startup_system(setup)
        .run();
}

fn set_picking_outlines(mut commands: Commands, mut outlines: ResMut<Assets<OutlineMaterial>>) {
    // Comment below to disable hover outline
    commands.insert_resource(HoverOutline(outlines.add(OutlineMaterial {
        width: 5.,
        color: Color::BLACK,
    })));
    // Comment below to disable selected outline
    commands.insert_resource(SelectedOutline(outlines.add(OutlineMaterial {
        width: 5.,
        color: Color::WHITE,
    })));
    // Comment below to disable pressed outline
    commands.insert_resource(PressedOutline(outlines.add(OutlineMaterial {
        width: 5.,
        color: Color::BLUE,
    })));
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ambient_light: ResMut<AmbientLight>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::default())),
            material: materials.add(Color::rgb(0.2, 0.7, 0.8).into()),
            transform: Transform::from_xyz(2.0, 0.5, 0.0),
            ..default()
        })
        .insert_bundle(PickableBundle::default());

    // Sphere
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere::default())),
            material: materials.add(Color::rgb(0.3, 0.2, 0.1).into()),
            transform: Transform::from_xyz(-2.0, 0.5, 0.0),
            ..default()
        })
        .insert_bundle(PickableBundle::default());

    // Torus
    // The built-in torus has some mistakes which will be fixed in 0.8
    commands
        .spawn_bundle(PbrBundle {
            mesh: asset_server.load("torus.obj"),
            material: materials.add(Color::rgb(0.2, 0.2, 0.5).into()),
            transform: Transform::from_xyz(6.0, 0.5, 0.0),
            ..default()
        })
        .insert_bundle(PickableBundle::default());

    // Monkey head
    commands
        .spawn_bundle(PbrBundle {
            mesh: asset_server.load("head.obj"),
            material: materials.add(Color::rgb(0.7, 0.2, 0.5).into()),
            transform: Transform::from_xyz(-6.0, 0.5, 0.0),
            ..default()
        })
        .insert_bundle(PickableBundle::default());

    // Light
    ambient_light.brightness = 1.0;

    // camera
    let camera_translation = Vec3::new(0.0, 6.0, 12.0);
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(camera_translation)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert_bundle(PickingCameraBundle::default());
}
