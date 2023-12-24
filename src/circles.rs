use bevy::{
    render::view::VisibleEntities,
    sprite::Mesh2dHandle,
    prelude::*};

use crate::{cursor::*, connections::*};

pub struct CirclesPlugin;

impl Plugin for CirclesPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<Mode>();

        app.register_type::<Radius>();

        app.register_type::<Selected>();
        app.register_type::<Visible>();

        app.register_type::<Depth>();
        app.register_type::<Index>();
        app.register_type::<Order>();
        app.register_type::<Num>();
        app.register_type::<Arr>();
        app.register_type::<Offset>();
        app.register_type::<CircleIds>();
        app.register_type::<MaxUsedIndex>();

        // test high depth
        app.insert_resource(Depth(-10.));
        app.init_resource::<CircleIds>();
        app.init_resource::<MaxUsedIndex>();

        app.add_systems(Update, spawn_circles.run_if(in_state(Mode::Draw)));
        app.add_systems(Update, draw_pointer_circle.run_if(not(in_state(Mode::Connect))));
        app.add_systems(Update, mark_visible.after(update_cursor_info));
        app.add_systems(Update, update_selection.after(mark_visible).run_if(in_state(Mode::Edit)));
        app.add_systems(Update, move_selected.after(update_selection).run_if(in_state(Mode::Edit)));
        app.add_systems(Update, update_color.after(update_selection).run_if(in_state(Mode::Edit)));
        app.add_systems(Update, update_radius.after(update_selection).run_if(in_state(Mode::Edit)));
        app.add_systems(Update, highlight_selected.run_if(in_state(Mode::Edit)));
        app.add_systems(Update, delete_selected.run_if(in_state(Mode::Edit)));
        app.add_systems(Update, update_order.run_if(in_state(Mode::Edit)));
        app.add_systems(Update, update_order_text.run_if(in_state(Mode::Edit)).after(update_order));
        app.add_systems(Update, switch_mode);
    }
}

#[derive(Component, Reflect)]
pub struct Num(f32);

#[derive(Component, Reflect)]
pub struct Arr(Vec<f32>);

#[derive(Component, Reflect)]
pub struct Offset {
    trans: Vec3,
    color: Color,
    radius: f32,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct CircleIds(pub Vec<Entity>);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct MaxUsedIndex(pub usize);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
struct Depth(f32);

#[derive(Component, Reflect)]
pub struct Radius(pub f32);

#[derive(Component, Reflect)]
pub struct Selected;

#[derive(Component, Reflect)]
pub struct Visible;

#[derive(Component, Reflect)]
pub struct Index(pub usize);

#[derive(Component, Reflect)]
pub struct Order(pub usize);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum Mode {
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

fn spawn_circles(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut depth: ResMut<Depth>,
    cursor: Res<CursorInfo>,
    mut index: ResMut<MaxUsedIndex>,
    mut circle_ids: ResMut<CircleIds>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
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
            Index(index.0),
            Order(0),
            Num(0.),
            Arr(Vec::new()),
            Offset {trans:Vec3::ZERO, color:Color::BLACK, radius:0.},
        )).id();

        // have the circle adopt a text entity
        let text = commands.spawn(Text2dBundle {
            text: Text::from_sections([
                TextSection::new(
                    index.0.to_string() + "\n",
                    TextStyle::default(),
                ),
                TextSection::new(
                    0.to_string(),
                    TextStyle::default()
                )
            ]),
            transform: Transform::from_translation(Vec3{z:0.000001, ..default()}),
            ..default()
        }).id();
        commands.entity(id).add_child(text);

        circle_ids.0.push(id);
        index.0 += 1;
        depth.0 += 0.00001;
    }
}

