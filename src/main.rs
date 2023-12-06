use bevy::{
    core_pipeline::{
        bloom::{BloomSettings},
        tonemapping::Tonemapping,
    },
    render::view::VisibleEntities,
    prelude::*};

use rand::prelude::random;
use bevy_pancam::{PanCam, PanCamPlugin};
//use bevy_inspector_egui::quick::WorldInspectorPlugin;

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
        //RESOURCES
        .insert_resource(Depth {value: -10.})
        .insert_resource(CursorInfo {i: Vec2::ZERO, f: Vec2::ZERO})
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Off)
        //PLUGINS
        .add_plugins(PanCamPlugin::default())
        //.add_plugins(WorldInspectorPlugin::new())
        //INTERNAL PLUGINS
        .add_plugins(BloomSettingsPlugin)
        //SYSTEMS
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_circles)
        .add_systems(Update, toggle_pan)
        .add_systems(Update, update_colors)
        .add_systems(Update, draw_pointer_circle)
        .add_systems(Update,
            (update_cursor_info, mark_visible, update_selection, highlight_selected, move_selected)
            .chain())
        .add_systems(Update, delete_selected)
        .run();
}

#[derive(Resource)]
struct Depth { value: f32 }

#[derive(Resource)]
struct CursorInfo {
    i: Vec2,
    f: Vec2,
}

#[derive(Component)]
struct Radius { value: f32 }

#[derive(Component)]
struct Pos { value: Vec3 }

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct Visible;


//in the params for the systems, make sure you have queries for every component
//(radius, color, position, data, extra, etc) then destructure based on match
#[derive(Component)]
struct Connection {
    src: Vec<Entity>,
    dst: Vec<Entity>,
    src_target: String,
    dst_target: String,
    op: String,
    order: usize,
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut depth: ResMut<Depth>,
    cursor: Res<CursorInfo>,
) {
    if mouse_button_input.just_released(MouseButton::Left) && keyboard_input.pressed(KeyCode::Z) {
        commands.spawn((ColorMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(cursor.f.distance(cursor.i)).into()).into(),
        material: materials.add(ColorMaterial::from(Color::hsl(0., 1.0, 0.5))),
        transform: Transform::from_translation(cursor.i.extend(depth.value)),
        ..default()
        },
        Radius { value: cursor.f.distance(cursor.i)}, //opt?
        Pos { value: cursor.i.extend(depth.value)},
        ));
        depth.value += 0.00001;
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

fn update_colors(
    mut mats: ResMut<Assets<ColorMaterial>>,
    material_ids: Query<&Handle<ColorMaterial>, With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
) {
    if keyboard_input.pressed(KeyCode::C) {
        for id in material_ids.iter() {
            let mat = mats.get_mut(id).unwrap();
            mat.color = Color::hsl(cursor.i.distance(cursor.f)%360., 1.0, 0.5);
        }
    }
}

fn toggle_pan(
    mut query: Query<&mut PanCam>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::V) {
        let mut pancam = query.single_mut();
        pancam.enabled = true;
    }
    if keyboard_input.just_released(KeyCode::V) {
        let mut pancam = query.single_mut();
        pancam.enabled = false;
    }
}


fn draw_pointer_circle(
    cursor: Res<CursorInfo>,
    mut gizmos: Gizmos,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        gizmos.circle_2d(cursor.i, cursor.f.distance(cursor.i), Color::GREEN).segments(64);
    }
}

fn highlight_selected(
    mut gizmos: Gizmos,
    query: Query<(&Radius, &Pos), With<Selected>>,
) {
    for (r, p) in query.iter() {
        gizmos.circle_2d(p.value.xy(), r.value, Color::RED).segments(64);
    }
}

fn mark_visible(
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    query: Query<Entity, With<Visible>>,
    visible: Query<&VisibleEntities>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        for e in query.iter() {
            commands.entity(e).remove::<Visible>();
        }
        let vis = visible.single();
        for e in &vis.entities {
            commands.entity(*e).insert(Visible);
        }
    }
}

fn update_selection(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<(Entity, &Radius, &Pos), Or<(With<Visible>, With<Selected>)>>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) && keyboard_input.pressed(KeyCode::X) {
        if !keyboard_input.pressed(KeyCode::ShiftLeft) {
            for (e, _, _) in query.iter() {
                commands.entity(e).remove::<Selected>();
            }
        }
        for (e, r, p) in query.iter() {
            if cursor.i.distance(p.value.xy()) < r.value {
                commands.entity(e).insert(Selected);
                break;
            }
        }
    }
}

fn move_selected(
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
    mut query: Query<(&mut Transform, &mut Pos), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) && keyboard_input.pressed(KeyCode::X) {
        for (mut t, p) in query.iter_mut() {
            t.translation = (p.value.xy() + cursor.f - cursor.i).extend(p.value.z);
            //t.translation.x = p.value.x + cursor.f.x - cursor.i.x;
            //t.translation.y = p.value.y + cursor.f.y - cursor.i.y;
        }
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        for (t, mut p) in query.iter_mut() {
            p.value = t.translation;
        }
    }
}

fn delete_selected(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<Entity, With<Selected>>,
    mut commands: Commands,
) {
    if keyboard_input.pressed(KeyCode::Delete) {
        for id in query.iter() {
            commands.entity(id).despawn();
        }
    }
}

