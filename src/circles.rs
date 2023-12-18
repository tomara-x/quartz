use bevy::{
    render::view::VisibleEntities,
    sprite::Mesh2dHandle,
    prelude::*};

use crate::{cursor::*};

pub struct CirclesPlugin;

impl Plugin for CirclesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Depth>();
        app.register_type::<Radius>();
        app.register_type::<Pos>();
        app.register_type::<Selected>();
        app.register_type::<Visible>();
        app.register_type::<Index>();
        app.register_type::<EntityIndices>();
        app.register_type::<Inputs>();
        app.register_type::<Outputs>();
        app.register_type::<Num>();
        app.register_type::<Arr>();
        app.register_type::<Order>();
        app.insert_resource(Depth(-10.));
        app.insert_resource(EntityIndices(Vec::new()));
        app.add_systems(Update, spawn_circles);
        app.add_systems(Update, update_color);
        app.add_systems(Update, update_radius);
        app.add_systems(Update, draw_pointer_circle);
        app.add_systems(Update, mark_visible.after(update_cursor_info));
        app.add_systems(Update, update_selection.after(mark_visible));
        app.add_systems(Update, highlight_selected);
        app.add_systems(Update, move_selected.after(update_selection));
        app.add_systems(Update, delete_selected);
        app.add_systems(Update, (connect, draw_connections));
        app.add_systems(Update, update_text);
    }
}


#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct EntityIndices(pub Vec<Entity>);

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
struct Order(i32);


