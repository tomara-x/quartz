use bevy::{
    //ecs::system::SystemParam,
    render::view::VisibleEntities,
    sprite::Mesh2dHandle,
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
        
        .register_type::<Offset>()
        .run();
}


macro_rules! mark_changed {
    ($n:expr, $children:expr, $bh_query:expr, $wh_query:expr) => {
        for child in $children.iter() {
            if let Ok(black_hole) = $bh_query.get(*child) {
                if black_hole.link_type == $n {
                    $wh_query.get_mut(black_hole.wh).unwrap().changed = true;
                }
            }
        }
    };
}

// ------------------- process -----------------------

#[derive(Component)]
struct Op(i32);

fn process(
    queue: Res<Queue>,
    children_query: Query<&Children>,
    black_hole_query: Query<&BlackHole>,
    mut white_hole_query: Query<&mut WhiteHole>,
    mut op_query: Query<&mut Op>,
    mut bloom: Query<&mut BloomSettings, With<Camera>>,
    mut num_query: Query<&mut Num>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    material_ids: Query<&Handle<ColorMaterial>>,
    mut radius_query: Query<&mut Radius>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
    mut trans_query: Query<&mut Transform>,
    mut offset_query: Query<&mut Offset>,
) {
    for id in queue.0.iter().flatten() {
        let children = children_query.get(*id).unwrap();
        match op_query.get(*id).unwrap().0 {
            -3 => {
            },
            -2 => {
            },
            -1 => { // 3 floats to position
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if white_hole.changed {
                            let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                            if black_hole.link_type == -4 && (1..4).contains(&white_hole.link_type) {
                                white_hole.changed = false;
                                let input = num_query.get(black_hole.parent).unwrap().0;
                                let mut t = trans_query.get_mut(*id).unwrap();
                                match white_hole.link_type {
                                    1 => t.translation.x = input,
                                    2 => t.translation.y = input,
                                    3 => t.translation.z = input,
                                    _ => {},
                                }
                                // position has changed
                                mark_changed!(-1, children, black_hole_query, white_hole_query);
                            }
                        }
                    }
                }
            },
            0 => { // pass
                // input to num
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if white_hole.changed {
                            let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                            if black_hole.link_type == -4 && white_hole.link_type == -4 {
                                white_hole.changed = false;
                                let input = num_query.get(black_hole.parent).unwrap().0;
                                num_query.get_mut(*id).unwrap().0 = input;
                                mark_changed!(-4, children, black_hole_query, white_hole_query);
                            }
                        }
                    }
                }
            },
            1 => { // bloom control
                let mut bloom_settings = bloom.single_mut();
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if !white_hole.changed { continue; }
                        let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                        if black_hole.link_type == -4 && (1..8).contains(&white_hole.link_type) {
                            white_hole.changed = false;
                            let input = num_query.get(black_hole.parent).unwrap().0 / 100.;
                            match white_hole.link_type {
                                1 => bloom_settings.intensity = input,
                                2 => bloom_settings.low_frequency_boost = input,
                                3 => bloom_settings.low_frequency_boost_curvature = input,
                                4 => bloom_settings.high_pass_frequency = input,
                                5 => bloom_settings.composite_mode = if input > 0. {
                                BloomCompositeMode::Additive } else { BloomCompositeMode::EnergyConserving },
                                6 => bloom_settings.prefilter_settings.threshold = input,
                                7 => bloom_settings.prefilter_settings.threshold_softness = input,
                                _ => {},
                            }
                        }
                    }
                }
            },
            _ => {},
        }
        for child in children {
            if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                if !white_hole.changed { continue; }
                let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                match (black_hole.link_type, white_hole.link_type) {
                    (-1, -1) => { //trans
                        white_hole.changed = false;
                        let input = trans_query.get(black_hole.parent).unwrap().translation;
                        let mut t = trans_query.get_mut(*id).unwrap();
                        let offset = offset_query.get(*id).unwrap().trans;
                        t.translation.x = input.x + offset.x;
                        t.translation.y = input.y + offset.y;
                        t.translation.z = input.z + offset.z;
                        mark_changed!(-1, children, black_hole_query, white_hole_query);
                    },
                    (-2, -2) => { // color
                        white_hole.changed = false;
                        let mat_id = material_ids.get(black_hole.parent).unwrap();
                        let offset = offset_query.get(*id).unwrap().color;
                        let mat = mats.get(mat_id).unwrap();
                        let input = mat.color;
                        mats.get_mut(material_ids.get(*id).unwrap()).unwrap().color = input + offset;
                        mark_changed!(-2, children, black_hole_query, white_hole_query);
                    },
                    (-3, -3) => { // radius
                        white_hole.changed = false;
                        if let Ok(Mesh2dHandle(mesh_id)) = mesh_ids.get(*id) {
                            let offset = offset_query.get(*id).unwrap().radius;
                            let input = radius_query.get(black_hole.parent).unwrap().0;
                            radius_query.get_mut(*id).unwrap().0 = input + offset;
                            let mesh = meshes.get_mut(mesh_id).unwrap();
                            *mesh = shape::Circle::new(input + offset).into();
                        }
                        mark_changed!(-3, children, black_hole_query, white_hole_query);
                    },
                    (-4, -5) => { // number to op
                        white_hole.changed = false;
                        let input = num_query.get(black_hole.parent).unwrap().0;
                        op_query.get_mut(*id).unwrap().0 = input as i32;
                    }
                    (-1, -6) => { // position to trans offset
                        white_hole.changed = false;
                        let input = trans_query.get(black_hole.parent).unwrap().translation;
                        offset_query.get_mut(*id).unwrap().trans = input;
                    },
                    (-2, -7) => { // color to color offset
                        white_hole.changed = false;
                        let mat_id = material_ids.get(black_hole.parent).unwrap();
                        let mat = mats.get(mat_id).unwrap();
                        let input = mat.color;
                        offset_query.get_mut(*id).unwrap().color = input;
                    },
                    (-3, -8) => { // radius to radius offset
                        white_hole.changed = false;
                        let input = radius_query.get(black_hole.parent).unwrap().0;
                        offset_query.get_mut(*id).unwrap().radius = input;
                    }
                    _ => {},
                }
            }
        }
    }
}


