use bevy::{
    render::view::VisibleEntities,
    sprite::Mesh2dHandle,
    prelude::*};

use fundsp::hacker32::*;

use crate::components::*;

pub fn spawn_circles(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut depth: Local<f32>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if mouse_button_input.just_released(MouseButton::Left) &&
    !keyboard_input.pressed(KeyCode::Space) {
        let radius = cursor.f.distance(cursor.i);
        let color = Color::hsla(300., 1., 0.5, 1.);
        let id = commands.spawn((
            ColorMesh2dBundle {
                mesh: meshes.add(bevy::prelude::shape::Circle::new(radius).into()).into(),
                material: materials.add(ColorMaterial::from(color)),
                transform: Transform::from_translation(cursor.i.extend(*depth)),
                ..default()
            },
            Radius(radius),
            Col(color),
            Visible, //otherwise it can't be selected til after mark_visible is updated
            Order(0),
            OpChanged(true),
            Network(Net32::new(0,1)),
            NetIns(Vec::new()),
            crate::components::Num(0.),
            Arr(vec!(42., 105., 420., 1729.)),
            Op(0),
        )).id();

        // have the circle adopt a text entity
        let text = commands.spawn(Text2dBundle {
            text: Text::from_sections([
                TextSection::new(
                    id.index().to_string() + "v" + &id.generation().to_string() + "\n",
                    TextStyle { color: Color::BLACK, font_size: 18., ..default() },
                ),
                TextSection::new(
                    "order: 0\n",
                    TextStyle { color: Color::BLACK, ..default() },
                ),
                TextSection::new(
                    "op: empty\n",
                    TextStyle { color: Color::BLACK, ..default() },
                ),
                TextSection::new(
                    "0",
                    TextStyle { color: Color::BLACK, ..default() },
                ),
            ]),
            transform: Transform::from_translation(Vec3{z:0.000001, ..default()}),
            ..default()
        }).id();
        commands.entity(id).add_child(text);

        *depth += 0.00001;
    }
}

pub fn highlight_selected(
    mut gizmos: Gizmos,
    time: Res<Time>,
    query: Query<(&Radius, &GlobalTransform), With<Selected>>,
) {
    for (r, t) in query.iter() {
        let color = Color::hsl((time.elapsed_seconds() * 100.) % 360., 1.0, 0.5);
        gizmos.circle_2d(t.translation().xy(), r.0, color).segments(64);
    }
}

// loop over the visible entities and give them a Visible component
// so we can query just the visible entities
pub fn mark_visible(
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
        for e in vis.iter() {
            commands.entity(*e).insert(Visible);
        }
    }
}

//optimize all those distance calls, use a distance squared instead
pub fn update_selection(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<(Entity, &Radius, &GlobalTransform), Or<(With<Visible>, With<Selected>)>>,
    selected: Query<Entity, With<Selected>>,
    selected_query: Query<&Selected>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<Input<KeyCode>>,
    mut top_clicked_circle: Local<Option<(Entity, f32)>>,
) {
    if keyboard_input.pressed(KeyCode::Space) { return; }
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
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
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        if top_clicked_circle.is_none() {
            if !shift {
                for entity in selected.iter() {
                    commands.entity(entity).remove::<Selected>();
                }
            }
            // select those in the dragged area
            for (e, r, t) in query.iter() {
                if cursor.i.distance(cursor.f) + r.0 > cursor.i.distance(t.translation().xy()) {
                    commands.entity(e).insert(Selected);
                }
            }
        }
        *top_clicked_circle = None;
    }
}

pub fn select_all(
    mut commands: Commands,
    order_query: Query<Entity, With<Order>>,
    connection_query: Query<Entity, Or<(With<BlackHole>, With<WhiteHole>)>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if ctrl && keyboard_input.pressed(KeyCode::A) {
        if shift {
            for e in connection_query.iter() { commands.entity(e).insert(Selected); }
        } else {
            for e in order_query.iter() { commands.entity(e).insert(Selected); }
        }
    }
}

