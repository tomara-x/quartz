use bevy::{
    core_pipeline::{
        bloom::{BloomSettings},
        tonemapping::Tonemapping,
    },
    sprite::MaterialMesh2dBundle,
    sprite::Material2d,
    prelude::*};

use rand::prelude::random;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_mod_picking::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

//use bevy_vector_shapes::prelude::*;
//use bevy::render::camera::ScalingMode;

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
        //.add_plugins(Shape2dPlugin::default())
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        //.add_plugins(BloomSettingsPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Off)
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_circles)
        .add_systems(Update, toggle_pan)
        .add_systems(Update, update_colors)
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
            tonemapping: Tonemapping::BlenderFilmic, //also for bloom
            transform: Transform::from_translation(Vec3::Z), //push the camera "back" one unit
            //projection: OrthographicProjection {
            //    scaling_mode: ScalingMode::AutoMin {
            //        min_width: 5.2 * 4.5,
            //        min_height: 3.2 * 4.5,
            //    },
            //    ..default()
            //},
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
        commands.spawn((MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(5.).into()).into(),
        material: materials.add(ColorMaterial::from(Color::rgb(255., 0., 170.))),
        transform: Transform::from_translation(point.extend(0.0)),
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
            mat.color = Color::hsl(random::<f32>()*360., 100.0, 50.0);
            }
    }
}

//fn draw_circles(mut painter: ShapePainter, query: Query<&Pos>) {
//    for pos in &query {
//        painter.reset();
//        painter.translate(pos.pos);
//        painter.color = Color::hsla(random::<f32>()*360., 100.0, 50.0, random::<f32>());
//        painter.circle(5.);
//    }
//}

fn toggle_pan(mut query: Query<&mut PanCam>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        for mut pancam in &mut query {
            pancam.enabled = !pancam.enabled;
        }
    }
}