// ----------------------- main --------------------------

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

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum Mode {
    #[default]
    Draw,
    Connect,
    Edit,
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

// ---------------------- order ------------------------

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
struct Queue(Vec<Vec<Entity>>);

#[derive(Event, Default)]
struct OrderChange;

fn sort_by_order(
    query: Query<(Entity, &Order)>,
    mut max_order: Local<usize>,
    mut queue: ResMut<Queue>,
) {
    *max_order = 1;
    queue.0.clear();
    queue.0.push(Vec::new());
    for (entity, order) in query.iter() {
        if order.0 > 0 {
            if order.0 > *max_order {
                queue.0.resize(order.0, Vec::new());
                *max_order = order.0;
            }
            queue.0[order.0 - 1].push(entity); //order 1 at index 0
        }
    }
}

fn update_order (
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Order, With<Selected>>,
    mut order_change: EventWriter<OrderChange>,
) {
    if keyboard_input.just_pressed(KeyCode::BracketRight) {
        for mut order in query.iter_mut() {
            order.0 += 1;
            order_change.send_default();
        }
    }
    if keyboard_input.just_pressed(KeyCode::BracketLeft) {
        for mut order in query.iter_mut() {
            if order.0 > 0 {
                order.0 -= 1;
                order_change.send_default();
            }
        }
    }
}

// ---------------------- cursor ------------------------

// initial, final, delta
#[derive(Resource, Default)]
struct CursorInfo {
    i: Vec2,
    f: Vec2,
    d: Vec2,
}

fn update_cursor_info(
    mouse_button_input: Res<Input<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut cursor: ResMut<CursorInfo>,
    mut last_pos: Local<Vec2>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let (cam, cam_transform) = camera_query.single();
        if let Some(cursor_pos) = windows.single().cursor_position() {
            if let Some(point) = cam.viewport_to_world_2d(cam_transform, cursor_pos) {
                cursor.i = point;
            }
        }
    }
    if mouse_button_input.pressed(MouseButton::Left) {
        let (cam, cam_transform) = camera_query.single();
        if let Some(cursor_pos) = windows.single().cursor_position() {
            if let Some(point) = cam.viewport_to_world_2d(cam_transform, cursor_pos) {
                cursor.f = point;
                cursor.d = point - *last_pos;
                *last_pos = point;
            }
        }
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        cursor.d = Vec2::ZERO;
        *last_pos = -cursor.f; // so on the pressed frame we don't get a delta
    }
}

