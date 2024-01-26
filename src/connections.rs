use bevy::{
    prelude::*,
    sprite::Mesh2dHandle,
    render::primitives::Aabb
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

            // spawn connection arrow
            let arrow = commands.spawn(
                ColorMesh2dBundle {
                    mesh: meshes.add(Tri {
                        i: cursor.i,
                        f: cursor.f,
                        ip: src_radius * 0.15,
                        fp: snk_radius * 0.15,
                        b: 2.
                    }.into()).into(),
                    material: materials.add(ColorMaterial::from(Color::hsla(0., 1., 1., 0.7))),
                    transform: Transform::from_translation(Vec3::Z),
                    ..default()
                }
            ).id();
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
                ConnectionArrow(arrow),
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

pub fn update_connection_arrows(
    bh_query: Query<(Entity, &BlackHole), Or<(Changed<Transform>, Changed<Radius>)>>,
    wh_query: Query<(Entity, &WhiteHole), Or<(Changed<Transform>, Changed<Radius>)>>,
    trans_query: Query<&GlobalTransform>,
    radius_query: Query<&Radius>,
    arrow_query: Query<&ConnectionArrow>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
    mut aabb_query: Query<&mut Aabb>,
) {
    for (id, bh) in bh_query.iter() {
        let i = trans_query.get(id).unwrap().translation().xy();
        let f = trans_query.get(bh.wh).unwrap().translation().xy();
        let ip = radius_query.get(id).unwrap().0;
        let fp = radius_query.get(bh.wh).unwrap().0;
        if let Ok(arrow_id) = arrow_query.get(bh.wh) {
            let aabb = Aabb::enclosing([i.extend(1.), f.extend(1.)]).unwrap();
            *aabb_query.get_mut(arrow_id.0).unwrap() = aabb;
            let Mesh2dHandle(mesh_id) = mesh_ids.get(arrow_id.0).unwrap();
            let mesh = meshes.get_mut(mesh_id).unwrap();
            *mesh = Tri { i, f, ip, fp, b: 2. } .into();
        }
    }
    for (id, wh) in wh_query.iter() {
        let f = trans_query.get(id).unwrap().translation().xy();
        let i = trans_query.get(wh.bh).unwrap().translation().xy();
        let fp = radius_query.get(id).unwrap().0;
        let ip = radius_query.get(wh.bh).unwrap().0;
        if let Ok(arrow_id) = arrow_query.get(id) {
            let aabb = Aabb::enclosing([i.extend(1.), f.extend(1.)]).unwrap();
            *aabb_query.get_mut(arrow_id.0).unwrap() = aabb;
            let Mesh2dHandle(mesh_id) = mesh_ids.get(arrow_id.0).unwrap();
            let mesh = meshes.get_mut(mesh_id).unwrap();
            *mesh = Tri { i, f, ip, fp, b: 2. } .into();
        }
    }
}

pub fn draw_connecting_arrow(
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
        *mesh = Tri { i: cursor.i, f: cursor.f, ip:0.0, fp:0.0, b:2. } .into();
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
    white_hole_query: Query<&WhiteHole, Changed<WhiteHole>>,
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
