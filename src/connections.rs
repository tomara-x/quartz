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
    mut order_query: Query<&mut Order>,
    mut order_change: EventWriter<OrderChange>,
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
            // don't connect entity to itself
            if source_entity.0 == sink_entity.0 { return; }
            // increment order of sink
            let src_order = order_query.get(src).unwrap().0;
            let snk_order = order_query.get(snk).unwrap().0;
            if snk_order <= src_order {
                order_query.get_mut(snk).unwrap().0 = src_order + 1;
                order_change.send_default();
            }
            // get radius and transform
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
                    transform: Transform::from_translation(Vec3::new(0.,0.,100.)),
                    ..default()
                }
            ).id();
            // spawn circles
            let black_hole = commands.spawn(( ColorMesh2dBundle {
                    mesh: meshes.add(shape::Circle { radius: src_radius * 0.15, vertices: 6 }.into()).into(),
                    material: materials.add(ColorMaterial::from(Color::BLACK)),
                    transform: Transform::from_translation((cursor.i - src_trans.xy()).extend(0.001)),
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
                    transform: Transform::from_translation((cursor.f - snk_trans.xy()).extend(0.001)),
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
        if let (Ok(i), Ok(f)) = (trans_query.get(id), trans_query.get(bh.wh)) {
            let i = i.translation().xy();
            let f = f.translation().xy();
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
    }
    for (id, wh) in wh_query.iter() {
        if let (Ok(f), Ok(i)) = (trans_query.get(id), trans_query.get(wh.bh)) {
            let f = f.translation().xy();
            let i = i.translation().xy();
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
}

pub fn draw_connecting_arrow(
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<Input<KeyCode>>,
    id: Res<ConnectingLine>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
    mut aabb_query: Query<&mut Aabb>,
) {
    if mouse_button_input.pressed(MouseButton::Left)
    && !mouse_button_input.just_pressed(MouseButton::Left)
    && !keyboard_input.pressed(KeyCode::Space) {
        let aabb = Aabb::enclosing([cursor.i.extend(1.), cursor.f.extend(1.)]).unwrap();
        *aabb_query.get_mut(id.0).unwrap() = aabb;
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

pub fn delete_selected_holes(
    keyboard_input: Res<Input<KeyCode>>,
    bh_query: Query<(Entity, &BlackHole), With<Selected>>,
    wh_query: Query<(Entity, &WhiteHole), With<Selected>>,
    unselected: Query<Entity, (Without<Selected>, Or<(With<BlackHole>, With<WhiteHole>)>)>,
    arrow_query: Query<&ConnectionArrow>,
    mut commands: Commands,
    info_text_query: Query<&InfoText>,
    highlight_query: Query<&Highlight>,
) {
    if keyboard_input.just_pressed(KeyCode::Delete) {
        for (e, bh) in bh_query.iter() {
            if let Ok(wh_id) = unselected.get(bh.wh) {
                let arrow = arrow_query.get(wh_id).unwrap().0;
                commands.entity(arrow).despawn();
                commands.entity(wh_id).remove_parent();
                commands.entity(wh_id).despawn_recursive();
                if let Ok(wh_text) = info_text_query.get(wh_id) {
                    commands.entity(wh_text.0).despawn();
                }
                if let Ok(highlight) = highlight_query.get(wh_id) {
                    commands.entity(highlight.0).despawn();
                }
            }
            commands.entity(e).remove_parent();
            commands.entity(e).despawn_recursive();
            if let Ok(bh_text) = info_text_query.get(e) {
                commands.entity(bh_text.0).despawn();
            }
            if let Ok(highlight) = highlight_query.get(e) {
                commands.entity(highlight.0).despawn();
            }
        }
        for (e, wh) in wh_query.iter() {
            if let Ok(bh_id) = unselected.get(wh.bh) {
                commands.entity(bh_id).remove_parent();
                commands.entity(bh_id).despawn_recursive();
                if let Ok(bh_text) = info_text_query.get(bh_id) {
                    commands.entity(bh_text.0).despawn();
                }
                if let Ok(highlight) = highlight_query.get(bh_id) {
                    commands.entity(highlight.0).despawn();
                }
            }
            let arrow = arrow_query.get(e).unwrap().0;
            commands.entity(arrow).despawn();
            commands.entity(e).remove_parent();
            commands.entity(e).despawn_recursive();
            if let Ok(wh_text) = info_text_query.get(e) {
                commands.entity(wh_text.0).despawn();
            }
            if let Ok(highlight) = highlight_query.get(e) {
                commands.entity(highlight.0).despawn();
            }
        }
    }
}
