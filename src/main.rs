use bevy::{
    //ecs::system::SystemParam,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
        },
    //tasks::IoTaskPool,
    prelude::*};

use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

//use std::{fs::File, io::Write};
//use std::time::{Duration, Instant};
//use rand::prelude::random;

mod components;
mod process;
mod cursor;
mod connections;
mod circles;
use {components::*, process::*, cursor::*, connections::*, circles::*,};

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
        .add_plugins(WorldInspectorPlugin::new())

        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Sample4)

        .add_systems(Startup, setup)
        
        .add_systems(Update, toggle_pan)
        .add_state::<Mode>()
        .add_systems(Update, switch_mode)
        .add_systems(Update, save_scene)

        .insert_resource(CursorInfo::default())
        .add_systems(Update, update_cursor_info)

        // test high depth
        .insert_resource(Depth(-10.))
        .add_systems(Update, draw_pointer_circle.run_if(not(in_state(Mode::Connect))))
        .add_systems(Update, mark_visible.after(update_cursor_info))
        .add_systems(Update, update_selection.after(mark_visible).run_if(in_state(Mode::Edit)))
        .add_systems(Update, move_selected.after(update_selection).run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_color.after(update_selection).run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_radius.after(update_selection).run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_num.after(update_selection).run_if(in_state(Mode::Edit)))
        .add_systems(Update, highlight_selected.run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_order.run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_op.run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_circle_text.run_if(in_state(Mode::Edit)))
        .add_systems(Update, select_all.run_if(in_state(Mode::Edit)))
        .add_systems(Update, duplicate_selected.run_if(in_state(Mode::Edit)))

        .add_systems(Update, connect.run_if(in_state(Mode::Connect)))
        .add_systems(Update, draw_connections)
        .add_systems(Update, draw_connecting_line.run_if(in_state(Mode::Connect)))
        .add_systems(Update, update_link_type.run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_link_type_text.run_if(in_state(Mode::Edit)))

        // order
        .add_systems(Update, (spawn_circles.run_if(in_state(Mode::Draw)),
                              delete_selected.run_if(in_state(Mode::Edit)),
                              apply_deferred, //to make sure the commands are applied
                              sort_by_order.run_if(on_event::<OrderChange>())).chain())
        .register_type::<Queue>()
        .init_resource::<Queue>()
        .add_event::<OrderChange>()

        .add_systems(Update, process.after(sort_by_order))

        .run();
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
        BloomSettings {
            intensity: 0.5,
            low_frequency_boost: 0.6,
            low_frequency_boost_curvature: 0.4,
            composite_mode: BloomCompositeMode::Additive,
            ..default()
        },
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
    if keyboard_input.just_pressed(KeyCode::Space) {
        let mut pancam = query.single_mut();
        pancam.enabled = true;
    }
    if keyboard_input.just_released(KeyCode::Space) {
        let mut pancam = query.single_mut();
        pancam.enabled = false;
    }
}


fn switch_mode(
    mut next_state: ResMut<NextState<Mode>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keyboard_input.just_pressed(KeyCode::Key1) { next_state.set(Mode::Draw); }
        if keyboard_input.just_pressed(KeyCode::Key2) { next_state.set(Mode::Connect); }
        if keyboard_input.just_pressed(KeyCode::Key3) { next_state.set(Mode::Edit); }
    }
}

// own file format?
// query the info needed to respawn the same entities on load
// switching?
// creating multiple worlds, switching between them, and saving/loading them
fn save_scene(
    circles_query: Query<&GlobalTransform, With<Order>>,
    keyboard_input: Res<Input<KeyCode>>,
    ) {
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if ctrl && keyboard_input.just_pressed(KeyCode::S) {
        for circle in circles_query.iter() {
            info!("{:?}", circle.translation());
        }
    }
//        #[cfg(not(target_arch = "wasm32"))]
//        IoTaskPool::get()
//            .spawn(async move {
//                File::create(format!("scene"))
//                    .and_then(|mut file| file.write(serialized_scene.as_bytes()))
//                    .expect("Error while writing scene to file");
//            })
//            .detach();
//    }
}

fn draw_pointer_circle(
    cursor: Res<CursorInfo>,
    mut gizmos: Gizmos,
    time: Res<Time>,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) &&
    ! mouse_button_input.just_pressed(MouseButton::Left) &&
    !keyboard_input.pressed(KeyCode::Space) {
        let color = Color::hsl((time.elapsed_seconds() * 100.) % 360., 1.0, 0.5);
        gizmos.circle_2d(cursor.i, cursor.f.distance(cursor.i), color).segments(64);
    }
}


