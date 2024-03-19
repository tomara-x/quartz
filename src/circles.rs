use bevy::{
    prelude::*,
    render::view::VisibleEntities,
    sprite::Mesh2dHandle,
    render::view::RenderLayers,
};

use fundsp::net::Net32;

use crate::{
    components::*,
    functions::*,
};

pub fn spawn_circles(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut depth: Local<f32>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    default_color: Res<DefaultDrawColor>,
    default_verts: Res<DefaultDrawVerts>,
    mut polygon_handles: ResMut<PolygonHandles>,
) {
    if mouse_button_input.just_released(MouseButton::Left) &&
    !keyboard_input.pressed(KeyCode::Space) {
        let r = cursor.f.distance(cursor.i);
        let v = default_verts.0;
        let color = default_color.0;
        if polygon_handles.0.len() <= v {
            polygon_handles.0.resize(v + 1, None);
        }
        if polygon_handles.0[v].is_none() {
            let handle = meshes.add(RegularPolygon::new(1., v)).into();
            polygon_handles.0[v] = Some(handle);
        }
        commands.spawn((
            ColorMesh2dBundle {
                mesh: polygon_handles.0[v].clone().unwrap(),
                material: materials.add(ColorMaterial::from(color)),
                transform: Transform {
                    translation: cursor.i.extend(*depth),
                    scale: Vec3::new(r,r,1.),
                    ..default()
                },
                ..default()
            },
            Vertices(v),
            Col(color),
            Number(0.),
            Arr(Vec::new()),
            Op("empty".to_string()),
            Targets(Vec::new()),
            Holes(Vec::new()),
            Order(0),
            (
                Network(Net32::new(0,1)),
                NetIns(Vec::new()),
                OpChanged(false),
                GainedWH(false),
                LostWH(false),
            ),
            RenderLayers::layer(1),
            Visible,
            Save,
        ));
        *depth += 0.01;
    }
}

pub fn highlight_selected(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    selected: Query<(Entity, &Vertices, &Transform), (With<Selected>, Without<Highlight>)>,
    deselected: Query<Entity, (With<Highlight>, Without<Selected>)>,
    highlight_query: Query<&Highlight>,
    highlight_color: Res<HighlightColor>,
    polygon_handles: Res<PolygonHandles>,
) {
    for (e, v, t) in selected.iter() {
        let trans = t.translation.xy().extend(t.translation.z - 0.00001);
        let highlight = commands.spawn(
            ColorMesh2dBundle {
                mesh: polygon_handles.0[v.0].clone().unwrap(),
                material: materials.add(ColorMaterial::from(highlight_color.0)),
                transform: Transform {
                    translation: trans,
                    scale: Vec3::new(t.scale.x + 5., t.scale.y + 5., 1.),
                    rotation: t.rotation,
                },
                ..default()
            }
        ).id();
        commands.entity(e).insert(Highlight(highlight));
    }
    for e in deselected.iter() {
        let highlight = highlight_query.get(e).unwrap();
        commands.entity(highlight.0).despawn();
        commands.entity(e).remove::<Highlight>();
    }
}

pub fn transform_highlights(
    moved: Query<(&Transform, &Highlight), Changed<Transform>>,
    changed_verts: Query<(&Vertices, &Highlight), Changed<Vertices>>,
    mut trans_query: Query<&mut Transform, Without<Highlight>>,
    mut handle_query: Query<&mut Mesh2dHandle>,
    polygon_handles: Res<PolygonHandles>,
) {
    for (t, h) in moved.iter() {
        let trans = t.translation.xy().extend(t.translation.z - 0.00001);
        trans_query.get_mut(h.0).unwrap().translation = trans;
        trans_query.get_mut(h.0).unwrap().rotation = t.rotation;
        trans_query.get_mut(h.0).unwrap().scale.x = t.scale.x + 5.;
        trans_query.get_mut(h.0).unwrap().scale.y = t.scale.y + 5.;
    }
    for (v, h) in changed_verts.iter() {
        if let Ok(mut handle) = handle_query.get_mut(h.0) {
            *handle = polygon_handles.0[v.0].clone().unwrap();
        }
    }
}