// ------------- circles ------------------------

#[derive(Component)]
struct Num(f32);

#[derive(Component)]
struct Arr(Vec<f32>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Offset {
    trans: Vec3,
    color: Color,
    radius: f32,
}

#[derive(Resource)]
struct Depth(f32);

#[derive(Component)]
struct Radius(f32);

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct Visible;

#[derive(Component)]
struct Order(usize);

fn spawn_circles(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut depth: ResMut<Depth>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if mouse_button_input.just_released(MouseButton::Left) &&
    !keyboard_input.pressed(KeyCode::Space) {
        let radius = cursor.f.distance(cursor.i);
        let id = commands.spawn((
            ColorMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(radius).into()).into(),
                material: materials.add(ColorMaterial::from(Color::hsl(0., 1.0, 0.5))),
                transform: Transform::from_translation(cursor.i.extend(depth.0)),
                ..default()
            },
            Radius(radius),
            Visible, //otherwise it can't be selected til after mark_visible is updated
            Order(0),
            Num(0.),
            Arr(Vec::new()),
            Offset {trans:Vec3::ZERO, color:Color::BLACK, radius:0.},
            Op(0),
        )).id();

        // have the circle adopt a text entity
        let text = commands.spawn(Text2dBundle {
            text: Text::from_sections([
                TextSection::new(
                    id.index().to_string() + "v" + &id.generation().to_string() + "\n",
                    TextStyle { color: Color::BLACK, ..default() },
                ),
                TextSection::new(
                    "order: 0\n",
                    TextStyle { color: Color::BLACK, ..default() },
                ),
                TextSection::new(
                    "op: yaas\n",
                    TextStyle { color: Color::BLACK, ..default() },
                ),
                TextSection::new(
                    "0",
                    TextStyle { color: Color::BLACK, ..default() },
                ),
            ]),
            transform: Transform::from_translation(Vec3{z:0.000001, ..default()}),
            ..default()
        }).id();
        commands.entity(id).add_child(text);

        depth.0 += 0.00001;
    }
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

fn highlight_selected(
    mut gizmos: Gizmos,
    time: Res<Time>,
    query: Query<(&Radius, &GlobalTransform), With<Selected>>,
) {
    for (r, t) in query.iter() {
        let color = Color::hsl((time.elapsed_seconds() * 100.) % 360., 1.0, 0.5);
        gizmos.circle_2d(t.translation().xy(), r.0, color).segments(64);
    }
}

// loop over the visible entities and give them a Visible component
// so we can query just the visible entities
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
        for e in vis.iter() {
            commands.entity(*e).insert(Visible);
        }
    }
}

//optimize all those distance calls, use a distance squared instead
fn update_selection(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<(Entity, &Radius, &GlobalTransform), Or<(With<Visible>, With<Selected>)>>,
    selected: Query<Entity, With<Selected>>,
    selected_query: Query<&Selected>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<Input<KeyCode>>,
    mut top_clicked_circle: Local<Option<(Entity, f32)>>,
) {
    if keyboard_input.pressed(KeyCode::Space) { return; }
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if mouse_button_input.just_pressed(MouseButton::Left) {
        for (e, r, t) in query.iter() {
            if top_clicked_circle.is_some() {
                if t.translation().z > top_clicked_circle.unwrap().1 &&
                    cursor.i.distance(t.translation().xy()) < r.0 {
                    *top_clicked_circle = Some((e, t.translation().z));
                }
            } else {
                if cursor.i.distance(t.translation().xy()) < r.0 {
                    *top_clicked_circle = Some((e, t.translation().z));
                }
            }
        }
        if let Some(top) = *top_clicked_circle {
            if !selected_query.contains(top.0) {
                if shift { commands.entity(top.0).insert(Selected); }
                else {
                    for entity in selected.iter() {
                        commands.entity(entity).remove::<Selected>();
                    }
                    commands.entity(top.0).insert(Selected);
                }
            }
        }
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        if top_clicked_circle.is_none() {
            if !shift {
                for entity in selected.iter() {
                    commands.entity(entity).remove::<Selected>();
                }
            }
            // select those in the dragged area
            for (e, r, t) in query.iter() {
                if cursor.i.distance(cursor.f) + r.0 > cursor.i.distance(t.translation().xy()) {
                    commands.entity(e).insert(Selected);
                }
            }
        }
        *top_clicked_circle = None;
    }
}

