use bevy::{
    core_pipeline::{
        bloom::{BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*};

use rand::prelude::random;
use bevy_pancam::{PanCam, PanCamPlugin};
//use bevy_mod_picking::prelude::*;
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
        .insert_resource(Depth {z: -10.})
        .insert_resource(CursorInfo {i: Vec2{x:0.,y:0.}, f: Vec2{x:0.,y:0.}})
        .add_plugins(PanCamPlugin::default())
        //.add_plugins(DefaultPickingPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(BloomSettingsPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_and_move_circles)
        .add_systems(Update, toggle_pan)
        .add_systems(Update, update_colors)
        //.add_systems(Update, selection_check)
        .add_systems(Update, (update_cursor_info, draw_selection).chain())
        .run();
}

#[derive(Resource)]
struct Depth { z: f32 }

#[derive(Resource)]
struct CursorInfo {
    i: Vec2,
    f: Vec2,
}

#[derive(Component)]
struct Selectable;

#[derive(Component)]
struct Selected(bool);

fn setup(
    mut commands: Commands,
    mut config: ResMut<GizmoConfig>,
    ) {
    config.line_width = 1.;
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

fn spawn_and_move_circles(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut depth: ResMut<Depth>,
    ) {
    if mouse_button_input.just_pressed(MouseButton::Left) && keyboard_input.pressed(KeyCode::ControlRight) {
        let (camera, camera_transform) = camera_query.single();
        let Some(cursor_position) = windows.single().cursor_position() else { return; };
        let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else { return; };
        commands.spawn((ColorMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(5.).into()).into(),
        material: materials.add(ColorMaterial::from(Color::hsl(0., 1.0, 0.5))),
        transform: Transform::from_translation(point.extend(depth.z)),
        ..default()
        },
        Selectable,
        Selected(false),
        //PickableBundle::default(),
        //    //need to take pancam's zoom into account (and disable panning)
        //    On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE),
        //    On::<Pointer<DragEnd>>::target_insert(Pickable::default()),
        //    On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
        //        transform.translation.x += drag.delta.x;
        //        transform.translation.y -= drag.delta.y;
        //    }),
        ));
        depth.z += 0.00001;
    }
}

fn update_cursor_info(
    mouse_button_input: Res<Input<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut cursor: ResMut<CursorInfo>,
    ) {
    let (cam, cam_transform) = camera_query.single();
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let Some(cursor_pos) = windows.single().cursor_position() else { return; };
        let Some(point) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };
        cursor.i = point;
    }
    if mouse_button_input.pressed(MouseButton::Left) {
        let Some(cursor_pos) = windows.single().cursor_position() else { return; };
        let Some(point) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };
        cursor.f = point;
    }
}

fn draw_selection(
    cursor: Res<CursorInfo>,
    mut gizmos: Gizmos,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        gizmos.circle_2d(cursor.i, cursor.f.distance(cursor.i), Color::GREEN).segments(64);
    }
}


fn selection_check(
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut movable: Query<(&ViewVisibility, &mut Transform), With<Selectable>>,
    mut selected: Query<&Selected>,
    ) {
    let (cam, cam_transform) = camera_query.single();
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let Some(cursor_pos_i) = windows.single().cursor_position() else { return; };
        let Some(point_i) = cam.viewport_to_world_2d(cam_transform, cursor_pos_i) else { return; };
        
        println!("{:?}", point_i);
        //for (v, t) in &mut movable {
        //    if v.get() {
        //        println!("{:?}", t.translation);
        //    }
        //}
    }
}

fn update_colors(
    mut mats: ResMut<Assets<ColorMaterial>>,
    material_ids: Query<&Handle<ColorMaterial>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::C) {
        for id in material_ids.iter() {
            let mat = mats.get_mut(id).unwrap();
            mat.color = Color::hsl(random::<f32>()*360., 1.0, 0.5);
            }
    }
}

fn toggle_pan(mut query: Query<&mut PanCam>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::P) {
        for mut pancam in &mut query {
            pancam.enabled = true;
        }
    }
    if keys.just_released(KeyCode::P) {
        for mut pancam in &mut query {
            pancam.enabled = false;
        }
    }
}