pub fn mark_visible_circles(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    marked_circles: Query<Entity, With<Visible>>,
    circles_query: Query<(), With<Vertices>>,
    visible: Query<&VisibleEntities>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        for e in marked_circles.iter() {
            commands.entity(e).remove::<Visible>();
        }
        for e in visible.single().iter() {
            if circles_query.contains(*e) {
                commands.entity(*e).insert(Visible);
            }
        }
    }
}

pub fn draw_drawing_circle(
    id: Res<SelectionCircle>,
    mut trans_query: Query<&mut Transform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorInfo>,
    default_verts: Res<DefaultDrawVerts>,
) {
    if mouse_button_input.pressed(MouseButton::Left) 
    && !mouse_button_input.just_pressed(MouseButton::Left) {
        let v = default_verts.0;
        trans_query.get_mut(id.0).unwrap().translation = cursor.i.extend(1.);
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = RegularPolygon::new(cursor.i.distance(cursor.f).max(0.1), v).into();
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        trans_query.get_mut(id.0).unwrap().translation = Vec3::Z;
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = Triangle2d::default().into();
    }
}

//optimize all those distance calls, use a distance squared instead
pub fn update_selection(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    query: Query<(Entity, &Transform), (Or<(With<Visible>, With<Selected>)>, With<Vertices>)>,
    selected: Query<Entity, With<Selected>>,
    selected_query: Query<&Selected>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut top_clicked_circle: Local<Option<(Entity, f32)>>,
    id: Res<SelectionCircle>,
    mut trans_query: Query<&mut Transform, Without<Vertices>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
    order_query: Query<(), With<Order>>, // non-hole circle
) {
    if keyboard_input.pressed(KeyCode::Space) { return; }
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let alt = keyboard_input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    if mouse_button_input.just_pressed(MouseButton::Left) {
        for (e, t) in query.iter() {
            if top_clicked_circle.is_some() {
                if t.translation.z > top_clicked_circle.unwrap().1 &&
                    cursor.i.distance(t.translation.xy()) < t.scale.x {
                    *top_clicked_circle = Some((e, t.translation.z));
                }
            } else {
                if cursor.i.distance(t.translation.xy()) < t.scale.x {
                    *top_clicked_circle = Some((e, t.translation.z));
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
            } else if ctrl {
                commands.entity(top.0).remove::<Selected>();
            }
        }
    } else if mouse_button_input.pressed(MouseButton::Left) && top_clicked_circle.is_none() {
        trans_query.get_mut(id.0).unwrap().translation = cursor.i.extend(100.);
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = RegularPolygon::new(cursor.i.distance(cursor.f).max(0.1), 8).into();
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        trans_query.get_mut(id.0).unwrap().translation = Vec3::Z;
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = Triangle2d::default().into();
        if top_clicked_circle.is_none() {
            if !shift {
                for entity in selected.iter() {
                    commands.entity(entity).remove::<Selected>();
                }
            }
            // select those in the dragged area
            for (e, t) in query.iter() {
                if cursor.i.distance(cursor.f) + t.scale.x > cursor.i.distance(t.translation.xy()) {
                    // only select holes if ctrl is held
                    if ctrl && order_query.contains(e) { continue; }
                    // only select non-holes if alt is held
                    else if alt && !order_query.contains(e) { continue; }
                    commands.entity(e).insert(Selected);
                }
            }
        }
        *top_clicked_circle = None;
    }
}

pub fn move_selected(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorInfo>,
    mut circle_query: Query<&mut Transform, With<Selected>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    drag_modes: Res<DragModes>,
) {
    if drag_modes.t {
        if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for mut t in circle_query.iter_mut() {
                t.translation.x += cursor.d.x;
                t.translation.y += cursor.d.y;
            }
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            for mut t in circle_query.iter_mut() {
                t.translation.y += 1.;
            }
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            for mut t in circle_query.iter_mut() {
                t.translation.y -= 1.;
            }
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            for mut t in circle_query.iter_mut() {
                t.translation.x += 1.;
            }
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            for mut t in circle_query.iter_mut() {
                t.translation.x -= 1.;
            }
        }
    }
}

