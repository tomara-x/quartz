use quartz::{
    bloom_settings::*,
    circles::*,
};

use bevy::{
    core_pipeline::{
        bloom::{BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*};

use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
//use rand::prelude::random;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("awawawa"),
                ..default()
            }),
            ..default()
        }))
        //RESOURCES
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Off)
        //PLUGINS
        .add_plugins(PanCamPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        //INTERNAL PLUGINS
        .add_plugins(BloomSettingsPlugin)
        .add_plugins(CirclesPlugin)
        //SYSTEMS
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_pan)
        .run();
}

//connections
enum Target {
    Color,
    Pos,
    Data,
    Block,
    Radius,
}

#[derive(Component)]
struct LinkFrom(Vec<(Entity, Target, Target)>);

// will be used to clear connections after despawning an entity
#[derive(Component)]
struct LinkTo(Vec<Entity>);

#[derive(Component)]
enum Op {
    Pass,
    Add,
    Mult,
}

fn setup(
    mut commands: Commands,
    mut config: ResMut<GizmoConfig>,
) {
    config.line_width = 1.;
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_translation(Vec3::Z), //push the camera "back" one unit
        ..default()
        },
        BloomSettings::default(), //enable bloom
        PanCam {
            enabled: false,
            //limit zooming
            max_scale: Some(80.),
            min_scale: 0.005,
            ..default()
        },
    ));
}

fn toggle_pan(
    mut query: Query<&mut PanCam>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::P) {
        let mut pancam = query.single_mut();
        pancam.enabled = true;
    }
    if keyboard_input.just_released(KeyCode::P) {
        let mut pancam = query.single_mut();
        pancam.enabled = false;
    }
}

