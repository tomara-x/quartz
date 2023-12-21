use bevy::{
    prelude::*};

use crate::{circles::*, cursor::*};

pub struct ConnectionsPlugin;

impl Plugin for ConnectionsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ConnectionIndices>();
        app.register_type::<MaxUsedConnectionIndex>();
        app.register_type::<WhiteHole>();
        app.register_type::<BlackHole>();
        app.register_type::<ConnectionType>();
        app.init_resource::<ConnectionIndices>();
        app.init_resource::<MaxUsedConnectionIndex>();
        app.add_systems(Update, connect.run_if(in_state(Mode::Connect)));
        //app.add_systems(Update, update_connected_color);
        app.add_systems(Update, draw_connections);
        app.add_systems(Update, draw_connecting_line.run_if(in_state(Mode::Connect)));
        app.add_systems(Update, update_connection_type.run_if(in_state(Mode::Edit)));
        app.add_systems(Update, update_connection_type_text
                                    .run_if(in_state(Mode::Edit))
                                    .after(update_connection_type));
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ConnectionIndices(pub Vec<Entity>);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct MaxUsedConnectionIndex(pub usize);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WhiteHole {
    pub index: usize,
    pub parent: usize,
    pub black_hole: usize,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct BlackHole {
    pub index: usize,
    pub parent: usize,
    pub white_hole: usize,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ConnectionType(usize);

fn connect(
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    query: Query<(Entity, &Radius, &Transform), (With<Visible>, With<Order>)>,
    index_query: Query<&Index>,
    cursor: Res<CursorInfo>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rad_query: Query<&Radius>,
    trans_query: Query<&Transform>,
    mut connection_indices: ResMut<ConnectionIndices>,
    mut max_connection_index: ResMut<MaxUsedConnectionIndex>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
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
            let src_index = index_query.get(src).unwrap().0;
            let snk_index = index_query.get(snk).unwrap().0;
            let src_radius = rad_query.get(src).unwrap().0;
            let snk_radius = rad_query.get(snk).unwrap().0;
            let src_trans = trans_query.get(src).unwrap().translation;
            let snk_trans = trans_query.get(snk).unwrap().translation;

            // spawn connection circles
            let black_hole = commands.spawn(( ColorMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(src_radius * 0.1).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::rgb(0.,0.,0.))),
                    transform: Transform::from_translation((cursor.i - src_trans.xy()).extend(0.000001)),
                    ..default()
                },
                Visible,
                Radius(src_radius * 0.1),
                BlackHole {
                    index: max_connection_index.0,
                    parent: src_index,
                    white_hole: max_connection_index.0 + 1,
                },
                ConnectionType(0),
            )).id();
            commands.entity(src).add_child(black_hole);

            connection_indices.0.push(black_hole);
            max_connection_index.0 += 1;

            let white_hole = commands.spawn(( ColorMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(snk_radius * 0.1).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::rgb(1.,1.,1.))),
                    transform: Transform::from_translation((cursor.f - snk_trans.xy()).extend(0.000001)),
                    ..default()
                },
                Visible,
                Radius(snk_radius * 0.1),
                WhiteHole {
                    index: max_connection_index.0,
                    parent: snk_index,
                    black_hole: max_connection_index.0 - 1,
                },
                ConnectionType(0),
            )).id();
            commands.entity(snk).add_child(white_hole);

            connection_indices.0.push(white_hole);
            max_connection_index.0 += 1;

            // give them connection type text
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

            // order
            //let src_order = order_query.get(src).unwrap().0;
            //order_query.get_mut(snk).unwrap().0 = src_order + 1;
        }
    }
}

fn draw_connections(
    mut gizmos: Gizmos,
    black_hole_query: Query<&BlackHole>,
    time: Res<Time>,
    trans_query: Query<&GlobalTransform>,
    connection_indices: Res<ConnectionIndices>
) {
    for blackhole in black_hole_query.iter() {
        let src_id = connection_indices.0[blackhole.index];
        let snk_id = connection_indices.0[blackhole.white_hole];
        let src_pos = trans_query.get(src_id).unwrap().translation().xy();
        let snk_pos = trans_query.get(snk_id).unwrap().translation().xy();
        let color = Color::hsl((time.elapsed_seconds() * 100.) % 360., 1.0, 0.5);
        gizmos.line_2d(src_pos, snk_pos, color);
    }
}

fn draw_connecting_line(
    mut gizmos: Gizmos,
    time: Res<Time>,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        let color = Color::hsl((time.elapsed_seconds() * 100.) % 360., 1.0, 0.5);
        gizmos.line_2d(cursor.i, cursor.f, color);
    }
}

fn update_connection_type (
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut ConnectionType, With<Selected>>,
) {
    if keyboard_input.pressed(KeyCode::Key5) {
        if keyboard_input.just_pressed(KeyCode::Up) {
            for mut connection in query.iter_mut() { connection.0 += 1; }
        }
        if keyboard_input.just_pressed(KeyCode::Down) {
            for mut connection in query.iter_mut() { if connection.0 > 0 { connection.0 -= 1; } }
        }
    }
}

fn update_connection_type_text(
    mut query: Query<(&mut Text, &Parent), With<Visible>>,
    connection_type_query: Query<&ConnectionType>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.any_just_pressed([KeyCode::Up, KeyCode::Down]) {
        for (mut text, parent) in query.iter_mut() {
            if let Ok(connection) = connection_type_query.get(**parent) {
                text.sections[0].value = connection.0.to_string();
            }
        }
    }
}

//fn update_connected_color(
//    mouse_button_input: Res<Input<MouseButton>>,
//    inputs_query: Query<(Entity, &Inputs)>,
//    entity_indices: Res<EntityIndices>,
//    material_ids: Query<&Handle<ColorMaterial>>,
//    mut mats: ResMut<Assets<ColorMaterial>>,
//) {
//    if mouse_button_input.pressed(MouseButton::Left) {
//        for (entity, inputs) in inputs_query.iter() {
//            //the first input's first field (entity index)
//            //then we find that entity id from the resource
//            if let Some(input) = inputs.0.get(0) {
//                let src_entity = entity_indices.0[input.0];
//                let src_mat = mats.get(material_ids.get(src_entity).unwrap()).unwrap();
//                let src_color = src_mat.color;
//                let snk_mat = mats.get_mut(material_ids.get(entity).unwrap()).unwrap();
//                snk_mat.color = src_color;
//            }
//        }
//    }
//}