pub fn rotate_selected(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorInfo>,
    mut query: Query<&mut Transform, With<Selected>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    drag_modes: Res<DragModes>,
) {
    if drag_modes.o {
        if mouse_button_input.pressed(MouseButton::Left)
        && !mouse_button_input.just_pressed(MouseButton::Left) {
            for mut t in query.iter_mut() {
                t.rotate_z(cursor.d.y / 100.);
            }
        }
        if keyboard_input.any_pressed([KeyCode::ArrowUp, KeyCode::ArrowRight]) {
            for mut t in query.iter_mut() {
                t.rotate_z(0.01);
            }
        }
        if keyboard_input.any_pressed([KeyCode::ArrowDown, KeyCode::ArrowLeft]) {
            for mut t in query.iter_mut() {
                t.rotate_z(-0.01);
            }
        }
    }
}

pub fn update_color(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorInfo>,
    mut query: Query<&mut Col, With<Selected>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    drag_modes: Res<DragModes>,
) {
    if mouse_button_input.pressed(MouseButton::Left)
    && !mouse_button_input.just_pressed(MouseButton::Left) {
        if drag_modes.h {
            for mut c in query.iter_mut() {
                let h = (c.0.h() + cursor.d.x).clamp(0., 360.);
                c.0.set_h(h);
            }
        }
        if drag_modes.s {
            for mut c in query.iter_mut() {
                let s = (c.0.s() + cursor.d.x / 100.).clamp(0., 1.);
                c.0.set_s(s);
            }
        }
        if drag_modes.l {
            for mut c in query.iter_mut() {
                let l = (c.0.l() + cursor.d.x / 100.).clamp(0., 1.);
                c.0.set_l(l);
            }
        }
        if drag_modes.a {
            for mut c in query.iter_mut() {
                let a = (c.0.a() + cursor.d.x / 100.).clamp(0., 1.);
                c.0.set_a(a);
            }
        }
    }
    if keyboard_input.any_pressed([KeyCode::ArrowLeft, KeyCode::ArrowDown]) {
        for mut c in query.iter_mut() {
            if drag_modes.h {
                let h = (c.0.h() - 1.).clamp(0., 360.);
                c.0.set_h(h);
            }
            if drag_modes.s {
                let s = (c.0.s() - 0.01).clamp(0., 1.);
                c.0.set_s(s);
            }
            if drag_modes.l {
                let l = (c.0.l() - 0.01).clamp(0., 1.);
                c.0.set_l(l);
            }
            if drag_modes.a {
                let a = (c.0.a() - 0.01).clamp(0., 1.);
                c.0.set_a(a);
            }
        }
    }
    if keyboard_input.any_pressed([KeyCode::ArrowRight, KeyCode::ArrowUp]) {
        for mut c in query.iter_mut() {
            if drag_modes.h {
                let h = (c.0.h() + 1.).clamp(0., 360.);
                c.0.set_h(h);
            }
            if drag_modes.s {
                let s = (c.0.s() + 0.01).clamp(0., 1.);
                c.0.set_s(s);
            }
            if drag_modes.l {
                let l = (c.0.l() + 0.01).clamp(0., 1.);
                c.0.set_l(l);
            }
            if drag_modes.a {
                let a = (c.0.a() + 0.01).clamp(0., 1.);
                c.0.set_a(a);
            }
        }
    }
}

pub fn update_mat(
    mut mats: ResMut<Assets<ColorMaterial>>,
    material_ids: Query<&Handle<ColorMaterial>>,
    color_query: Query<(Entity, &Col), Changed<Col>>,
) {
    for (id, c) in color_query.iter() {
        if let Ok(mat_id) = material_ids.get(id) {
            let mat = mats.get_mut(mat_id).unwrap();
            mat.color = c.0;
        }
    }
}

pub fn update_radius(
    mut query: Query<&mut Transform, With<Selected>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    drag_modes: Res<DragModes>,
) {
    if drag_modes.r {
        if mouse_button_input.pressed(MouseButton::Left)
        && !mouse_button_input.just_pressed(MouseButton::Left) {
            for mut t in query.iter_mut() {
                t.scale.x = (t.scale.x + cursor.d.y).max(0.);
                t.scale.y = (t.scale.y + cursor.d.y).max(0.);
            }
        }
        if keyboard_input.any_pressed([KeyCode::ArrowUp, KeyCode::ArrowRight]) {
            for mut t in query.iter_mut() {
                t.scale.x = (t.scale.x + 1.).max(0.);
                t.scale.y = (t.scale.y + 1.).max(0.);
            }
        }
        if keyboard_input.any_pressed([KeyCode::ArrowDown, KeyCode::ArrowLeft]) {
            for mut t in query.iter_mut() {
                t.scale.x = (t.scale.x - 1.).max(0.);
                t.scale.y = (t.scale.y - 1.).max(0.);
            }
        }
    }
}