// HAZARDOUS!
pub fn duplicate_selected(
    mut commands: Commands,
    query: Query<(&Radius, &Handle<ColorMaterial>,
    &Transform, &crate::components::Num, &Arr), With<Selected>>,
    selected_query: Query<Entity, With<Selected>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if ctrl && keyboard_input.just_pressed(KeyCode::D) {
        for e in selected_query.iter() {
            commands.entity(e).remove::<Selected>();
        }
        for (radius, mat_id, trans, num, arr) in query.iter() {
            let color = materials.get(mat_id).unwrap().color;
            let id = commands.spawn((
                ColorMesh2dBundle {
                    mesh: meshes.add(bevy::prelude::shape::Circle::new(radius.0).into()).into(),
                    material: materials.add(ColorMaterial::from(color)),
                    transform: Transform::from_translation(
                        Vec3 {z: trans.translation.z + 1., ..trans.translation }),
                    ..default()
                },
                Radius(radius.0),
                Visible,
                Selected,
                Order(0),
                OpChanged(true),
                Network(Net32::new(0,1)),
                NetIns(Vec::new()),
                crate::components::Num(num.0),
                Arr(arr.0.clone().into()),
                Op(0),
            )).id();
            let text = commands.spawn(Text2dBundle {
                text: Text::from_sections([
                    TextSection::new(
                        id.index().to_string() + "v" + &id.generation().to_string() + "\n",
                        TextStyle { color: Color::BLACK, font_size: 18., ..default() },
                    ),
                    TextSection::new(
                        "order: 0\n",
                        TextStyle { color: Color::BLACK, ..default() },
                    ),
                    TextSection::new(
                        "op: empty\n",
                        TextStyle { color: Color::BLACK, ..default() },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle { color: Color::BLACK, ..default() },
                    ),
                ]),
                transform: Transform::from_translation(Vec3{z:0.000001, ..default()}),
                ..default()
            }).id();
            commands.entity(id).add_child(text);
        }
    }
}

pub fn move_selected(
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
    mut query: Query<&mut Transform, With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
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
        if keyboard_input.pressed(KeyCode::Up) {
            for mut t in query.iter_mut() {
                t.translation.y += 1.;
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for mut t in query.iter_mut() {
                t.translation.y -= 1.;
            }
        }
        if keyboard_input.pressed(KeyCode::Right) {
            for mut t in query.iter_mut() {
                t.translation.x += 1.;
            }
        }
        if keyboard_input.pressed(KeyCode::Left) {
            for mut t in query.iter_mut() {
                t.translation.x -= 1.;
            }
        }
    }
}

pub fn update_color(
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
    mut query: Query<(Entity, &mut Col), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    drag_modes: Res<DragModes>,
    mut color_change_event: EventWriter<ColorChange>,
) {
    if mouse_button_input.pressed(MouseButton::Left)
    && !mouse_button_input.just_pressed(MouseButton::Left) {
        if drag_modes.h {
            for (e, mut c) in query.iter_mut() {
                let h = (c.0.h() + cursor.d.x).clamp(0., 360.);
                c.0.set_h(h);
                color_change_event.send(ColorChange(e, c.0));
            }
        }
        if drag_modes.s {
            for (e, mut c) in query.iter_mut() {
                let s = (c.0.s() + cursor.d.x / 100.).clamp(0., 1.);
                c.0.set_s(s);
                color_change_event.send(ColorChange(e, c.0));
            }
        }
        if drag_modes.l {
            for (e, mut c) in query.iter_mut() {
                let l = (c.0.l() + cursor.d.x / 100.).clamp(0., 1.);
                c.0.set_l(l);
                color_change_event.send(ColorChange(e, c.0));
            }
        }
        if drag_modes.a {
            for (e, mut c) in query.iter_mut() {
                let a = (c.0.a() + cursor.d.x / 100.).clamp(0., 1.);
                c.0.set_a(a);
                color_change_event.send(ColorChange(e, c.0));
            }
        }
    }
    if keyboard_input.any_pressed([KeyCode::Left, KeyCode::Down]) {
        for (e, mut c) in query.iter_mut() {
            if drag_modes.h {
                let h = (c.0.h() - 1.).clamp(0., 360.);
                c.0.set_h(h);
                color_change_event.send(ColorChange(e, c.0));
            }
            if drag_modes.s {
                let s = (c.0.s() - 0.01).clamp(0., 1.);
                c.0.set_s(s);
                color_change_event.send(ColorChange(e, c.0));
            }
            if drag_modes.l {
                let l = (c.0.l() - 0.01).clamp(0., 1.);
                c.0.set_l(l);
                color_change_event.send(ColorChange(e, c.0));
            }
            if drag_modes.a {
                let a = (c.0.a() - 0.01).clamp(0., 1.);
                c.0.set_a(a);
                color_change_event.send(ColorChange(e, c.0));
            }
        }
    }
    if keyboard_input.any_pressed([KeyCode::Right, KeyCode::Up]) {
        for (e, mut c) in query.iter_mut() {
            if drag_modes.h {
                let h = (c.0.h() + 1.).clamp(0., 360.);
                c.0.set_h(h);
                color_change_event.send(ColorChange(e, c.0));
            }
            if drag_modes.s {
                let s = (c.0.s() + 0.01).clamp(0., 1.);
                c.0.set_s(s);
                color_change_event.send(ColorChange(e, c.0));
            }
            if drag_modes.l {
                let l = (c.0.l() + 0.01).clamp(0., 1.);
                c.0.set_l(l);
                color_change_event.send(ColorChange(e, c.0));
            }
            if drag_modes.a {
                let a = (c.0.a() + 0.01).clamp(0., 1.);
                c.0.set_a(a);
                color_change_event.send(ColorChange(e, c.0));
            }
        }
    }
}