fn spawn_circles(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut depth: ResMut<Depth>,
    cursor: Res<CursorInfo>,
    mut index: Local<usize>,
    mut entity_indices: ResMut<EntityIndices>,
) {
    if mouse_button_input.just_released(MouseButton::Left) && keyboard_input.pressed(KeyCode::Z) {
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
            Index(*index),
            Order(0),
        )).id();

        // have the circle adopt a text entity
        let text = commands.spawn(Text2dBundle {
            text: Text::from_sections([
                TextSection::new(
                    index.to_string(),
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
        *index += 1;
        depth.0 += 0.00001;
    }
}

// for debugging
fn update_text(
    query: Query<(Entity, &Children), With<Visible>>,
    order_query: Query<&Order>,
    mut text_query: Query<&mut Text>,
) {
    for (entity, children) in query.iter() {
        for child in children {
            text_query.get_mut(*child).unwrap().sections[1].value = order_query.get(entity).unwrap().0.to_string();
        }
    }
}


//need to make this conditional
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
    query: Query<(&Radius, &Transform), With<Selected>>,
) {
    for (r, p) in query.iter() {
        gizmos.circle_2d(p.translation.xy(), r.0, Color::BLUE).segments(64);
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
    query: Query<(Entity, &Radius, &Pos), Or<(With<Visible>, With<Selected>)>>,
    selected: Query<Entity, With<Selected>>,
    selected_query: Query<&Selected>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<Input<KeyCode>>,
    mut clicked_on_circle: Local<bool>,
) {
    let none_selected = selected.is_empty();
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    if mouse_button_input.just_pressed(MouseButton::Left) {
        for (e, r, p) in query.iter() {
            if cursor.i.distance(p.0.xy()) < r.0 {
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
                for (e, r, p) in query.iter() {
                    if cursor.i.distance(p.0.xy()) < r.0 {
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
            for (e, r, p) in query.iter() {
                if cursor.i.distance(cursor.f) + r.0 > cursor.i.distance(p.0.xy()) {
                    commands.entity(e).insert(Selected);
                }
            }
        }
    }
}

// add arrow key interaction for those 3?
// move the selected entities by changing the translation of entity directly
// when mouse is released we store the translation in temporary position component
fn move_selected(
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
    mut query: Query<(&mut Transform, &mut Pos), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) && keyboard_input.pressed(KeyCode::X) {
        for (mut t, p) in query.iter_mut() {
            t.translation = (p.0.xy() + cursor.f - cursor.i).extend(p.0.z);
            //t.translation.x = p.0.x + cursor.f.x - cursor.i.x;
            //t.translation.y = p.0.y + cursor.f.y - cursor.i.y;
        }
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        for (t, mut p) in query.iter_mut() {
            p.0 = t.translation;
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
    if mouse_button_input.pressed(MouseButton::Left) && keyboard_input.pressed(KeyCode::C) {
        for id in material_ids.iter() {
            let mat = mats.get_mut(id).unwrap();
            mat.color = Color::hsl(cursor.i.distance(cursor.f)%360., 1.0, 0.5);
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
    if mouse_button_input.pressed(MouseButton::Left) && keyboard_input.pressed(KeyCode::V) {
        for (entity, Mesh2dHandle(id)) in mesh_ids.iter() {
            let r = cursor.f.distance(cursor.i);
            let mesh = meshes.get_mut(id).unwrap();
            *mesh = shape::Circle::new(r).into();
            radius_query.get_mut(entity).unwrap().0 = r;
        }
    }
}

fn delete_selected(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<(Entity, &Index), With<Selected>>,
    mut inputs_query: Query<&mut Inputs>,
    mut outputs_query: Query<&mut Outputs>,
    entity_indices: Res<EntityIndices>,
    mut commands: Commands,
) {
    if keyboard_input.pressed(KeyCode::Delete) {
        for (id, index) in query.iter() {
            if let Ok(inputs) = inputs_query.get(id) {
                for src in &inputs.0 {
                    let src_outputs = &mut outputs_query.get_mut(entity_indices.0[*src]).unwrap().0;
                    src_outputs.retain(|&x| x != index.0);
                }
            }
            if let Ok(outputs) = outputs_query.get(id) {
                for snk in &outputs.0 {
                    let snk_inputs = &mut inputs_query.get_mut(entity_indices.0[*snk]).unwrap().0;
                    snk_inputs.retain(|&x| x != index.0);
                }
            }
            commands.entity(id).despawn_recursive();
        }
    }
}


//-----------------------connections-----------------------

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Inputs(Vec<usize>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Outputs(Vec<usize>);

fn connect(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    query: Query<(Entity, &Radius, &Pos), With<Visible>>,
    index_query: Query<&Index>,
    mut inputs_query: Query<&mut Inputs>,
    mut outputs_query: Query<&mut Outputs>,
    mut order_query: Query<&mut Order>,
    cursor: Res<CursorInfo>,
) {
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if ctrl && mouse_button_input.just_released(MouseButton::Left) {
        let mut source_entity: Option<Entity> = None;
        let mut sink_entity: Option<Entity> = None;
        for (e, r, p) in query.iter() {
            if cursor.i.distance(p.0.xy()) < r.0 { source_entity = Some(e) };
            if cursor.f.distance(p.0.xy()) < r.0 { sink_entity = Some(e) };
        }

        if let Some(src) = source_entity {
            if let Some(snk) = sink_entity {
                let src_index = index_query.get(src).unwrap().0;
                let snk_index = index_query.get(snk).unwrap().0;
                // source has outputs (we push to its outputs vector)
                if let Ok(mut outputs) = outputs_query.get_mut(src) {
                    outputs.0.push(snk_index);
                }
                else {
                    commands.entity(src).insert(Outputs(vec![snk_index]));
                }
                if let Ok(mut inputs) = inputs_query.get_mut(snk) {
                    inputs.0.push(src_index);
                }
                else {
                    commands.entity(snk).insert(Inputs(vec![src_index]));
                }

                // order
                let src_order = order_query.get(src).unwrap().0;
                order_query.get_mut(snk).unwrap().0 = src_order + 1;
            }
        }
    }
}

fn draw_connections(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Inputs), With<Visible>>,
    pos_query: Query<&Transform>,
    entity_indices: Res<EntityIndices>,
) {
    for (pos, inputs) in query.iter() {
        for input in &inputs.0 {
            let src_pos = pos_query.get(entity_indices.0[*input]).unwrap();
            gizmos.line_2d(pos.translation.xy(), src_pos.translation.xy(), Color::BLUE);
        }
    }
}


// filtering components to determin behavior of connection
#[derive(Component)]
struct ReadColor;
#[derive(Component)]
struct WriteColor;

#[derive(Component)]
struct ReadPos;
#[derive(Component)]
struct WritePos;

#[derive(Component)]
struct ReadRadius;
#[derive(Component)]
struct WriteRadius;

#[derive(Component)]
struct ReadG0;
#[derive(Component)]
struct WriteG0;

#[derive(Component)]
struct ReadG1;
#[derive(Component)]
struct WriteG2;

#[derive(Component)]
struct ReadG3;
#[derive(Component)]
struct WriteG3;

#[derive(Component)]
struct ReadG4;
#[derive(Component)]
struct WriteG4;



//-------------------------added-components---------------------------

#[derive(Component, Reflect)]
struct Num(f32);

#[derive(Component, Reflect)]
struct Arr(Vec<f32>);

#[derive(Component, Reflect)]
struct ColorOffset(Color);

#[derive(Component, Reflect)]
struct PosOffset(Vec3);