pub fn update_vertices(
    mut query: Query<&mut Vertices, With<Selected>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    drag_modes: Res<DragModes>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorInfo>,
    mut delta: Local<f32>,
) {
    if drag_modes.v {
        if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {
            *delta += cursor.d.y / 10.;
            let d = *delta as i32;
            if d >= 1 {
                for mut v in query.iter_mut() {
                    v.0 += d as usize;
                }
                *delta = 0.;
            } else if d <= -1 {
                for mut v in query.iter_mut() {
                    let x = v.0 as i32 + d;
                    if x >= 3 {
                        v.0 = x as usize;
                    }
                }
                *delta = 0.;
            }
        }
        if keyboard_input.any_just_pressed([KeyCode::ArrowUp, KeyCode::ArrowRight]) {
            for mut v in query.iter_mut() {
                v.0 += 1;
            }
        }
        if keyboard_input.any_just_pressed([KeyCode::ArrowDown, KeyCode::ArrowLeft]) {
            for mut v in query.iter_mut() {
                v.0 = (v.0 - 1).max(3);
            }
        }
    }
}

pub fn update_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    mut handle_query: Query<&mut Mesh2dHandle>,
    query: Query<(Entity, &Vertices), Changed<Vertices>>,
    mut polygon_handles: ResMut<PolygonHandles>,
) {
    for (id, v) in query.iter() {
        if polygon_handles.0.len() <= v.0 {
            polygon_handles.0.resize(v.0 + 1, None);
        }
        if polygon_handles.0[v.0].is_none() {
            let handle = meshes.add(RegularPolygon::new(1., v.0)).into();
            polygon_handles.0[v.0] = Some(handle);
        }
        if let Ok(mut handle) = handle_query.get_mut(id) {
            *handle = polygon_handles.0[v.0].clone().unwrap();
        }
    }
}

pub fn update_num(
    mut query: Query<&mut Number, With<Selected>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    drag_modes: Res<DragModes>,
) {
    if drag_modes.n {
        if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for mut n in query.iter_mut() {
                n.0 += cursor.d.y / 10.;
            }
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            for mut n in query.iter_mut() {
                n.0 += 0.01;
            }
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            for mut n in query.iter_mut() {
                n.0 -= 0.01;
            }
        }
    }
}

pub fn update_info_text(
    mut text_query: Query<&mut Text>,
    mut text_trans: Query<&mut Transform, (Without<Vertices>, Without<InfoText>)>,
    trans_query: Query<(&Transform, &InfoText), Or<(Changed<Transform>, Added<InfoText>)>>,
    order_query: Query<(&Order, &InfoText), Or<(Changed<Order>, Added<InfoText>)>>,
    num_query: Query<(&Number, &InfoText), Or<(Changed<Number>, Added<InfoText>)>>,
    op_query: Query<(&Op, &InfoText), Or<(Changed<Op>, Added<InfoText>)>>,
    white_hole_query: Query<(&WhiteHole, &InfoText), Or<(Changed<WhiteHole>, Added<InfoText>)>>,
    black_hole_query: Query<&InfoText, With<BlackHole>>,
    color_query: Query<(&Col, &InfoText), Or<(Changed<Col>, Added<InfoText>)>>,
    // TODO(amy): cleanup!
    added_bh_query: Query<(&BlackHole, &InfoText), Added<InfoText>>,
    generic_wh_query: Query<&WhiteHole>,
) {
    for (trans, text) in trans_query.iter() {
        let t = trans.translation;
        text_trans.get_mut(text.0).unwrap().translation = t.xy().extend(t.z + 0.00001);
    }
    for (order, text) in order_query.iter() {
        text_query.get_mut(text.0).unwrap().sections[1].value = format!("{}\n", order.0);
    }
    // this is messy as it changes every time the wh changes (open / lt change)
    for (wh, text) in white_hole_query.iter() {
        text_query.get_mut(text.0).unwrap().sections[1].value = lt_to_string(wh.link_types.1);
        if let Ok(bh_text) = black_hole_query.get(wh.bh) {
            text_query.get_mut(bh_text.0).unwrap().sections[1].value = lt_to_string(wh.link_types.0);
        }
    }
    // more duck tape
    for (bh, text) in added_bh_query.iter() {
        let wh = generic_wh_query.get(bh.wh).unwrap();
        text_query.get_mut(text.0).unwrap().sections[1].value = lt_to_string(wh.link_types.0);
    }
    for (op, text) in op_query.iter() {
        text_query.get_mut(text.0).unwrap().sections[2].value = format!("{}\n", op.0);
    }
    for (n, text) in num_query.iter() {
        text_query.get_mut(text.0).unwrap().sections[3].value = n.0.to_string();
    }
    for (col, text) in color_query.iter() {
        let l = if col.0.l() < 0.3 {1.} else {0.};
        let opposite_color = Color::hsl(0., 1.0, l);
        let t = &mut text_query.get_mut(text.0).unwrap();
        for section in &mut t.sections {
            section.style.color = opposite_color;
        }
    }
}

