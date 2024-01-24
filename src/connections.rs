use bevy::{
    prelude::*,
    sprite::Mesh2dHandle,
};

use crate::components::*;

pub fn connect(
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    query: Query<(Entity, &Radius, &GlobalTransform), (With<Visible>, With<Order>)>,
    cursor: Res<CursorInfo>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rad_query: Query<&Radius>,
    trans_query: Query<&GlobalTransform>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if mouse_button_input.just_released(MouseButton::Left) &&
    !keyboard_input.pressed(KeyCode::Space) {
        let mut source_entity: (Option<Entity>, f32) = (None, f32::MIN);
        let mut sink_entity: (Option<Entity>, f32) = (None, f32::MIN);
        for (e, r, t) in query.iter() {
            if cursor.i.distance(t.translation().xy()) < r.0
                && t.translation().z > source_entity.1 {
                source_entity = (Some(e), t.translation().z);
            }
            if cursor.f.distance(t.translation().xy()) < r.0
                && t.translation().z > sink_entity.1 {
                sink_entity = (Some(e), t.translation().z);
            }
        }

        if let (Some(src), Some(snk)) = (source_entity.0, sink_entity.0) {
            if source_entity.0 == sink_entity.0 { return; }
            let src_radius = rad_query.get(src).unwrap().0;
            let snk_radius = rad_query.get(snk).unwrap().0;
            let src_trans = trans_query.get(src).unwrap().translation();
            let snk_trans = trans_query.get(snk).unwrap().translation();

            // spawn circles
            let black_hole = commands.spawn(( ColorMesh2dBundle {
                    mesh: meshes.add(shape::Circle { radius: src_radius * 0.15, vertices: 6 }.into()).into(),
                    material: materials.add(ColorMaterial::from(Color::BLACK)),
                    transform: Transform::from_translation((cursor.i - src_trans.xy()).extend(0.000001)),
                    ..default()
                },
                Visible,
                Radius(src_radius * 0.15),
                Col(Color::BLACK),
                Vertices(6),
                Save,
            )).id();
            let white_hole = commands.spawn(( ColorMesh2dBundle {
                    mesh: meshes.add(shape::Circle { radius: snk_radius * 0.15, vertices: 6 }.into()).into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE)),
                    transform: Transform::from_translation((cursor.f - snk_trans.xy()).extend(0.000001)),
                    ..default()
                },
                Visible,
                Radius(snk_radius * 0.15),
                Col(Color::WHITE),
                Vertices(6),
                WhiteHole {
                    bh_parent: src,
                    bh: black_hole,
                    link_types: (0, 0),
                    new_lt: true,
                    open: true,
                },
                Save,
            )).id();

            // insert black hole white hole
            commands.entity(black_hole).insert(
                BlackHole {
                    wh: white_hole,
                });
                
            // add to parents
            commands.entity(src).add_child(black_hole);
            commands.entity(snk).add_child(white_hole);

            // add link type text
            let black_hole_text = commands.spawn((Text2dBundle {
                text: Text::from_section(
                    0.to_string(),
                    TextStyle { color: Color::WHITE, ..default() },
                    ),
                transform: Transform::from_translation(Vec3{z:0.000001, ..default()}),
                ..default()
                },
                Save,
                )).id();
            commands.entity(black_hole).add_child(black_hole_text);

            let white_hole_text = commands.spawn((Text2dBundle {
                text: Text::from_section(
                    0.to_string(),
                    TextStyle { color: Color::BLACK, ..default() },
                    ),
                transform: Transform::from_translation(Vec3{z:0.000001, ..default()}),
                ..default()
                },
                Save,
                )).id();
            commands.entity(white_hole).add_child(white_hole_text);
        }
    }
}

pub fn draw_connections(
    mut gizmos: Gizmos,
    black_hole_query: Query<(Entity, &BlackHole)>,
    time: Res<Time>,
    trans_query: Query<&GlobalTransform>,
) {
    for (id, black_hole) in black_hole_query.iter() {
        let src_pos = trans_query.get(id).unwrap().translation().xy();
        if let Ok(wh) = trans_query.get(black_hole.wh) {
            let snk_pos = wh.translation().xy();
            let color = Color::hsl((time.elapsed_seconds() * 100.) % 360., 1.0, 0.5);
            gizmos.line_2d(src_pos, snk_pos, color);
        }
    }
}

pub fn draw_connecting_line(
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<Input<KeyCode>>,
    id: Res<ConnectingLine>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
) {
    if mouse_button_input.pressed(MouseButton::Left)
    && !mouse_button_input.just_pressed(MouseButton::Left)
    && !keyboard_input.pressed(KeyCode::Space) {
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = Tri { i: cursor.i, f: cursor.f } .into();
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = shape::Circle { radius: 0., vertices: 3 }.into();
    }
}

pub fn update_link_type_text(
    mut query: Query<(&mut Text, &Parent), With<Visible>>,
    black_hole_query: Query<&BlackHole>,
    white_hole_query: Query<&WhiteHole>,
) {
    for (mut text, parent) in query.iter_mut() {
        if let Ok(hole) = black_hole_query.get(**parent) {
            if let Ok(wh) = white_hole_query.get(hole.wh) {
                text.sections[0].value = lt_to_string(wh.link_types.0);
            }
        }
        if let Ok(hole) = white_hole_query.get(**parent) {
            text.sections[0].value = lt_to_string(hole.link_types.1);
        }
    }
}
