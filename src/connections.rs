use bevy::{
    prelude::*,
    sprite::Mesh2dHandle,
    render::{
        primitives::Aabb,
        view::RenderLayers,
    },
};

use crate::{
    components::*,
    meshes::*,
};

pub fn connect(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Vertices), (With<Visible>, With<Order>)>,
    cursor: Res<CursorInfo>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut order_query: Query<&mut Order>,
    mut order_change: EventWriter<OrderChange>,
    mut holes_query: Query<&mut Holes>,
    mut gained_wh_query: Query<&mut GainedWH>,
    connection_color: Res<ConnectionColor>,
    default_lt: Res<DefaultLT>,
    polygon_handles: Res<PolygonHandles>,
    arrow_handle: Res<ArrowHandle>,
) {
    if mouse_button_input.just_released(MouseButton::Left)
    && !keyboard_input.pressed(KeyCode::KeyT)
    && !keyboard_input.pressed(KeyCode::Space) {
        let mut source_entity: (Option<Entity>, f32) = (None, f32::MIN);
        let mut sink_entity: (Option<Entity>, f32) = (None, f32::MIN);
        for (e, t, _) in query.iter() {
            if cursor.i.distance(t.translation.xy()) < t.scale.x
                && t.translation.z > source_entity.1 {
                source_entity = (Some(e), t.translation.z);
            }
            if cursor.f.distance(t.translation.xy()) < t.scale.x
                && t.translation.z > sink_entity.1 {
                sink_entity = (Some(e), t.translation.z);
            }
        }

        if let (Some(src), Some(snk)) = (source_entity.0, sink_entity.0) {
            // don't connect entity to itself
            if source_entity.0 == sink_entity.0 { return; }
            // sink has gained a connection
            gained_wh_query.get_mut(snk).unwrap().0 = true;
            // increment order of sink
            let src_order = order_query.get(src).unwrap().0;
            let snk_order = order_query.get(snk).unwrap().0;
            if snk_order <= src_order {
                order_query.get_mut(snk).unwrap().0 = src_order + 1;
                order_change.send_default();
            }
            // get translation, radius, and vertices
            let src_trans = query.get(src).unwrap().1.translation;
            let snk_trans = query.get(snk).unwrap().1.translation;
            let src_radius = query.get(src).unwrap().1.scale.x;
            let snk_radius = query.get(snk).unwrap().1.scale.x;
            let src_verts = query.get(src).unwrap().2.0;
            let snk_verts = query.get(snk).unwrap().2.0;
            let bh_radius = src_radius * 0.15;
            let wh_radius = snk_radius * 0.15;
            let perp = (cursor.i - cursor.f).perp();

            // spawn connection arrow
            let arrow = commands.spawn((
                ColorMesh2dBundle {
                    mesh: arrow_handle.0.clone(),
                    material: materials.add(ColorMaterial::from(connection_color.0)),
                    transform: Transform {
                        translation: ((cursor.i + cursor.f) / 2.).extend(100.),
                        scale: Vec3::new(4., cursor.i.distance(cursor.f) - (bh_radius + wh_radius), 1.),
                        rotation: Quat::from_rotation_z(perp.y.atan2(perp.x)),
                    },
                    ..default()
                },
                RenderLayers::layer(4),
            )).id();
            // spawn circles
            let bh_depth = 0.001 * (holes_query.get(src).unwrap().0.len() + 1) as f32;
            let bh_verts = snk_verts;
            let bh_color = Color::hsl(0., 0., 0.2);
            let black_hole = commands.spawn(( ColorMesh2dBundle {
                    mesh: polygon_handles.0[bh_verts].clone().unwrap(),
                    material: materials.add(ColorMaterial::from(bh_color)),
                    transform: Transform {
                        translation: cursor.i.extend(bh_depth + src_trans.z),
                        scale: Vec3::new(bh_radius, bh_radius, 1.),
                        ..default()
                    },
                    ..default()
                },
                Visible,
                Col(bh_color),
                Vertices(bh_verts),
                RenderLayers::layer(2),
                Save,
            )).id();
            let wh_depth = 0.001 * (holes_query.get(snk).unwrap().0.len() + 1) as f32;
            let wh_verts = src_verts;
            let wh_color = Color::hsl(0., 0., 0.8);
            let white_hole = commands.spawn(( ColorMesh2dBundle {
                    mesh: polygon_handles.0[bh_verts].clone().unwrap(),
                    material: materials.add(ColorMaterial::from(wh_color)),
                    transform: Transform {
                        translation: cursor.f.extend(wh_depth + snk_trans.z),
                        scale: Vec3::new(wh_radius, wh_radius, 1.),
                        ..default()
                    },
                    ..default()
                },
                Visible,
                Col(wh_color),
                Vertices(wh_verts),
                WhiteHole {
                    bh_parent: src,
                    bh: black_hole,
                    link_types: default_lt.0,
                    open: true,
                },
                RenderLayers::layer(3),
                Save,
                ConnectionArrow(arrow),
            )).id();

            // insert black hole white hole
            commands.entity(black_hole).insert(
                BlackHole {
                    wh: white_hole,
                    wh_parent: snk,
                });
                
            // add to parents
            holes_query.get_mut(src).unwrap().0.push(black_hole);
            holes_query.get_mut(snk).unwrap().0.push(white_hole);
        }
    }
}