pub fn delete_selected(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    selected_query: Query<Entity, With<Selected>>,
    mut holes_query: Query<&mut Holes>,
    bh_query: Query<&BlackHole>,
    wh_query: Query<&WhiteHole>,
    arrow_query: Query<&ConnectionArrow>,
    info_text_query: Query<&InfoText>,
    highlight_query: Query<&Highlight>,
    mut order_change: EventWriter<OrderChange>,
    mut lost_wh_query: Query<&mut LostWH>,
) {
    if keyboard_input.just_pressed(KeyCode::Delete) {
        let mut order = false;
        for e in selected_query.iter() {
            if let Ok(holes) = holes_query.get(e) { // it's a circle
                for hole in &holes.0.clone() {
                    if let Ok(bh) = bh_query.get(*hole) {
                        let arrow = arrow_query.get(bh.wh).unwrap().0;
                        commands.entity(arrow).despawn();
                        commands.entity(*hole).despawn();
                        commands.entity(bh.wh).despawn();
                        if let Ok(wh_text) = info_text_query.get(bh.wh) {
                            commands.entity(wh_text.0).despawn();
                        }
                        if let Ok(bh_text) = info_text_query.get(*hole) {
                            commands.entity(bh_text.0).despawn();
                        }
                        if let Ok(highlight) = highlight_query.get(bh.wh) {
                            commands.entity(highlight.0).despawn();
                        }
                        if let Ok(highlight) = highlight_query.get(*hole) {
                            commands.entity(highlight.0).despawn();
                        }
                        lost_wh_query.get_mut(bh.wh_parent).unwrap().0 = true;
                        holes_query.get_mut(bh.wh_parent).unwrap().0.retain(|x| *x != bh.wh);
                    } else if let Ok(wh) = wh_query.get(*hole) {
                        // don't remove things that will get removed later
                        if selected_query.contains(wh.bh_parent) { continue; }
                        let arrow = arrow_query.get(*hole).unwrap().0;
                        commands.entity(arrow).despawn();
                        commands.entity(wh.bh).despawn();
                        commands.entity(*hole).despawn();
                        if let Ok(wh_text) = info_text_query.get(*hole) {
                            commands.entity(wh_text.0).despawn();
                        }
                        if let Ok(bh_text) = info_text_query.get(wh.bh) {
                            commands.entity(bh_text.0).despawn();
                        }
                        if let Ok(highlight) = highlight_query.get(*hole) {
                            commands.entity(highlight.0).despawn();
                        }
                        if let Ok(highlight) = highlight_query.get(wh.bh) {
                            commands.entity(highlight.0).despawn();
                        }
                        holes_query.get_mut(wh.bh_parent).unwrap().0.retain(|x| *x != wh.bh);
                    }
                }
                order = true;
                if let Ok(text) = info_text_query.get(e) {
                    commands.entity(text.0).despawn();
                }
                if let Ok(highlight) = highlight_query.get(e) {
                    commands.entity(highlight.0).despawn();
                }
                commands.entity(e).despawn();
            } else { // it's a hole
                if let Ok(wh) = wh_query.get(e) {
                    // get parent
                    let parent = bh_query.get(wh.bh).unwrap().wh_parent;
                    if selected_query.contains(parent) { continue; }
                    if selected_query.contains(wh.bh_parent) { continue; }
                    // remove from parents' vecs
                    holes_query.get_mut(parent).unwrap().0.retain(|x| *x != e);
                    holes_query.get_mut(wh.bh_parent).unwrap().0.retain(|x| *x != wh.bh);
                    // parent has lost a wh
                    lost_wh_query.get_mut(parent).unwrap().0 = true;
                    let arrow = arrow_query.get(e).unwrap().0;
                    commands.entity(arrow).despawn();
                    commands.entity(e).despawn();
                    commands.entity(wh.bh).despawn();
                    // info texts and highlights
                    if let Ok(wh_text) = info_text_query.get(e) {
                        commands.entity(wh_text.0).despawn();
                    }
                    if let Ok(bh_text) = info_text_query.get(wh.bh) {
                        commands.entity(bh_text.0).despawn();
                    }
                    if let Ok(highlight) = highlight_query.get(e) {
                        commands.entity(highlight.0).despawn();
                    }
                    if let Ok(highlight) = highlight_query.get(wh.bh) {
                        commands.entity(highlight.0).despawn();
                    }
                } else if let Ok(bh) = bh_query.get(e) {
                    let parent = wh_query.get(bh.wh).unwrap().bh_parent;
                    if selected_query.contains(parent) { continue; }
                    if selected_query.contains(bh.wh_parent) { continue; }
                    if selected_query.contains(bh.wh) { continue; }
                    holes_query.get_mut(parent).unwrap().0.retain(|x| *x != e);
                    holes_query.get_mut(bh.wh_parent).unwrap().0.retain(|x| *x != bh.wh);
                    lost_wh_query.get_mut(bh.wh_parent).unwrap().0 = true;
                    let arrow = arrow_query.get(bh.wh).unwrap().0;
                    commands.entity(arrow).despawn();
                    commands.entity(e).despawn();
                    commands.entity(bh.wh).despawn();
                    if let Ok(wh_text) = info_text_query.get(e) {
                        commands.entity(wh_text.0).despawn();
                    }
                    if let Ok(bh_text) = info_text_query.get(bh.wh) {
                        commands.entity(bh_text.0).despawn();
                    }
                    if let Ok(highlight) = highlight_query.get(e) {
                        commands.entity(highlight.0).despawn();
                    }
                    if let Ok(highlight) = highlight_query.get(bh.wh) {
                        commands.entity(highlight.0).despawn();
                    }
                }
            }
        }
        if order { order_change.send_default(); }
    }
}

