use bevy::{
    prelude::*,
    render::view::{RenderLayers, VisibleEntities},
    sprite::WithMesh2d,
};

use crate::components::*;

pub fn connect(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    trans_query: Query<&Transform>,
    vertices_query: Query<&Vertices>,
    visible: Query<&VisibleEntities>,
    cursor: Res<CursorInfo>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut holes_query: Query<&mut Holes>,
    mut gained_wh_query: Query<&mut GainedWH>,
    default_lt: Res<DefaultLT>,
    polygon_handles: Res<PolygonHandles>,
    arrow_handle: Res<ArrowHandle>,
    connection_mat: Res<ConnectionMat>,
    mut order_query: Query<&mut Order>,
    mut order_change: EventWriter<OrderChange>,
) {
    if mouse_button_input.just_released(MouseButton::Left)
        && !keyboard_input.pressed(KeyCode::KeyT)
        && !keyboard_input.pressed(KeyCode::Space)
    {
        let mut source_entity: (Option<Entity>, f32) = (None, f32::MIN);
        let mut sink_entity: (Option<Entity>, f32) = (None, f32::MIN);
        for e in visible.single().get::<WithMesh2d>() {
            if holes_query.contains(*e) {
                // it's a non-hole
                let t = trans_query.get(*e).unwrap();
                if cursor.i.distance(t.translation.xy()) < t.scale.x
                    && t.translation.z > source_entity.1
                {
                    source_entity = (Some(*e), t.translation.z);
                }
                if cursor.f.distance(t.translation.xy()) < t.scale.x
                    && t.translation.z > sink_entity.1
                {
                    sink_entity = (Some(*e), t.translation.z);
                }
            }
        }

        if let (Some(src), Some(snk)) = (source_entity.0, sink_entity.0) {
            // don't connect entity to itself
            if source_entity.0 == sink_entity.0 {
                return;
            }
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
            let src_trans = trans_query.get(src).unwrap().translation;
            let snk_trans = trans_query.get(snk).unwrap().translation;
            let src_radius = trans_query.get(src).unwrap().scale.x;
            let snk_radius = trans_query.get(snk).unwrap().scale.x;
            let src_verts = vertices_query.get(src).unwrap().0;
            let snk_verts = vertices_query.get(snk).unwrap().0;
            let bh_radius = src_radius * 0.15;
            let wh_radius = snk_radius * 0.15;

            // spawn connection arrow
            let arrow = commands
                .spawn((
                    ColorMesh2dBundle {
                        mesh: arrow_handle.0.clone(),
                        material: connection_mat.0.clone(),
                        transform: Transform::default(),
                        ..default()
                    },
                    RenderLayers::layer(4),
                ))
                .id();
            // spawn circles
            let bh_depth = 0.001 * (holes_query.get(src).unwrap().0.len() + 1) as f32;
            let bh_verts = snk_verts;
            let bh_color = Hsla::new(0., 0., 0.2, 1.);
            let black_hole = commands
                .spawn((
                    ColorMesh2dBundle {
                        mesh: polygon_handles.0[bh_verts].clone().unwrap(),
                        material: materials.add(ColorMaterial::from_color(bh_color)),
                        transform: Transform {
                            translation: cursor.i.extend(bh_depth + src_trans.z),
                            scale: Vec3::new(bh_radius, bh_radius, 1.),
                            ..default()
                        },
                        ..default()
                    },
                    Col(bh_color),
                    Vertices(bh_verts),
                    RenderLayers::layer(2),
                    Save,
                ))
                .id();
            let wh_depth = 0.001 * (holes_query.get(snk).unwrap().0.len() + 1) as f32;
            let wh_verts = src_verts;
            let wh_color = Hsla::new(0., 0., 0.8, 1.);
            let white_hole = commands
                .spawn((
                    ColorMesh2dBundle {
                        mesh: polygon_handles.0[bh_verts].clone().unwrap(),
                        material: materials.add(ColorMaterial::from_color(wh_color)),
                        transform: Transform {
                            translation: cursor.f.extend(wh_depth + snk_trans.z),
                            scale: Vec3::new(wh_radius, wh_radius, 1.),
                            ..default()
                        },
                        ..default()
                    },
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
                ))
                .id();

            // insert black hole white hole
            commands.entity(black_hole).insert(BlackHole { wh: white_hole, wh_parent: snk });

            // add to parents
            holes_query.get_mut(src).unwrap().0.push(black_hole);
            holes_query.get_mut(snk).unwrap().0.push(white_hole);
        }
    }
}

