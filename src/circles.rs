use bevy::{
    render::view::VisibleEntities,
    sprite::Mesh2dHandle,
    prelude::*};

use fundsp::hacker32::*;

use crate::components::*;

macro_rules! mark_changed {
    ($n:expr, $children:expr, $bh_query:expr, $wh_query:expr) => {
        for child in $children.iter() {
            if let Ok(black_hole) = $bh_query.get(*child) {
                if black_hole.link_type == $n {
                    $wh_query.get_mut(black_hole.wh).unwrap().changed = true;
                }
            }
        }
    };
}

pub fn spawn_circles(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut depth: ResMut<Depth>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if mouse_button_input.just_released(MouseButton::Left) &&
    !keyboard_input.pressed(KeyCode::Space) {
        let radius = cursor.f.distance(cursor.i);
        let id = commands.spawn((
            ColorMesh2dBundle {
                mesh: meshes.add(bevy::prelude::shape::Circle::new(radius).into()).into(),
                material: materials.add(ColorMaterial::from(Color::hsl(0., 1.0, 0.5))),
                transform: Transform::from_translation(cursor.i.extend(depth.0)),
                ..default()
            },
            Radius(radius),
            Visible, //otherwise it can't be selected til after mark_visible is updated
            Order(0),
            OpChanged(true),
            Network(Net32::new(0,0)),
            NetIns(Vec::new()),
            crate::components::Num(0.),
            Arr(vec!(42., 105., 420., 1729.)),
            Offset {trans:Vec3::ZERO, color:Color::hsla(0.,0.,0.,0.), radius:0.},
            Op(0),
        )).id();

        // have the circle adopt a text entity
        let text = commands.spawn(Text2dBundle {
            text: Text::from_sections([
                TextSection::new(
                    id.index().to_string() + "v" + &id.generation().to_string() + "\n",
                    TextStyle { color: Color::BLACK, ..default() },
                ),
                TextSection::new(
                    "order: 0\n",
                    TextStyle { color: Color::BLACK, ..default() },
                ),
                TextSection::new(
                    "op: yaas\n",
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

        depth.0 += 0.00001;
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

pub fn duplicate_selected(
    mut commands: Commands,
    query: Query<(&Radius, &Handle<ColorMaterial>,
    &Transform, &Order, &crate::components::Num, &Arr, &Offset, &Op, &Network, &NetIns), With<Selected>>,
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
        for (radius, mat_id, trans, order, num, arr, offset, op, net, net_ins) in query.iter() {
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
                Order(order.0),
                OpChanged(true),
                Network(net.0.clone()),
                NetIns(net_ins.0.clone()),
                crate::components::Num(num.0),
                Arr(arr.0.clone().into()),
                Offset {trans: offset.trans, color: offset.color, radius: offset.radius},
                Op(op.0),
            )).id();
            let text = commands.spawn(Text2dBundle {
                text: Text::from_sections([
                    TextSection::new(
                        id.index().to_string() + "v" + &id.generation().to_string() + "\n",
                        TextStyle { color: Color::BLACK, ..default() },
                    ),
                    TextSection::new(
                        "order: 0\n",
                        TextStyle { color: Color::BLACK, ..default() },
                    ),
                    TextSection::new(
                        "op: nope\n",
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
    mut query: Query<(&mut Transform, &Children), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    black_hole_query: Query<&BlackHole>,
    mut white_hole_query: Query<&mut WhiteHole>,
) {
    if keyboard_input.pressed(KeyCode::Key1) {
        if mouse_button_input.pressed(MouseButton::Left) &&
        //lol because the update to entities isn't read until the next frame
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for (mut t, children) in query.iter_mut() {
                t.translation.x += cursor.d.x;
                t.translation.y += cursor.d.y;
                mark_changed!(-1, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Up) {
            for (mut t, children) in query.iter_mut() {
                t.translation.y += 1.;
                mark_changed!(-1, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for (mut t, children) in query.iter_mut() {
                t.translation.y -= 1.;
                mark_changed!(-1, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Right) {
            for (mut t, children) in query.iter_mut() {
                t.translation.x += 1.;
                mark_changed!(-1, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Left) {
            for (mut t, children) in query.iter_mut() {
                t.translation.x -= 1.;
                mark_changed!(-1, children, black_hole_query, white_hole_query);
            }
        }
    }
}

pub fn update_color(
    mut mats: ResMut<Assets<ColorMaterial>>,
    material_ids: Query<(&Handle<ColorMaterial>, &Children), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<&BlackHole>,
) {
    if keyboard_input.pressed(KeyCode::Key2) {
        if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for (id, children) in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_h((mat.color.h() + cursor.d.x).rem_euclid(360.));
                // mark change
                mark_changed!(-2, children, black_hole_query, white_hole_query);
            }
        }

        let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        let increment = if shift { 0.01 } else { -0.01 };
        if keyboard_input.pressed(KeyCode::Up) {
            for (id, _) in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_h((mat.color.h() + increment * 100.).rem_euclid(360.));
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for (id, _) in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_s((mat.color.s() + increment).rem_euclid(2.));
            }
        }
        if keyboard_input.pressed(KeyCode::Right) {
            for (id, _) in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_l((mat.color.l() + increment).rem_euclid(4.));
            }
        }
        if keyboard_input.pressed(KeyCode::Left) {
            for (id, _) in material_ids.iter() {
                let mat = mats.get_mut(id).unwrap();
                mat.color.set_a((mat.color.a() + increment).rem_euclid(1.));
            }
        }
    }
}

pub fn update_radius(
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<(Entity, &Children, &Mesh2dHandle), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut radius_query: Query<&mut Radius>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<&BlackHole>,
) {
    if keyboard_input.pressed(KeyCode::Key3) {
        if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for (entity, children, Mesh2dHandle(id)) in mesh_ids.iter() {
                let r = cursor.f.distance(cursor.i);
                let mesh = meshes.get_mut(id).unwrap();
                *mesh = bevy::prelude::shape::Circle::new(r).into();
                radius_query.get_mut(entity).unwrap().0 = r;
                mark_changed!(-3, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Up) {
            for (entity, children, Mesh2dHandle(id)) in mesh_ids.iter() {
                let r = radius_query.get_mut(entity).unwrap().0 + 1.;
                radius_query.get_mut(entity).unwrap().0 = r;
                let mesh = meshes.get_mut(id).unwrap();
                *mesh = bevy::prelude::shape::Circle::new(r).into();
                mark_changed!(-3, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for (entity, children, Mesh2dHandle(id)) in mesh_ids.iter() {
                let r = radius_query.get_mut(entity).unwrap().0 - 1.;
                radius_query.get_mut(entity).unwrap().0 = r;
                let mesh = meshes.get_mut(id).unwrap();
                *mesh = bevy::prelude::shape::Circle::new(r).into();
                mark_changed!(-3, children, black_hole_query, white_hole_query);
            }
        }
    }
}

pub fn update_num(
    mut query: Query<(&mut crate::components::Num, &Children), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<&BlackHole>,
) {
    if keyboard_input.pressed(KeyCode::Key4) {
        if mouse_button_input.pressed(MouseButton::Left) &&
        !mouse_button_input.just_pressed(MouseButton::Left) {
            for (mut n, children) in query.iter_mut() {
                // change the number
                n.0 += cursor.d.y / 10.;
                // inform any white holes connected through link -4 black holes
                // that our value has changed
                mark_changed!(-4, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Up) {
            for (mut n, children) in query.iter_mut() {
                n.0 += 0.01;
                mark_changed!(-4, children, black_hole_query, white_hole_query);
            }
        }
        if keyboard_input.pressed(KeyCode::Down) {
            for (mut n, children) in query.iter_mut() {
                n.0 -= 0.01;
                mark_changed!(-4, children, black_hole_query, white_hole_query);
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
                6 => { // sin
                    let freq = shared(220.);
                    n.0 = Net32::wrap(Box::new(var(&freq) >> sine()));
                    inputs.0.clear();
                    inputs.0.push(freq);
                },
                7 => { // saw
                    let freq = shared(220.);
                    n.0 = Net32::wrap(Box::new(var(&freq) >> saw()));
                    inputs.0.clear();
                    inputs.0.push(freq);
                },
                8 => { // square
                    let freq = shared(220.);
                    n.0 = Net32::wrap(Box::new(var(&freq) >> square()));
                    inputs.0.clear();
                    inputs.0.push(freq);
                },
                _ => {},
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
                -3 => "op: toRadius\n".to_string(),
                -2 => "op: toColor\n".to_string(),
                -1 => "op: toTrans\n".to_string(),
                0 => "op: nope\n".to_string(),
                1 => "op: BloomControl\n".to_string(),
                2 => "op: Tonemapping\n".to_string(),
                3 => "op: Get\n".to_string(),
                4 => "op: fromTCR\n".to_string(),
                5 => "op: Out\n".to_string(),
                6 => "op: Sin\n".to_string(),
                7 => "op: Saw\n".to_string(),
                8 => "op: Square\n".to_string(),
                _ => op.0.to_string() + "\n",
            };
        }
        if let Ok(num) = num_query.get(**parent) {
            text.sections[3].value = num.0.to_string();
        }
    }
}

pub fn delete_selected(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<(Entity, &Children), With<Selected>>,
    mut commands: Commands,
    white_hole_query: Query<&WhiteHole>,
    black_hole_query: Query<&BlackHole>,
    mut order_change: EventWriter<OrderChange>,
) {
    if keyboard_input.pressed(KeyCode::Delete) {
        for (id, children) in query.iter() {
            // if the circle we're deleting is a connection
            if let Ok(black_hole) = black_hole_query.get(id) {
                commands.entity(black_hole.wh).despawn_recursive();
            } else if let Ok(white_hole) = white_hole_query.get(id) {
                commands.entity(white_hole.bh).despawn_recursive();
            } else {
                // not a connection, despawn the holes on the other side
                for child in children.iter() {
                    if let Ok(black_hole) = black_hole_query.get(*child) {
                        commands.entity(black_hole.wh).despawn_recursive();
                    }
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        commands.entity(white_hole.bh).despawn_recursive();
                    }
                }
            }
            commands.entity(id).despawn_recursive();
            order_change.send_default();
        }
    }
}