pub fn update_mat_from_color(
    mut mats: ResMut<Assets<ColorMaterial>>,
    material_ids: Query<&Handle<ColorMaterial>>,
    mut color_change_event: EventReader<ColorChange>,
) {
    for event in color_change_event.read() {
        let id = material_ids.get(event.0).unwrap();
        let mat = mats.get_mut(id).unwrap();
        mat.color = event.1;
    }
}

pub fn update_radius(
    mut query: Query<(Entity, &mut Radius), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
    drag_modes: Res<DragModes>,
    mut radius_change_event: EventWriter<RadiusChange>,
) {
    if drag_modes.r {
        if mouse_button_input.pressed(MouseButton::Left)
        && !mouse_button_input.just_pressed(MouseButton::Left) {
            for (e, mut r) in query.iter_mut() {
                r.0 = cursor.f.distance(cursor.i);
                radius_change_event.send(RadiusChange(e, r.0));
            }
        }
        if keyboard_input.any_pressed([KeyCode::Up, KeyCode::Right]) {
            for (e, mut r) in query.iter_mut() {
                r.0 += 1.;
                radius_change_event.send(RadiusChange(e, r.0));
            }
        }
        if keyboard_input.any_pressed([KeyCode::Down, KeyCode::Left]) {
            for (e, mut r) in query.iter_mut() {
                r.0 -= 1.;
                radius_change_event.send(RadiusChange(e, r.0));
            }
        }
    }
}

pub fn update_mesh_from_radius(
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
    mut radius_change_event: EventReader<RadiusChange>,
) {
    for event in radius_change_event.read() {
        let Mesh2dHandle(id) = mesh_ids.get(event.0).unwrap();
        let mesh = meshes.get_mut(id).unwrap();
        *mesh = bevy::prelude::shape::Circle::new(event.1).into();
    }
}

pub fn update_num(
    mut query: Query<&mut crate::components::Num, With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
    drag_modes: Res<DragModes>,
) {
    if drag_modes.n {
        if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for mut n in query.iter_mut() {
                n.0 += cursor.d.y / 10.;
            }
        }
        if keyboard_input.pressed(KeyCode::Up) {
            for mut n in query.iter_mut() {
                n.0 += 0.01;
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for mut n in query.iter_mut() {
                n.0 -= 0.01;
            }
        }
    }
}

pub fn update_op(
    mut query: Query<(&mut Op, &mut OpChanged,
    &mut Network, &mut NetIns), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.any_just_pressed([KeyCode::O, KeyCode::P]) {
        let increment = if keyboard_input.just_pressed(KeyCode::O) { -1 } else { 1 };
        for (mut op, mut op_changed, mut n, mut inputs) in query.iter_mut() {
            op.0 += increment;
            op_changed.0 = true;
            // can we use duplicate macros here?
            match op.0 {
                1 => { // Var
                    let input = shared(0.);
                    n.0 = Net32::wrap(Box::new(var(&input)));
                    inputs.0.clear();
                    inputs.0.push(input);
                },
                _ => {
                    n.0 = Net32::wrap(Box::new(dc(0.)));
                    inputs.0.clear();
                },
            }
        }
    }
}

