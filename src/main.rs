use quartz::{
    bloom_settings::*,
    circles::*,
    cursor::*,
    connections::*,
    detachable_components::*,
};

use bevy::{
    core_pipeline::{
        bloom::{BloomSettings},
        tonemapping::Tonemapping,
    },
    //tasks::IoTaskPool,
    prelude::*};

//use std::{fs::File, io::Write};

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
        //States
        //.add_state::<Mode>()
        //RESOURCES
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Off)
        //PLUGINS
        .add_plugins(PanCamPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        //INTERNAL PLUGINS
        .add_plugins(BloomSettingsPlugin)
        .add_plugins(CirclesPlugin)
        .add_plugins(CursorPlugin)
        .add_plugins(ConnectionsPlugin)
        .add_plugins(DetachableComponentsPlugin)
        //SYSTEMS
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_pan)
        //.add_systems(Update, save_scene)
        .run();
}

// circles for all!!
// spawn in setup and they get their own systems and markers for quick query
//#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
//enum Mode {
//    #[default]
//    Edit,
//    Run,
//}

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

//scene saving mess
//fn save_scene(world: &mut World) {
//    let keyboard_input = world.resource::<Input<KeyCode>>();
//    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
//    if ctrl && keyboard_input.just_pressed(KeyCode::S) {
//        let mut query = world.query_filtered::<Entity, With<Pos>>();
//        let scene = DynamicSceneBuilder::from_world(&world).allow::<Pos>().build();
//
//        let type_registry = world.resource::<AppTypeRegistry>();
//        let serialized_scene = scene.serialize_ron(type_registry).unwrap();
//
//        #[cfg(not(target_arch = "wasm32"))]
//        IoTaskPool::get()
//            .spawn(async move {
//                // Write the scene RON data to file
//                File::create(format!("scene"))
//                    .and_then(|mut file| file.write(serialized_scene.as_bytes()))
//                    .expect("Error while writing scene to file");
//            })
//            .detach();
//    }
//}

