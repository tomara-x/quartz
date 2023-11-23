use bevy::{
    core_pipeline::{
        bloom::{BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*};

use rand::prelude::random;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_mod_picking::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod bloom_settings;
use bloom_settings::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("awawawa"),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PanCamPlugin::default())
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(BloomSettingsPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_circles)
        .add_systems(Update, toggle_pan)
        .add_systems(Update, update_colors)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        //Camera2dBundle::default(),
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::BlenderFilmic,
            transform: Transform::from_translation(Vec3::Z), //push the camera "back" one unit
        ..default()
        },
        BloomSettings::default(), //enable bloom
        PanCam {
            //limit zooming
            max_scale: Some(80.),
            min_scale: 0.005,
            ..default()
        },
    ));
}

fn spawn_circles(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
    if mouse_button_input.just_pressed(MouseButton::Left) && keyboard_input.pressed(KeyCode::ControlRight) {
        let (camera, camera_transform) = camera_query.single();
        let Some(cursor_position) = windows.single().cursor_position() else { return; };
        let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else { return; };
        commands.spawn((ColorMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(5.).into()).into(),
        material: materials.add(ColorMaterial::from(Color::hsl(0., 1.0, 0.5))),
        transform: Transform::from_translation(point.extend(-1.0)),
        ..default()
        },
        PickableBundle::default(),
        ));
    }
}

fn update_colors(
    mut mats: ResMut<Assets<ColorMaterial>>,
    materials: Query<&Handle<ColorMaterial>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.pressed(KeyCode::C) {
        for m in materials.iter() {
            let mat = mats.get_mut(m).unwrap();
            mat.color = Color::hsl(random::<f32>()*360., 1.0, 0.5);
            }
    }
}

fn toggle_pan(mut query: Query<&mut PanCam>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        for mut pancam in &mut query {
            pancam.enabled = !pancam.enabled;
        }
    }
}

