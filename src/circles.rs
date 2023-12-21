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
        app.register_type::<Pos>();

        app.register_type::<Selected>();
        app.register_type::<Visible>();

        app.register_type::<Depth>();
        app.register_type::<Index>();
        app.register_type::<Order>();
        app.register_type::<EntityIndices>();
        app.register_type::<MaxUsedIndex>();

        // test high depth
        app.insert_resource(Depth(-10.));
        app.init_resource::<EntityIndices>();
        app.init_resource::<MaxUsedIndex>();

        app.add_systems(Update, spawn_circles.run_if(in_state(Mode::Draw)));
        app.add_systems(Update, update_color.run_if(in_state(Mode::Edit)));
        app.add_systems(Update, update_radius.run_if(in_state(Mode::Edit)));
        app.add_systems(Update, draw_pointer_circle.run_if(not(in_state(Mode::Connect))));
        app.add_systems(Update, mark_visible.after(update_cursor_info));
        app.add_systems(Update, update_selection.after(mark_visible).run_if(in_state(Mode::Edit)));
        app.add_systems(Update, move_selected.after(update_selection).run_if(in_state(Mode::Edit)));
        app.add_systems(Update, highlight_selected.run_if(in_state(Mode::Edit)));
        app.add_systems(Update, delete_selected.run_if(in_state(Mode::Edit)));
        app.add_systems(Update, update_order.run_if(in_state(Mode::Edit)));
        app.add_systems(Update, update_order_text.run_if(in_state(Mode::Edit)).after(update_order));
        app.add_systems(Update, switch_mode);
    }
}


#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct EntityIndices(pub Vec<Entity>);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct MaxUsedIndex(pub usize);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
struct Depth(f32);

#[derive(Component, Reflect)]
pub struct Radius(pub f32);

#[derive(Component, Reflect)]
pub struct Pos(pub Vec3);

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
    mut entity_indices: ResMut<EntityIndices>,
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
            Pos(cursor.i.extend(depth.0)), //keeps track of initial position while moving
            Visible, //otherwise it can't be selected til after mark_visible is updated
            Index(index.0),
            Order(0),
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

        entity_indices.0.push(id);
        index.0 += 1;
        depth.0 += 0.00001;
    }
}

// better way to do this?
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
        for e in &vis.entities {
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
    mut clicked_on_circle: Local<bool>,
) {
    let none_selected = selected.is_empty();
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    if mouse_button_input.just_pressed(MouseButton::Left) {
        for (e, r, t) in query.iter() {
            if cursor.i.distance(t.translation().xy()) < r.0 {
                *clicked_on_circle = true;
                // we clicked a circle that wasn't already selected
                if !selected_query.contains(e) {
                    if shift { commands.entity(e).insert(Selected); }
                    else {
                        // deselect everything
                        for entity in selected.iter() {
                            commands.entity(entity).remove::<Selected>();
                        }
                        commands.entity(e).insert(Selected);
                    }
                }
                break;
            }
            *clicked_on_circle = false;
        }
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        if *clicked_on_circle {
            // some entities are selected and we just clicked (no drag) on one
            if !shift && !none_selected && cursor.i.distance(cursor.f) < 0.01 {
                for entity in selected.iter() {
                    commands.entity(entity).remove::<Selected>();
                }
                for (e, r, t) in query.iter() {
                    if cursor.i.distance(t.translation().xy()) < r.0 {
                        commands.entity(e).insert(Selected);
                        break;
                    }
                }
            }
        }
        else {
            // deselect everything
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
                // need it to be locking tho, so you can go back and forth
                if cursor.f.x - cursor.i.x > 0. {
                    mat.color.set_h((mat.color.h() + cursor.d.x) % 360.);
                }
                if cursor.f.x - cursor.i.x < 0. {
                    mat.color.set_s((mat.color.s() - (cursor.d.x / 100.)) % 1.);
                }
                if cursor.f.y - cursor.i.y > 0. {
                    mat.color.set_l((mat.color.l() + (cursor.d.y / 100.)) % 1.);
                }
                if cursor.f.y - cursor.i.y < 0. {
                    mat.color.set_a((mat.color.a() - (cursor.d.y / 100.)) % 1.);
                }
            }
        }
        if keyboard_input.pressed(KeyCode::Up) {
            for id in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_h((mat.color.h() + 1.) % 360.);
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for id in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_s((mat.color.s() + 0.01) % 2.);
            }
        }
        if keyboard_input.pressed(KeyCode::Right) {
            for id in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_l((mat.color.l() + 0.01) % 4.);
            }
        }
        if keyboard_input.pressed(KeyCode::Left) {
            for id in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_a((mat.color.a() + 0.01) % 1.);
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

fn delete_selected(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<(Entity, &Index, &Children), With<Selected>>,
    entity_indices: Res<EntityIndices>,
    mut commands: Commands,
    white_hole_query: Query<&WhiteHole>,
    black_hole_query: Query<&BlackHole>,
) {
    if keyboard_input.pressed(KeyCode::Delete) {
        for (id, index, children) in query.iter() {
            // delete any connections to the entity being deleted
            //if let Ok(inputs) = inputs_query.get(id) {
            //    for (src, _, _, _) in &inputs.0 {
            //        let src_outputs = &mut outputs_query.get_mut(entity_indices.0[*src]).unwrap().0;
            //        src_outputs.retain(|&x| x != index.0);
            //    }
            //}
            //if let Ok(outputs) = outputs_query.get(id) {
            //    for snk in &outputs.0 {
            //        let snk_inputs = &mut inputs_query.get_mut(entity_indices.0[*snk]).unwrap().0;
            //        snk_inputs.retain(|&x| x.0 != index.0);
            //    }
            //}
            // despawn corresponding black/white holes
            //for child in children.iter() {
            //    if let Ok(i) = white_hole_query.get(*child) {
            //        commands.entity(i.0).despawn();
            //    }
            //    if let Ok(i) = black_hole_query.get(*child) {
            //        commands.entity(i.0).despawn();
            //    }
            //}
            commands.entity(id).despawn_recursive();
        }
    }
}