pub fn target(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    query: Query<(Entity, &Transform), With<Visible>>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut targets_query: Query<&mut Targets>,
) {
    if mouse_button_input.just_released(MouseButton::Left)
    && keyboard_input.pressed(KeyCode::KeyT)
    && !keyboard_input.pressed(KeyCode::Space) {
        let mut source_entity: (Option<Entity>, f32) = (None, f32::MIN);
        let mut sink_entity: (Option<Entity>, f32) = (None, f32::MIN);
        for (e, t) in query.iter() {
            if cursor.i.distance(t.translation.xy()) < t.scale.x
            && t.translation.z > source_entity.1 {
                source_entity = (Some(e), t.translation.z);
            }
            if cursor.f.distance(t.translation.xy()) < t.scale.x
            && t.translation.z > sink_entity.1 {
                sink_entity = (Some(e), t.translation.z);
            }
        }
        if let (Some(src), Some(snk)) = (source_entity.0, sink_entity.0) {
            // don't target self
            if source_entity.0 == sink_entity.0 { return; }
            if let Ok(mut targets) = targets_query.get_mut(src) {
                targets.0.push(snk);
            }
        }
    }
}

pub fn update_connection_arrows(
    bh_query: Query<(Entity, &BlackHole), (Changed<Transform>, With<Vertices>)>,
    wh_query: Query<(Entity, &WhiteHole), (Changed<Transform>, With<Vertices>)>,
    trans_query: Query<&Transform, With<Vertices>>,
    mut arrow_trans: Query<&mut Transform, Without<Vertices>>,
    arrow_query: Query<&ConnectionArrow>,
) {
    for (id, bh) in bh_query.iter() {
        if wh_query.contains(bh.wh) { continue; }
        if let (Ok(bh_t), Ok(wh_t)) = (trans_query.get(id), trans_query.get(bh.wh)) {
            let bh_radius = bh_t.scale.x;
            let wh_radius = wh_t.scale.x;
            let bh_trans = bh_t.translation.xy();
            let wh_trans = wh_t.translation.xy();
            if let Ok(arrow_id) = arrow_query.get(bh.wh) {
                let perp = (bh_trans - wh_trans).perp();
                *arrow_trans.get_mut(arrow_id.0).unwrap() = Transform {
                    translation: ((bh_trans + wh_trans) / 2.).extend(100.),
                    scale: Vec3::new(4., wh_trans.distance(bh_trans) - (bh_radius + wh_radius), 1.),
                    rotation: Quat::from_rotation_z(perp.y.atan2(perp.x)),
                };
            }
        }
    }
    for (id, wh) in wh_query.iter() {
        if let (Ok(bh_t), Ok(wh_t)) = (trans_query.get(wh.bh), trans_query.get(id)) {
            let bh_radius = bh_t.scale.x;
            let wh_radius = wh_t.scale.x;
            let bh_trans = bh_t.translation.xy();
            let wh_trans = wh_t.translation.xy();
            if let Ok(arrow_id) = arrow_query.get(id) {
                let perp = (bh_trans - wh_trans).perp();
                *arrow_trans.get_mut(arrow_id.0).unwrap() = Transform {
                    translation: ((bh_trans + wh_trans) / 2.).extend(100.),
                    scale: Vec3::new(4., wh_trans.distance(bh_trans) - (bh_radius + wh_radius), 1.),
                    rotation: Quat::from_rotation_z(perp.y.atan2(perp.x)),
                };
            }
        }
    }
}

pub fn draw_connecting_arrow(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
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
        *mesh = RegularPolygon::new(0.1, 3).into();
    }
}

pub fn delete_selected_holes(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    bh_query: Query<(Entity, &BlackHole), With<Selected>>,
    wh_query: Query<(Entity, &WhiteHole), With<Selected>>,
    unselected: Query<Entity, (Without<Selected>, Or<(With<BlackHole>, With<WhiteHole>)>)>,
    arrow_query: Query<&ConnectionArrow>,
    mut commands: Commands,
    info_text_query: Query<&InfoText>,
    highlight_query: Query<&Highlight>,
    mut lost_wh_query: Query<&mut LostWH>,
    mut holes_query: Query<&mut Holes>,
) {
    if keyboard_input.just_pressed(KeyCode::Delete) {
        for (e, bh) in bh_query.iter() {
            if let Ok(wh_id) = unselected.get(bh.wh) {
                let arrow = arrow_query.get(wh_id).unwrap().0;
                commands.entity(arrow).despawn();
                commands.entity(wh_id).despawn();
                if let Ok(wh_text) = info_text_query.get(wh_id) {
                    commands.entity(wh_text.0).despawn();
                }
                if let Ok(highlight) = highlight_query.get(wh_id) {
                    commands.entity(highlight.0).despawn();
                }
                //lost_wh_query.get_mut(bh.wh_parent).unwrap().0 = true;
            }
            commands.entity(e).despawn();
            if let Ok(bh_text) = info_text_query.get(e) {
                commands.entity(bh_text.0).despawn();
            }
            if let Ok(highlight) = highlight_query.get(e) {
                commands.entity(highlight.0).despawn();
            }
        }
        for (e, wh) in wh_query.iter() {
            if let Ok(bh_id) = unselected.get(wh.bh) {
                commands.entity(bh_id).despawn();
                if let Ok(bh_text) = info_text_query.get(bh_id) {
                    commands.entity(bh_text.0).despawn();
                }
                if let Ok(highlight) = highlight_query.get(bh_id) {
                    commands.entity(highlight.0).despawn();
                }
            }
            let arrow = arrow_query.get(e).unwrap().0;
            commands.entity(arrow).despawn();
            commands.entity(e).despawn();
            if let Ok(wh_text) = info_text_query.get(e) {
                commands.entity(wh_text.0).despawn();
            }
            if let Ok(highlight) = highlight_query.get(e) {
                commands.entity(highlight.0).despawn();
            }
            // parent has lost a connection
            //let parent = parent_query.get(e).unwrap();
            //lost_wh_query.get_mut(**parent).unwrap().0 = true;
        }
    }
}