fn move_selected(
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
    mut query: Query<(&mut Transform, &Children), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    black_hole_query: Query<&BlackHole>,
    mut white_hole_query: Query<&mut WhiteHole>,
) {
    if keyboard_input.pressed(KeyCode::Key1) {
        if mouse_button_input.pressed(MouseButton::Left) &&
        //lol because the update to entities isn't read until the next frame
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for (mut t, children) in query.iter_mut() {
                t.translation.x += cursor.d.x;
                t.translation.y += cursor.d.y;
                mark_changed!(-1, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Up) {
            for (mut t, children) in query.iter_mut() {
                t.translation.y += 1.;
                mark_changed!(-1, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for (mut t, children) in query.iter_mut() {
                t.translation.y -= 1.;
                mark_changed!(-1, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Right) {
            for (mut t, children) in query.iter_mut() {
                t.translation.x += 1.;
                mark_changed!(-1, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Left) {
            for (mut t, children) in query.iter_mut() {
                t.translation.x -= 1.;
                mark_changed!(-1, children, black_hole_query, white_hole_query);
            }
        }
    }
}

fn update_color(
    mut mats: ResMut<Assets<ColorMaterial>>,
    material_ids: Query<(&Handle<ColorMaterial>, &Children), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<&BlackHole>,
) {
    if keyboard_input.pressed(KeyCode::Key2) {
        if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for (id, children) in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_h((mat.color.h() + cursor.d.x).rem_euclid(360.));
                // mark change
                mark_changed!(-2, children, black_hole_query, white_hole_query);
            }
        }

        let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        let increment = if shift { 0.01 } else { -0.01 };
        if keyboard_input.pressed(KeyCode::Up) {
            for (id, _) in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_h((mat.color.h() + increment * 100.).rem_euclid(360.));
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for (id, _) in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_s((mat.color.s() + increment).rem_euclid(2.));
            }
        }
        if keyboard_input.pressed(KeyCode::Right) {
            for (id, _) in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_l((mat.color.l() + increment).rem_euclid(4.));
            }
        }
        if keyboard_input.pressed(KeyCode::Left) {
            for (id, _) in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_a((mat.color.a() + increment).rem_euclid(1.));
            }
        }
    }
}

fn update_radius(
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<(Entity, &Children, &Mesh2dHandle), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut radius_query: Query<&mut Radius>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<&BlackHole>,
) {
    if keyboard_input.pressed(KeyCode::Key3) {
        if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for (entity, children, Mesh2dHandle(id)) in mesh_ids.iter() {
                let r = cursor.f.distance(cursor.i);
                let mesh = meshes.get_mut(id).unwrap();
                *mesh = shape::Circle::new(r).into();
                radius_query.get_mut(entity).unwrap().0 = r;
                mark_changed!(-3, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Up) {
            for (entity, children, Mesh2dHandle(id)) in mesh_ids.iter() {
                let r = radius_query.get_mut(entity).unwrap().0 + 1.;
                radius_query.get_mut(entity).unwrap().0 = r;
                let mesh = meshes.get_mut(id).unwrap();
                *mesh = shape::Circle::new(r).into();
                mark_changed!(-3, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for (entity, children, Mesh2dHandle(id)) in mesh_ids.iter() {
                let r = radius_query.get_mut(entity).unwrap().0 - 1.;
                radius_query.get_mut(entity).unwrap().0 = r;
                let mesh = meshes.get_mut(id).unwrap();
                *mesh = shape::Circle::new(r).into();
                mark_changed!(-3, children, black_hole_query, white_hole_query);
            }
        }
    }
}

fn update_num(
    mut query: Query<(&mut Num, &Children), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<&BlackHole>,
) {
    if keyboard_input.pressed(KeyCode::Key4) {
        if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for (mut n, children) in query.iter_mut() {
                // change the number
                n.0 += cursor.d.y / 10.;
                // inform any white holes connected through link -4 black holes
                // that our value has changed
                mark_changed!(-4, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Up) {
            for (mut n, children) in query.iter_mut() {
                n.0 += 0.01;
                mark_changed!(-4, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for (mut n, children) in query.iter_mut() {
                n.0 -= 0.01;
                mark_changed!(-4, children, black_hole_query, white_hole_query);
            }
        }
    }
}

fn update_op(
    mut query: Query<&mut Op, With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::O) {
        for mut op in query.iter_mut() { op.0 -= 1; }
    }
    if keyboard_input.just_pressed(KeyCode::P) {
        for mut op in query.iter_mut() { op.0 += 1; }
    }
}

fn update_circle_text(
    mut query: Query<(&mut Text, &Parent), With<Visible>>,
    order_query: Query<&Order>,
    num_query: Query<&Num>,
    op_query: Query<&Op>,
) {
    for (mut text, parent) in query.iter_mut() {
        if let Ok(order) = order_query.get(**parent) {
            text.sections[1].value = "order: ".to_string() + &order.0.to_string() + "\n";
        }
        if let Ok(op) = op_query.get(**parent) {
            text.sections[2].value = match op.0 {
                -1 => "op: toTrans\n".to_string(),
                0 => "op: yaas\n".to_string(),
                1 => "op: BloomControl\n".to_string(),
                _ => op.0.to_string() + "\n",
            };
        }
        if let Ok(num) = num_query.get(**parent) {
            text.sections[3].value = num.0.to_string();
        }
    }
}

fn delete_selected(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<(Entity, &Children), With<Selected>>,
    mut commands: Commands,
    white_hole_query: Query<&WhiteHole>,
    black_hole_query: Query<&BlackHole>,
    mut order_change: EventWriter<OrderChange>,
) {
    if keyboard_input.pressed(KeyCode::Delete) {
        for (id, children) in query.iter() {
            // if the circle we're deleting is a connection
            if let Ok(black_hole) = black_hole_query.get(id) {
                commands.entity(black_hole.wh).despawn_recursive();
            } else if let Ok(white_hole) = white_hole_query.get(id) {
                commands.entity(white_hole.bh).despawn_recursive();
            } else {
                // not a connection, despawn the holes on the other side
                for child in children.iter() {
                    if let Ok(black_hole) = black_hole_query.get(*child) {
                        commands.entity(black_hole.wh).despawn_recursive();
                    }
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        commands.entity(white_hole.bh).despawn_recursive();
                    }
                }
            }
            commands.entity(id).despawn_recursive();
            order_change.send_default();
        }
    }
}

// ------------------- connections -------------------

// hole enum?
#[allow(dead_code)]
#[derive(Component)]
struct WhiteHole {
    id: Entity,
    parent: Entity,
    bh: Entity,
    link_type: i32,
    changed: bool,
}

#[derive(Component)]
struct BlackHole {
    id: Entity,
    parent: Entity,
    wh: Entity,
    link_type: i32,
}

fn connect(
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    query: Query<(Entity, &Radius, &Transform), (With<Visible>, With<Order>)>,
    cursor: Res<CursorInfo>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rad_query: Query<&Radius>,
    trans_query: Query<&Transform>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if mouse_button_input.just_released(MouseButton::Left) &&
    !keyboard_input.pressed(KeyCode::Space) {
        let mut source_entity: Option<Entity> = None;
        let mut sink_entity: Option<Entity> = None;
        for (e, r, t) in query.iter() {
            if cursor.i.distance(t.translation.xy()) < r.0 {
                source_entity = Some(e);
                continue;
            }
            if cursor.f.distance(t.translation.xy()) < r.0 {
                sink_entity = Some(e);
                continue;
            }
            if source_entity.is_some() && sink_entity.is_some() { break; }
        }

        if let (Some(src), Some(snk)) = (source_entity, sink_entity) {
            let src_radius = rad_query.get(src).unwrap().0;
            let snk_radius = rad_query.get(snk).unwrap().0;
            let src_trans = trans_query.get(src).unwrap().translation;
            let snk_trans = trans_query.get(snk).unwrap().translation;

            // spawn circles
            let black_hole = commands.spawn(( ColorMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(src_radius * 0.1).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::BLACK)),
                    transform: Transform::from_translation((cursor.i - src_trans.xy()).extend(0.000001)),
                    ..default()
                },
                Visible,
                Radius(src_radius * 0.1),
            )).id();
            let white_hole = commands.spawn(( ColorMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(snk_radius * 0.1).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE)),
                    transform: Transform::from_translation((cursor.f - snk_trans.xy()).extend(0.000001)),
                    ..default()
                },
                Visible,
                Radius(snk_radius * 0.1),
            )).id();

            // insert connection info
            commands.entity(black_hole).insert(
                BlackHole {
                    id: black_hole,
                    parent: src,
                    wh: white_hole,
                    link_type: 0,
                });
            commands.entity(white_hole).insert(
                WhiteHole {
                    id: white_hole,
                    parent: snk,
                    bh: black_hole,
                    link_type: 0,
                    changed: false,
                });

            // add to parents
            commands.entity(src).add_child(black_hole);
            commands.entity(snk).add_child(white_hole);

            // add link type text
            let black_hole_text = commands.spawn(Text2dBundle {
                text: Text::from_section(
                    0.to_string(),
                    TextStyle { color: Color::WHITE, ..default() },
                    ),
                transform: Transform::from_translation(Vec3{z:0.000001, ..default()}),
                ..default()
                }).id();
            commands.entity(black_hole).add_child(black_hole_text);

            let white_hole_text = commands.spawn(Text2dBundle {
                text: Text::from_section(
                    0.to_string(),
                    TextStyle { color: Color::BLACK, ..default() },
                    ),
                transform: Transform::from_translation(Vec3{z:0.000001, ..default()}),
                ..default()
                }).id();
            commands.entity(white_hole).add_child(white_hole_text);
        }
    }
}