pub fn target(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    circle_trans_query: Query<&Transform, With<Vertices>>,
    visible: Query<&VisibleEntities>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut targets_query: Query<&mut Targets>,
) {
    if mouse_button_input.just_released(MouseButton::Left)
        && keyboard_input.pressed(KeyCode::KeyT)
        && !keyboard_input.pressed(KeyCode::Space)
    {
        let mut source_entity: (Option<Entity>, f32) = (None, f32::MIN);
        let mut sink_entity: (Option<Entity>, f32) = (None, f32::MIN);
        for e in visible.single().get::<WithMesh2d>() {
            if let Ok(t) = circle_trans_query.get(*e) {
                if cursor.i.distance(t.translation.xy()) < t.scale.x
                    && t.translation.z > source_entity.1
                {
                    source_entity = (Some(*e), t.translation.z);
                }
                if cursor.f.distance(t.translation.xy()) < t.scale.x
                    && t.translation.z > sink_entity.1
                {
                    sink_entity = (Some(*e), t.translation.z);
                }
            }
        }
        if let (Some(src), Some(snk)) = (source_entity.0, sink_entity.0) {
            // don't target self
            if source_entity.0 == sink_entity.0 {
                return;
            }
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
    connection_width: Res<ConnectionWidth>,
) {
    for (id, bh) in bh_query.iter() {
        if wh_query.contains(bh.wh) {
            continue;
        }
        if let (Ok(bh_t), Ok(wh_t)) = (trans_query.get(id), trans_query.get(bh.wh)) {
            let bh_radius = bh_t.scale.x;
            let wh_radius = wh_t.scale.x;
            let bh_trans = bh_t.translation.xy();
            let wh_trans = wh_t.translation.xy();
            if let Ok(arrow_id) = arrow_query.get(bh.wh) {
                let perp = (bh_trans - wh_trans).perp();
                let norm = (wh_trans - bh_trans).normalize_or_zero();
                let i = wh_trans - wh_radius * norm;
                let f = bh_trans + bh_radius * norm;
                *arrow_trans.get_mut(arrow_id.0).unwrap() = Transform {
                    translation: ((i + f) / 2.).extend(100.),
                    scale: Vec3::new(
                        connection_width.0,
                        wh_trans.distance(bh_trans) - (bh_radius + wh_radius),
                        1.,
                    ),
                    rotation: Quat::from_rotation_z(perp.to_angle()),
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
                let norm = (wh_trans - bh_trans).normalize_or_zero();
                let i = wh_trans - wh_radius * norm;
                let f = bh_trans + bh_radius * norm;
                *arrow_trans.get_mut(arrow_id.0).unwrap() = Transform {
                    translation: ((i + f) / 2.).extend(100.),
                    scale: Vec3::new(
                        connection_width.0,
                        wh_trans.distance(bh_trans) - (bh_radius + wh_radius),
                        1.,
                    ),
                    rotation: Quat::from_rotation_z(perp.to_angle()),
                };
            }
        }
    }
}

pub fn connect_targets(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Vertices), With<Order>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut holes_query: Query<&mut Holes>,
    mut gained_wh_query: Query<&mut GainedWH>,
    polygon_handles: Res<PolygonHandles>,
    arrow_handle: Res<ArrowHandle>,
    connection_mat: Res<ConnectionMat>,
    mut connect_command: EventReader<ConnectCommand>,
    arr_query: Query<&Arr>,
    mut targets_query: Query<&mut Targets>,
    white_hole_query: Query<&WhiteHole>,
) {
    for e in connect_command.read() {
        let targets = &targets_query.get(e.0).unwrap().0;
        if targets.is_empty() {
            continue;
        }
        let arr = &arr_query.get(e.0).unwrap().0;
        let lt = if let Some(a) = arr.get(0..2) { (a[0] as i8, a[1] as i8) } else { (0, 0) };
        let mut white_holes = Vec::new();
        for pair in targets.windows(2) {
            let (src, snk) = (pair[0], pair[1]);
            // don't connect entity to itself
            if src == snk {
                continue;
            }
            // sink has gained a connection
            gained_wh_query.get_mut(snk).unwrap().0 = true;
            // get translation, radius, and vertices
            let src_trans = query.get(src).unwrap().1.translation;
            let snk_trans = query.get(snk).unwrap().1.translation;
            let src_radius = query.get(src).unwrap().1.scale.x;
            let snk_radius = query.get(snk).unwrap().1.scale.x;
            let src_verts = query.get(src).unwrap().2 .0;
            let snk_verts = query.get(snk).unwrap().2 .0;
            let bh_radius = src_radius * 0.15;
            let wh_radius = snk_radius * 0.15;

            // spawn connection arrow
            let arrow = commands
                .spawn((
                    ColorMesh2dBundle {
                        mesh: arrow_handle.0.clone(),
                        material: connection_mat.0.clone(),
                        transform: Transform::default(),
                        ..default()
                    },
                    RenderLayers::layer(4),
                ))
                .id();
            // spawn circles
            let bh_depth = 0.001 * (holes_query.get(src).unwrap().0.len() + 1) as f32;
            let bh_verts = snk_verts;
            let bh_color = Hsla::new(0., 0., 0.2, 1.);
            let black_hole = commands
                .spawn((
                    ColorMesh2dBundle {
                        mesh: polygon_handles.0[bh_verts].clone().unwrap(),
                        material: materials.add(ColorMaterial::from_color(bh_color)),
                        transform: Transform {
                            translation: src_trans.xy().extend(bh_depth + src_trans.z),
                            scale: Vec3::new(bh_radius, bh_radius, 1.),
                            ..default()
                        },
                        ..default()
                    },
                    Col(bh_color),
                    Vertices(bh_verts),
                    RenderLayers::layer(2),
                    Save,
                ))
                .id();
            let wh_depth = 0.001 * (holes_query.get(snk).unwrap().0.len() + 1) as f32;
            let wh_verts = src_verts;
            let wh_color = Hsla::new(0., 0., 0.8, 1.);
            let white_hole = commands
                .spawn((
                    ColorMesh2dBundle {
                        mesh: polygon_handles.0[bh_verts].clone().unwrap(),
                        material: materials.add(ColorMaterial::from_color(wh_color)),
                        transform: Transform {
                            translation: snk_trans.xy().extend(wh_depth + snk_trans.z),
                            scale: Vec3::new(wh_radius, wh_radius, 1.),
                            ..default()
                        },
                        ..default()
                    },
                    Col(wh_color),
                    Vertices(wh_verts),
                    WhiteHole { bh_parent: src, bh: black_hole, link_types: lt, open: true },
                    RenderLayers::layer(3),
                    Save,
                    ConnectionArrow(arrow),
                ))
                .id();

            // insert black hole white hole
            commands.entity(black_hole).insert(BlackHole { wh: white_hole, wh_parent: snk });

            // add to parents
            holes_query.get_mut(src).unwrap().0.push(black_hole);
            holes_query.get_mut(snk).unwrap().0.push(white_hole);

            white_holes.push(white_hole);
        }
        for hole in &holes_query.get(e.0).unwrap().0 {
            if let Ok(wh) = white_hole_query.get(*hole) {
                if wh.link_types == (-14, 2) {
                    targets_query.get_mut(wh.bh_parent).unwrap().0 = white_holes;
                    break;
                }
            }
        }
    }
}
