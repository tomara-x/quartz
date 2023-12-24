use bevy::{
    prelude::*};

use crate::{circles::*, cursor::*};

pub struct ConnectionsPlugin;

impl Plugin for ConnectionsPlugin {
    fn build(&self, app: &mut App) { app
        .register_type::<ConnectionIds>()
        .register_type::<MaxUsedConnectionIndex>()
        .register_type::<WhiteHole>()
        .register_type::<BlackHole>()
        .init_resource::<ConnectionIds>()
        .init_resource::<MaxUsedConnectionIndex>()
        .add_systems(Update, connect.run_if(in_state(Mode::Connect)))
        //.add_systems(Update, update_connected_color)
        .add_systems(Update, draw_connections)
        .add_systems(Update, draw_connecting_line.run_if(in_state(Mode::Connect)))
        .add_systems(Update, update_link_type.run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_text.run_if(in_state(Mode::Edit)))
        ;
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ConnectionIds(pub Vec<Entity>);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct MaxUsedConnectionIndex(pub usize);

// hole enum?
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WhiteHole {
    pub id: Entity,
    pub index: usize,
    pub parent: Entity,
    pub parent_index: usize,
    pub bh: Entity,
    pub bh_index: usize,
    pub link_type: usize,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct BlackHole {
    pub id: Entity,
    pub index: usize,
    pub parent: Entity,
    pub parent_index: usize,
    pub wh: Entity,
    pub wh_index: usize,
    pub link_type: usize,
}

// i'll be back
impl Default for WhiteHole {
    fn default() -> Self {
        WhiteHole {
            id: Entity::from_raw(0),
            index: 0,
            parent: Entity::from_raw(0),
            parent_index: 0,
            bh: Entity::from_raw(0),
            bh_index: 0,
            link_type: 0,
        }
    }
}
impl Default for BlackHole {
    fn default() -> Self {
        BlackHole {
            id: Entity::from_raw(0),
            index: 0,
            parent: Entity::from_raw(0),
            parent_index: 0,
            wh: Entity::from_raw(0),
            wh_index: 0,
            link_type: 0,
        }
    }
}

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
    mut connection_ids: ResMut<ConnectionIds>,
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
                    index: max_connection_index.0,
                    parent: src,
                    parent_index: src_index,
                    wh: white_hole,
                    wh_index: max_connection_index.0 + 1,
                    link_type: 0,
                });
            commands.entity(white_hole).insert(
                WhiteHole {
                    id: white_hole,
                    index: max_connection_index.0 + 1,
                    parent: snk,
                    parent_index: snk_index,
                    bh: black_hole,
                    bh_index: max_connection_index.0,
                    link_type: 0,
                });

            // add to parents
            commands.entity(src).add_child(black_hole);
            commands.entity(snk).add_child(white_hole);

            // save ids
            connection_ids.0.push(white_hole);
            connection_ids.0.push(black_hole);
            max_connection_index.0 += 2;

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
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        let color = Color::hsl((time.elapsed_seconds() * 100.) % 360., 1.0, 0.5);
        gizmos.line_2d(cursor.i, cursor.f, color);
    }
}

fn update_link_type (
    keyboard_input: Res<Input<KeyCode>>,
    mut black_hole_query: Query<&mut BlackHole, With<Selected>>,
    mut white_hole_query: Query<&mut WhiteHole, With<Selected>>,
) {
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if shift && keyboard_input.just_pressed(KeyCode::Up) {
        for mut hole in black_hole_query.iter_mut() {
            hole.link_type = hole.link_type.saturating_add(1);
        }
        for mut hole in white_hole_query.iter_mut() {
            hole.link_type = hole.link_type.saturating_add(1);
        }
    }
    if shift && keyboard_input.just_pressed(KeyCode::Down) {
        for mut hole in black_hole_query.iter_mut() {
            hole.link_type = hole.link_type.saturating_sub(1);
        }
        for mut hole in white_hole_query.iter_mut() {
            hole.link_type = hole.link_type.saturating_sub(1);
        }
    }
}

fn update_text(
    mut query: Query<(&mut Text, &Parent), With<Visible>>,
    keyboard_input: Res<Input<KeyCode>>,
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

