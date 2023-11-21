//use std::f32::consts::TAU;

use bevy::{
    core_pipeline::{
        bloom::{BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*};
use bevy_vector_shapes::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy::render::camera::ScalingMode;
use rand::prelude::random;

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
        .add_plugins(Shape2dPlugin::default())
        .add_plugins(BloomSettingsPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_circles, draw_circles))
        .add_systems(Update, toggle_pan)
        .run();
}

#[derive(Component)]
struct Pos {
    pos: Vec3
}

fn setup(mut commands: Commands) {
    // Spawn the camera
    commands.spawn((
        //Camera2dBundle::default(),
        Camera2dBundle {
            camera: Camera {
                hdr: true, //for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, //also for bloom
            transform: Transform::from_translation(Vec3::Z), //push the camera "back" one unit
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::AutoMin { //something to do with window size
                    min_width: 5.2 * 4.5,
                    min_height: 3.2 * 4.5,
                },
                ..default()
            },
        ..default()
        },
        BloomSettings::default(), //enable bloom
        PanCam {
            //limit zooming
            max_scale: Some(40.),
            min_scale: 0.25,
            ..default()
        },
    ));
}

fn spawn_circles(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    ) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let (camera, camera_transform) = camera_query.single();
        let Some(cursor_position) = windows.single().cursor_position() else {
            return;
        };
        let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
            return;
        };
        commands.spawn(    
            Pos {
                pos: point.extend(0.0)
            }
        );
    }
}

fn draw_circles(mut painter: ShapePainter, query: Query<&Pos>) {
    for pos in &query {
        painter.reset();
        painter.translate(pos.pos);
        painter.color = Color::hsla(random::<f32>()*360., 100.0, 50.0, random::<f32>());
        painter.circle(5.);
    }
}

fn toggle_pan(mut query: Query<&mut PanCam>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        for mut pancam in &mut query {
            pancam.enabled = !pancam.enabled;
        }
    }
}