fn draw_pointer_circle(
    cursor: Res<CursorInfo>,
    mut gizmos: Gizmos,
    time: Res<Time>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
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
    mut query: Query<&mut Transform, With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.pressed(KeyCode::Key1) {
        if mouse_button_input.pressed(MouseButton::Left) &&
        //lol because the update to entities isn't read until the next frame
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for mut t in query.iter_mut() {
                t.translation.x += cursor.d.x;
                t.translation.y += cursor.d.y;
            }
        }
        if keyboard_input.pressed(KeyCode::Up) {
            for mut t in query.iter_mut() { t.translation.y += 1.; }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for mut t in query.iter_mut() { t.translation.y -= 1.; }
        }
        if keyboard_input.pressed(KeyCode::Right) {
            for mut t in query.iter_mut() { t.translation.x += 1.; }
        }
        if keyboard_input.pressed(KeyCode::Left) {
            for mut t in query.iter_mut() { t.translation.x -= 1.; }
        }
    }
}

fn update_color(
    mut mats: ResMut<Assets<ColorMaterial>>,
    material_ids: Query<&Handle<ColorMaterial>, With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    if keyboard_input.pressed(KeyCode::Key2) {
        if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for id in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_h((mat.color.h() + cursor.d.x).rem_euclid(360.));
            }
        }

        let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        let increment = if shift { 0.01 } else { -0.01 };
        if keyboard_input.pressed(KeyCode::Up) {
            for id in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_h((mat.color.h() + increment * 100.).rem_euclid(360.));
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for id in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_s((mat.color.s() + increment).rem_euclid(2.));
            }
        }
        if keyboard_input.pressed(KeyCode::Right) {
            for id in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_l((mat.color.l() + increment).rem_euclid(4.));
            }
        }
        if keyboard_input.pressed(KeyCode::Left) {
            for id in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_a((mat.color.a() + increment).rem_euclid(1.));
            }
        }
    }
}

fn update_radius(
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<(Entity, &Mesh2dHandle), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut radius_query: Query<&mut Radius>,
) {
    if keyboard_input.pressed(KeyCode::Key3) {
        if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for (entity, Mesh2dHandle(id)) in mesh_ids.iter() {
                let r = cursor.f.distance(cursor.i);
                let mesh = meshes.get_mut(id).unwrap();
                *mesh = shape::Circle::new(r).into();
                radius_query.get_mut(entity).unwrap().0 = r;
            }
        }
        if keyboard_input.pressed(KeyCode::Up) {
            for (entity, Mesh2dHandle(id)) in mesh_ids.iter() {
                let r = radius_query.get_mut(entity).unwrap().0 + 1.;
                radius_query.get_mut(entity).unwrap().0 = r;
                let mesh = meshes.get_mut(id).unwrap();
                *mesh = shape::Circle::new(r).into();
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for (entity, Mesh2dHandle(id)) in mesh_ids.iter() {
                let r = radius_query.get_mut(entity).unwrap().0 - 1.;
                radius_query.get_mut(entity).unwrap().0 = r;
                let mesh = meshes.get_mut(id).unwrap();
                *mesh = shape::Circle::new(r).into();
            }
        }
    }
}

fn update_order (
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Order, With<Selected>>,
) {
    if keyboard_input.pressed(KeyCode::Key4) {
        if keyboard_input.just_pressed(KeyCode::Up) {
            for mut order in query.iter_mut() { order.0 += 1; }
        }
        if keyboard_input.just_pressed(KeyCode::Down) {
            for mut order in query.iter_mut() { if order.0 > 0 { order.0 -= 1; } }
        }
    }
}

fn update_order_text(
    mut query: Query<(&mut Text, &Parent), With<Visible>>,
    order_query: Query<&Order>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.any_just_pressed([KeyCode::Up, KeyCode::Down]) {
        for (mut text, parent) in query.iter_mut() {
            if let Ok(order) = order_query.get(**parent) {
                text.sections[1].value = order.0.to_string();
            }
        }
    }
}

fn delete_selected(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<(Entity, &Children), With<Selected>>,
    mut commands: Commands,
    white_hole_query: Query<&WhiteHole>,
    black_hole_query: Query<&BlackHole>,
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
        }
    }
}

