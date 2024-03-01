use bevy::{
    prelude::*,
    render::view::VisibleEntities,
    sprite::Mesh2dHandle,
    render::{
        primitives::Aabb,
        view::RenderLayers,
    },
};

use fundsp::net::Net32;

use crate::components::*;

pub fn spawn_circles(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut depth: Local<f32>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if mouse_button_input.just_released(MouseButton::Left) &&
    !keyboard_input.pressed(KeyCode::Space) {
        let r = cursor.f.distance(cursor.i);
        let v = 4;
        let color = Color::hsla(1., 1., 0.84, 1.);
        commands.spawn((
            ColorMesh2dBundle {
                mesh: meshes.add(RegularPolygon::new(r.max(0.1), v)).into(),
                material: materials.add(ColorMaterial::from(color)),
                transform: Transform::from_translation(cursor.i.extend(*depth)),
                ..default()
            },
            Radius(r),
            Vertices(v),
            Col(color),
            Num(0.),
            Arr(Vec::new()),
            Op("empty".to_string()),
            Targets(Vec::new()),
            Order(0),
            (
                Network(Net32::new(0,1)),
                NetIns(Vec::new()),
                NetChanged(true),
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    selected: Query<Entity, (With<Selected>, Without<Highlight>)>,
    deselected: Query<Entity, (With<Highlight>, Without<Selected>)>,
    highlight_query: Query<&Highlight>,
) {
    for e in selected.iter() {
        let highlight = commands.spawn(
            ColorMesh2dBundle {
                mesh: meshes.add(RegularPolygon::new(0.1, 3)).into(),
                material: materials.add(ColorMaterial::from(Color::hsl(0.0,1.0,0.5))),
                transform: Transform::from_translation(Vec3::Z),
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
    moved: Query<(&GlobalTransform, &Highlight), Or<(Changed<Transform>, Added<Highlight>)>>,
    resized: Query<(&Vertices, &Radius, &Highlight), Or<(Changed<Vertices>, Changed<Radius>, Added<Highlight>)>>,
    mut trans_query: Query<&mut Transform, Without<Highlight>>,
    mesh_ids: Query<&Mesh2dHandle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut aabb_query: Query<&mut Aabb>,
) {
    for (t, h) in moved.iter() {
        let t = t.compute_transform();
        let trans = t.translation.xy().extend(t.translation.z - 0.00001);
        trans_query.get_mut(h.0).unwrap().translation = trans;
        trans_query.get_mut(h.0).unwrap().rotation = t.rotation;
    }
    for (v, r, h) in resized.iter() {
        if let Ok(Mesh2dHandle(mesh_id)) = mesh_ids.get(h.0) {
            let mesh = meshes.get_mut(mesh_id).unwrap();
            *mesh = RegularPolygon::new(r.0 + 5., v.0).into();
            if let Ok(mut aabb) = aabb_query.get_mut(h.0) {
                *aabb = mesh.compute_aabb().unwrap();
            }
        }
    }
}

pub fn mark_visible_circles(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    marked_circles: Query<Entity, With<Visible>>,
    circles_query: Query<(), With<Radius>>,
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
) {
    if mouse_button_input.pressed(MouseButton::Left) 
    && !mouse_button_input.just_pressed(MouseButton::Left) {
        trans_query.get_mut(id.0).unwrap().translation = cursor.i.extend(1.);
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = RegularPolygon::new(cursor.i.distance(cursor.f).max(0.1), 8).into();
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        trans_query.get_mut(id.0).unwrap().translation = Vec3::Z;
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = RegularPolygon::new(0.1, 3).into();
    }
}

//optimize all those distance calls, use a distance squared instead
pub fn update_selection(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    query: Query<(Entity, &Radius, &GlobalTransform), Or<(With<Visible>, With<Selected>)>>,
    selected: Query<Entity, With<Selected>>,
    selected_query: Query<&Selected>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut top_clicked_circle: Local<Option<(Entity, f32)>>,
    id: Res<SelectionCircle>,
    mut trans_query: Query<&mut Transform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
    order_query: Query<(), With<Order>>, // non-hole circle
) {
    if keyboard_input.pressed(KeyCode::Space) { return; }
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let alt = keyboard_input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    if mouse_button_input.just_pressed(MouseButton::Left) {
        for (e, r, t) in query.iter() {
            if top_clicked_circle.is_some() {
                if t.translation().z > top_clicked_circle.unwrap().1 &&
                    cursor.i.distance(t.translation().xy()) < r.0 {
                    *top_clicked_circle = Some((e, t.translation().z));
                }
            } else {
                if cursor.i.distance(t.translation().xy()) < r.0 {
                    *top_clicked_circle = Some((e, t.translation().z));
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
            }
        }
    } else if mouse_button_input.pressed(MouseButton::Left) && top_clicked_circle.is_none() {
        trans_query.get_mut(id.0).unwrap().translation = cursor.i.extend(1.);
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = RegularPolygon::new(cursor.i.distance(cursor.f).max(0.1), 8).into();
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        trans_query.get_mut(id.0).unwrap().translation = Vec3::Z;
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = RegularPolygon::new(0.1, 3).into();
        if top_clicked_circle.is_none() {
            if !shift {
                for entity in selected.iter() {
                    commands.entity(entity).remove::<Selected>();
                }
            }
            // select those in the dragged area
            for (e, r, t) in query.iter() {
                if cursor.i.distance(cursor.f) + r.0 > cursor.i.distance(t.translation().xy()) {
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
    mut query: Query<&mut Transform, With<Selected>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    drag_modes: Res<DragModes>,
) {
    if drag_modes.t {
        if mouse_button_input.pressed(MouseButton::Left) &&
        //lol because the update to entities isn't read until the next frame
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for mut t in query.iter_mut() {
                t.translation.x += cursor.d.x;
                t.translation.y += cursor.d.y;
            }
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            for mut t in query.iter_mut() {
                t.translation.y += 1.;
            }
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            for mut t in query.iter_mut() {
                t.translation.y -= 1.;
            }
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            for mut t in query.iter_mut() {
                t.translation.x += 1.;
            }
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            for mut t in query.iter_mut() {
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
    mut query: Query<&mut Radius, With<Selected>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    drag_modes: Res<DragModes>,
) {
    if drag_modes.r {
        if mouse_button_input.pressed(MouseButton::Left)
        && !mouse_button_input.just_pressed(MouseButton::Left) {
            for mut r in query.iter_mut() {
                r.0 += cursor.d.y;
                r.0 = r.0.max(0.);
            }
        }
        if keyboard_input.any_pressed([KeyCode::ArrowUp, KeyCode::ArrowRight]) {
            for mut r in query.iter_mut() {
                r.0 += 1.;
            }
        }
        if keyboard_input.any_pressed([KeyCode::ArrowDown, KeyCode::ArrowLeft]) {
            for mut r in query.iter_mut() {
                r.0 = (r.0 - 1.).max(0.);
            }
        }
    }
}

pub fn update_vertices(
    mut query: Query<&mut Vertices, With<Selected>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    drag_modes: Res<DragModes>,
) {
    if drag_modes.v {
        if keyboard_input.any_just_pressed([KeyCode::ArrowUp, KeyCode::ArrowRight]) {
            for mut v in query.iter_mut() {
                v.0 += 1;
            }
        }
        if keyboard_input.any_just_pressed([KeyCode::ArrowDown, KeyCode::ArrowLeft]) {
            for mut v in query.iter_mut() {
                v.0 = Ord::max(v.0 - 1, 3);
            }
        }
    }
}

pub fn update_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
    mut query: Query<(Entity, &Vertices, &Radius, &mut Aabb), Or<(Changed<Vertices>, Changed<Radius>)>>,
) {
    for (id, v, r, mut aabb) in query.iter_mut() {
        if let Ok(Mesh2dHandle(mesh_id)) = mesh_ids.get(id) {
            let mesh = meshes.get_mut(mesh_id).unwrap();
            *mesh = RegularPolygon::new(r.0.max(0.1), v.0).into();
            *aabb = mesh.compute_aabb().unwrap();
        }
    }
}

pub fn update_num(
    mut query: Query<&mut Num, With<Selected>>,
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

pub fn update_order (
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Order, With<Selected>>,
    mut order_change: EventWriter<OrderChange>,
) {
    if keyboard_input.just_pressed(KeyCode::BracketRight) {
        for mut order in query.iter_mut() {
            order.0 += 1;
            order_change.send_default();
        }
    }
    if keyboard_input.just_pressed(KeyCode::BracketLeft) {
        for mut order in query.iter_mut() {
            if order.0 > 0 {
                order.0 -= 1;
                order_change.send_default();
            }
        }
    }
}

pub fn shake_order (
    keyboard_input: Res<ButtonInput<KeyCode>>,
    changed_order: Query<(Entity, &Children), With<Order>>,
    mut order_query: Query<&mut Order>,
    white_hole_query: Query<&WhiteHole>,
    mut order_change: EventWriter<OrderChange>,
) {
    if keyboard_input.just_pressed(KeyCode::Digit0) {
        for (e, children) in changed_order.iter() {
            for child in children {
                if let Ok(wh) = white_hole_query.get(*child) {
                    let this = order_query.get(e).unwrap().0;
                    let previous = order_query.get(wh.bh_parent).unwrap().0;
                    if this <= previous {
                        order_query.get_mut(e).unwrap().0 = previous + 1;
                    }
                }
            }
        }
        order_change.send_default();
    }
}

pub fn update_info_text(
    mut text_query: Query<&mut Text>,
    mut text_trans: Query<&mut Transform, Without<Radius>>,
    trans_query: Query<(&GlobalTransform, &InfoText), (Or<(Changed<Transform>, Added<InfoText>)>, With<Radius>)>,
    order_query: Query<(&Order, &InfoText), Or<(Changed<Order>, Added<InfoText>)>>,
    num_query: Query<(&Num, &InfoText), Or<(Changed<Num>, Added<InfoText>)>>,
    op_query: Query<(&Op, &InfoText), Or<(Changed<Op>, Added<InfoText>)>>,
    white_hole_query: Query<(&WhiteHole, &InfoText), Or<(Changed<WhiteHole>, Added<InfoText>)>>,
    black_hole_query: Query<&InfoText, With<BlackHole>>,
    color_query: Query<(&Col, &InfoText), Or<(Changed<Col>, Added<InfoText>)>>,
    // TODO(amy): cleanup!
    added_bh_query: Query<(&BlackHole, &InfoText), Added<InfoText>>,
    generic_wh_query: Query<&WhiteHole>,
) {
    for (trans, text) in trans_query.iter() {
        let t = trans.translation();
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

pub fn delete_selected_circles(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, (With<Selected>, With<Order>)>,
    bh_query: Query<&BlackHole, Without<Selected>>,
    wh_query: Query<&WhiteHole, Without<Selected>>,
    arrow_query: Query<&ConnectionArrow>,
    mut commands: Commands,
    mut order_change: EventWriter<OrderChange>,
    info_text_query: Query<&InfoText>,
    children_query: Query<&Children>,
    highlight_query: Query<&Highlight>,
    parent_query: Query<&Parent>,
    mut lost_wh_query: Query<&mut LostWH>,
) {
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if keyboard_input.just_pressed(KeyCode::Delete) && !shift {
        for e in query.iter() {
            if let Ok(children) = children_query.get(e) {
                for child in children {
                    // TODO(amy): do we need to remove parent?
                    if let Ok(bh) = bh_query.get(*child) {
                        if wh_query.contains(bh.wh) {
                            let arrow = arrow_query.get(bh.wh).unwrap().0;
                            commands.entity(arrow).despawn();
                            commands.entity(*child).remove_parent();
                            commands.entity(*child).despawn_recursive();
                            commands.entity(bh.wh).remove_parent();
                            commands.entity(bh.wh).despawn_recursive();
                            if let Ok(wh_text) = info_text_query.get(bh.wh) {
                                commands.entity(wh_text.0).despawn();
                            }
                            if let Ok(bh_text) = info_text_query.get(*child) {
                                commands.entity(bh_text.0).despawn();
                            }
                            if let Ok(highlight) = highlight_query.get(bh.wh) {
                                commands.entity(highlight.0).despawn();
                            }
                            if let Ok(highlight) = highlight_query.get(*child) {
                                commands.entity(highlight.0).despawn();
                            }
                            // parent of wh lost a connection
                            let parent = parent_query.get(bh.wh).unwrap();
                            lost_wh_query.get_mut(**parent).unwrap().0 = true;
                        }
                    } else if let Ok(wh) = wh_query.get(*child) {
                        if bh_query.contains(wh.bh) {
                            // don't remove things that will get removed later
                            if query.contains(wh.bh_parent) { continue; }
                            let arrow = arrow_query.get(*child).unwrap().0;
                            commands.entity(arrow).despawn();
                            commands.entity(wh.bh).remove_parent();
                            commands.entity(wh.bh).despawn_recursive();
                            commands.entity(*child).remove_parent();
                            commands.entity(*child).despawn_recursive();
                            if let Ok(wh_text) = info_text_query.get(*child) {
                                commands.entity(wh_text.0).despawn();
                            }
                            if let Ok(bh_text) = info_text_query.get(wh.bh) {
                                commands.entity(bh_text.0).despawn();
                            }
                            if let Ok(highlight) = highlight_query.get(*child) {
                                commands.entity(highlight.0).despawn();
                            }
                            if let Ok(highlight) = highlight_query.get(wh.bh) {
                                commands.entity(highlight.0).despawn();
                            }
                        }
                    }
                }
            }
            if let Ok(text) = info_text_query.get(e) {
                commands.entity(text.0).despawn();
            }
            if let Ok(highlight) = highlight_query.get(e) {
                commands.entity(highlight.0).despawn();
            }
            commands.entity(e).despawn();
            order_change.send_default();
        }
    }
}

pub fn mark_children_change(
    query: Query<&Children, (With<Order>, Changed<Transform>)>,
    mut trans_query: Query<&mut Transform, Without<Order>>,
) {
    for children in query.iter() {
        for child in children {
            trans_query.get_mut(*child).unwrap().set_changed();
        }
    }
}