fn draw_connections(
    mut gizmos: Gizmos,
    black_hole_query: Query<&BlackHole>,
    time: Res<Time>,
    trans_query: Query<&GlobalTransform>,
) {
    for black_hole in black_hole_query.iter() {
        let src_pos = trans_query.get(black_hole.id).unwrap().translation().xy();
        let snk_pos = trans_query.get(black_hole.wh).unwrap().translation().xy();
        let color = Color::hsl((time.elapsed_seconds() * 100.) % 360., 1.0, 0.5);
        gizmos.line_2d(src_pos, snk_pos, color);
    }
}

fn draw_connecting_line(
    mut gizmos: Gizmos,
    time: Res<Time>,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) &&
    !keyboard_input.pressed(KeyCode::Space) {
        let color = Color::hsl((time.elapsed_seconds() * 100.) % 360., 1.0, 0.5);
        gizmos.line_2d(cursor.i, cursor.f, color);
    }
}

fn update_link_type (
    keyboard_input: Res<Input<KeyCode>>,
    mut black_hole_query: Query<&mut BlackHole, With<Selected>>,
    mut white_hole_query: Query<&mut WhiteHole, With<Selected>>,
) {
    if keyboard_input.just_pressed(KeyCode::Period) {
        for mut hole in black_hole_query.iter_mut() { hole.link_type += 1; }
        for mut hole in white_hole_query.iter_mut() { hole.link_type += 1; }
    }
    if keyboard_input.just_pressed(KeyCode::Comma) {
        for mut hole in black_hole_query.iter_mut() { hole.link_type -= 1; }
        for mut hole in white_hole_query.iter_mut() { hole.link_type -= 1; }
    }
}

fn update_link_type_text(
    mut query: Query<(&mut Text, &Parent), With<Visible>>,
    black_hole_query: Query<&BlackHole>,
    white_hole_query: Query<&WhiteHole>,
) {
    for (mut text, parent) in query.iter_mut() {
        if let Ok(hole) = black_hole_query.get(**parent) {
            text.sections[0].value = hole.link_type.to_string();
        }
        if let Ok(hole) = white_hole_query.get(**parent) {
            text.sections[0].value = hole.link_type.to_string();
        }
    }
}