pub fn open_after_drag(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    drag_modes: Res<DragModes>,
    query: Query<&Holes, With<Selected>>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<&BlackHole>,
) {
    let arrows = [KeyCode::ArrowDown, KeyCode::ArrowUp, KeyCode::ArrowLeft, KeyCode::ArrowRight];
    if keyboard_input.any_pressed(arrows)
    || mouse_button_input.pressed(MouseButton::Left) {
        let mut lts_to_open = Vec::new();
        if drag_modes.t { lts_to_open.push(-3); lts_to_open.push(-4); }
        if drag_modes.r { lts_to_open.push(-2); }
        if drag_modes.n { lts_to_open.push(-1); }
        if drag_modes.h { lts_to_open.push(-6); }
        if drag_modes.s { lts_to_open.push(-7); }
        if drag_modes.l { lts_to_open.push(-8); }
        if drag_modes.a { lts_to_open.push(-9); }
        if drag_modes.o { lts_to_open.push(-12); }
        if drag_modes.v { lts_to_open.push(-11); }
        for holes in query.iter() {
            for hole in &holes.0 {
                if let Ok(bh) = black_hole_query.get(*hole) {
                    if let Ok(wh) = white_hole_query.get(bh.wh) {
                        if lts_to_open.contains(&wh.link_types.0) {
                            white_hole_query.get_mut(bh.wh).unwrap().open = true;
                        }
                    }
                }
            }
        }
    }
}