pub fn update_order (
    keyboard_input: Res<Input<KeyCode>>,
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

pub fn update_circle_text(
    mut query: Query<(&mut Text, &Parent), With<Visible>>,
    order_query: Query<&Order>,
    num_query: Query<&crate::components::Num>,
    op_query: Query<&Op>,
) {
    for (mut text, parent) in query.iter_mut() {
        if let Ok(order) = order_query.get(**parent) {
            text.sections[1].value = "order: ".to_string() + &order.0.to_string() + "\n";
        }
        if let Ok(op) = op_query.get(**parent) {
            text.sections[2].value = match op.0 {
                -10 => "op: pass\n".to_string(),
                -9 => "op: sum\n".to_string(),
                -8 => "op: tonemapping\n".to_string(),
                -7 => "op: bloom\n".to_string(),
                -6 => "op: set\n".to_string(),
                -5 => "op: get\n".to_string(),
                -4 => "op: fromTCR\n".to_string(),
                -3 => "op: to_radius\n".to_string(),
                -2 => "op: to_color\n".to_string(),
                -1 => "op: to_trans\n".to_string(),
                0 => "op: empty\n".to_string(),
                1 => "op: Var\n".to_string(),
                2 => "op: Oscil\n".to_string(),
                3 => "op: Sum\n".to_string(),
                4 => "op: Product\n".to_string(),
                5 => "op: Out\n".to_string(),
                6 => "op: Probe\n".to_string(),
                _ => op.0.to_string() + "\n",
            };
        }
        if let Ok(num) = num_query.get(**parent) {
            text.sections[3].value = num.0.to_string();
        }
    }
}

// HAZARDOUS!
pub fn remove_connections(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<&Children, With<Selected>>,
    mut commands: Commands,
    white_hole_query: Query<&WhiteHole>,
    black_hole_query: Query<&BlackHole>,
) {
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if shift && keyboard_input.just_pressed(KeyCode::Delete) {
        for children in query.iter() {
            for child in children {
                if let Ok(wh) = white_hole_query.get(*child) {
                    commands.entity(wh.bh).remove_parent();
                    commands.entity(wh.bh).despawn_recursive();
                    commands.entity(*child).remove_parent();
                    commands.entity(*child).despawn_recursive();
                } else if let Ok(bh) = black_hole_query.get(*child) {
                    commands.entity(bh.wh).remove_parent();
                    commands.entity(bh.wh).despawn_recursive();
                    commands.entity(*child).remove_parent();
                    commands.entity(*child).despawn_recursive();
                }
            }
        }
    }
}

pub fn delete_selected(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<Entity, With<Selected>>,
    mut commands: Commands,
    mut order_change: EventWriter<OrderChange>,
    //white_hole_query: Query<&WhiteHole>,
    //black_hole_query: Query<&BlackHole>,
) {
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if !shift && keyboard_input.just_pressed(KeyCode::Delete) {
        for id in query.iter() {
            commands.add(DespawnCircle(id));
            order_change.send_default();
            //if let Ok(wh) = white_hole_query.get(id) {
            //    commands.entity(wh.bh).despawn_recursive();
            //    commands.entity(id).despawn_recursive();
            //} else if let Ok(bh) = black_hole_query.get(id) {
            //    commands.entity(bh.wh).despawn_recursive();
            //    commands.entity(id).despawn_recursive();
            //} else {
            //    for child in children {
            //        if let Ok(wh) = white_hole_query.get(*child) {
            //            commands.entity(wh.bh).despawn_recursive();
            //            commands.entity(*child).despawn_recursive();
            //        } else if let Ok(bh) = black_hole_query.get(*child) {
            //            commands.entity(bh.wh).despawn_recursive();
            //            commands.entity(*child).despawn_recursive();
            //        }
            //    }
            //    commands.entity(id).despawn_recursive();
            //    order_change.send_default();
            //}
        }
    }
}


