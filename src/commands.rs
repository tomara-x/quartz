use bevy::{
    prelude::*,
    ecs::system::SystemParam,
    window::ReceivedCharacter,
};

use crate::components::*;

#[derive(SystemParam)]
pub struct Access<'w, 's> {
    op_query: Query<'w, 's, (Entity, &'static mut Op)>,
    num_query: Query<'w, 's, &'static mut Num>,
    radius_query: Query<'w, 's, (Entity, &'static mut Radius)>,
    col_query: Query<'w, 's, (Entity, &'static mut Col)>,
    trans_query: Query<'w, 's, &'static mut Transform>,
    arr_query: Query<'w, 's, &'static mut Arr>,
    order_query: Query<'w, 's, &'static mut Order>,
    selected_query: Query<'w, 's, Entity, With<Selected>>,
    white_hole_query: Query<'w, 's, &'static mut WhiteHole>,
    black_hole_query: Query<'w, 's, &'static mut BlackHole>,
    color_change_event: EventWriter<'w, ColorChange>,
    order_change: EventWriter<'w, OrderChange>,
    vertices_query: Query<'w, 's, (Entity, &'static mut Vertices)>,
}

pub fn command_parser(
    keyboard_input: Res<Input<KeyCode>>,
    mut display: Query<&mut Text, With<CommandText>>,
    mut char_input_events: EventReader<ReceivedCharacter>,
    mut commands: Commands,
    entities: Query<Entity, With<Radius>>,
    mut access: Access,
    mut mode: Local<i32>,
    mut in_progress: Local<bool>,
    mut next_state: ResMut<NextState<Mode>>,
    mut drag_modes: ResMut<DragModes>,
) {
    if char_input_events.is_empty() && !*in_progress { return; }
    let text = &mut display.single_mut().sections[0].value;

    // draw mode
    if *mode == 1 {
        // exit to edit
        if keyboard_input.any_just_pressed([KeyCode::Escape, KeyCode::E]) {
            text.clear();
            *mode = 0;
            next_state.set(Mode::Edit);
            char_input_events.clear(); // we have an 'e' that we don't want
        }
        // switch to connect mode
        if keyboard_input.just_pressed(KeyCode::C) {
            *text = "-- CONNECT --".to_string();
            *mode = 2;
            next_state.set(Mode::Connect);
        }
    }

    // connect mode
    if *mode == 2 {
        // exit to edit
        if keyboard_input.any_just_pressed([KeyCode::Escape, KeyCode::E]) {
            text.clear();
            *mode = 0;
            next_state.set(Mode::Edit);
            char_input_events.clear();
        }
        // switch to draw mode
        if keyboard_input.just_pressed(KeyCode::D) {
            *text = "-- DRAW --".to_string();
            *mode = 1;
            next_state.set(Mode::Draw);
        }
    }

    // edit mode
    if *mode == 0 {
        for event in char_input_events.read() {
            if !event.char.is_control() { text.push(event.char); }
        }
        if keyboard_input.just_pressed(KeyCode::Back) { text.pop(); }
        if keyboard_input.just_pressed(KeyCode::Escape) { text.clear(); }
        if keyboard_input.just_pressed(KeyCode::Return) {
            // commands starting with :
            let lines = text.as_str().split("|");
            for line in lines {
                let mut command = line.split_ascii_whitespace();
                match command.next() {
                    Some(":d") | Some("d") => {
                        if let Some(s) = command.next() {
                            if let Ok(e) = str_to_id(s) {
                                if entities.contains(e) {
                                    commands.add(DespawnCircle(e));
                                }
                            }
                        }
                    },
                    Some(":lt") | Some("lt") => {
                        if let Some(s) = command.next() {
                            if let Ok(e) = str_to_id(s) {
                                if let Ok(mut wh) = access.white_hole_query.get_mut(e) {
                                    if let Some(s) = command.next() {
                                        wh.link_types.1 = str_to_lt(s);
                                    }
                                } else if let Ok(bh) = access.black_hole_query.get(e) {
                                    let wh = &mut access.white_hole_query.get_mut(bh.wh).unwrap();
                                    if let Some(s) = command.next() {
                                        wh.link_types.0 = str_to_lt(s);
                                    }
                                }
                            } else {
                                for id in access.selected_query.iter() {
                                    if let Ok(mut wh) = access.white_hole_query.get_mut(id) {
                                        wh.link_types.1 = str_to_lt(s);
                                    } else if let Ok(bh) = access.black_hole_query.get(id) {
                                        let wh = &mut access.white_hole_query.get_mut(bh.wh).unwrap();
                                        wh.link_types.0 = str_to_lt(s);
                                    }
                                }
                            }
                        }
                    },
                    Some(":set") | Some("set") => {
                        match command.next() {
                            Some("n") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut num) = access.num_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    num.0 = n;
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut num) = access.num_query.get_mut(id) {
                                                num.0 = n;
                                            }
                                        }
                                    }
                                }
                            },
                            Some("r") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut radius) = access.radius_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    radius.1.0 = n.max(0.);
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut radius) = access.radius_query.get_mut(id) {
                                                radius.1.0 = n.max(0.);
                                            }
                                        }
                                    }
                                }
                            },
                            Some("x") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut t) = access.trans_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    t.translation.x = n;
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut t) = access.trans_query.get_mut(id) {
                                                t.translation.x = n;
                                            }
                                        }
                                    }
                                }
                            },
                            Some("y") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut t) = access.trans_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    t.translation.y = n;
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut t) = access.trans_query.get_mut(id) {
                                                t.translation.y = n;
                                            }
                                        }
                                    }
                                }
                            },
                            Some("z") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut t) = access.trans_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    t.translation.z = n;
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut t) = access.trans_query.get_mut(id) {
                                                t.translation.z = n;
                                            }
                                        }
                                    }
                                }
                            },
                            Some("h") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut color) = access.col_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    color.1.0.set_h(n);
                                                    access.color_change_event.send(ColorChange(e, color.1.0));
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut color) = access.col_query.get_mut(id) {
                                                color.1.0.set_h(n);
                                                access.color_change_event.send(ColorChange(id, color.1.0));
                                            }
                                        }
                                    }
                                }
                            },
                            Some("s") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut color) = access.col_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    color.1.0.set_s(n);
                                                    access.color_change_event.send(ColorChange(e, color.1.0));
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut color) = access.col_query.get_mut(id) {
                                                color.1.0.set_s(n);
                                                access.color_change_event.send(ColorChange(id, color.1.0));
                                            }
                                        }
                                    }
                                }
                            },
                            Some("l") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut color) = access.col_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    color.1.0.set_l(n);
                                                    access.color_change_event.send(ColorChange(e, color.1.0));
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut color) = access.col_query.get_mut(id) {
                                                color.1.0.set_l(n);
                                                access.color_change_event.send(ColorChange(id, color.1.0));
                                            }
                                        }
                                    }
                                }
                            },
                            Some("a") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut color) = access.col_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    color.1.0.set_a(n);
                                                    access.color_change_event.send(ColorChange(e, color.1.0));
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut color) = access.col_query.get_mut(id) {
                                                color.1.0.set_a(n);
                                                access.color_change_event.send(ColorChange(id, color.1.0));
                                            }
                                        }
                                    }
                                }
                            },
                            Some("v") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut vertices) = access.vertices_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<usize>() {
                                                    vertices.1.0 = n.max(3);
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<usize>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut vertices) = access.vertices_query.get_mut(id) {
                                                vertices.1.0 = n.max(3);
                                            }
                                        }
                                    }
                                }
                            },
                            Some("o") | Some("rot") | Some("rotation") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut t) = access.trans_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    t.rotation = Quat::from_rotation_z(n);
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut t) = access.trans_query.get_mut(id) {
                                                t.rotation = Quat::from_rotation_z(n);
                                            }
                                        }
                                    }
                                }
                            },
                            Some("op") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut op) = access.op_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                op.1.0 = n.to_string();
                                            }
                                        }
                                    } else {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut op) = access.op_query.get_mut(id) {
                                                op.1.0 = s.to_string();
                                            }
                                        }
                                    }
                                }
                            },
                            Some("ord") | Some("order") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut order) = access.order_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<usize>() {
                                                    order.0 = n;
                                                    access.order_change.send_default();
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<usize>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut order) = access.order_query.get_mut(id) {
                                                order.0 = n;
                                                access.order_change.send_default();
                                            }
                                        }
                                    }
                                }
                            },
                            Some("arr") | Some("array") => {
                                if let Some(s) = command.next() {
                                    if let Ok(e) = str_to_id(s) {
                                        if let Ok(mut arr) = access.arr_query.get_mut(e) {
                                            arr.0.clear();
                                            for n in command {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    arr.0.push(n);
                                                }
                                            }
                                        }
                                    } else {
                                        let mut tmp = Vec::new();
                                        if let Ok(n) = s.parse::<f32>() { tmp.push(n); }
                                        for n in command {
                                            if let Ok(n) = n.parse::<f32>() {
                                                tmp.push(n);
                                            }
                                        }
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut arr) = access.arr_query.get_mut(id) {
                                                arr.0 = tmp.clone();
                                            }
                                        }
                                    }
                                }
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                }
            }
            text.clear();
        }
        // key commands
        let mut command = text.as_str().split_ascii_whitespace();
        match command.next() {
            Some("d") => {
                *mode = 1;
                next_state.set(Mode::Draw);
                *text = "-- DRAW --".to_string();
            },
            Some("c") => {
                *mode = 2;
                next_state.set(Mode::Connect);
                *text = "-- CONNECT --".to_string();
            },
            Some("et") => {
                drag_modes.falsify();
                drag_modes.t = true;
                text.clear();
            },
            Some("er") => {
                drag_modes.falsify();
                drag_modes.r = true;
                text.clear();
            },
            Some("en") => {
                drag_modes.falsify();
                drag_modes.n = true;
                text.clear();
            },
            Some("eh") => {
                drag_modes.falsify();
                drag_modes.h = true;
                text.clear();
            },
            Some("es") => {
                drag_modes.falsify();
                drag_modes.s = true;
                text.clear();
            },
            Some("el") => {
                drag_modes.falsify();
                drag_modes.l = true;
                text.clear();
            },
            Some("ea") => {
                drag_modes.falsify();
                drag_modes.a = true;
                text.clear();
            },
            Some("eo") => {
                drag_modes.falsify();
                drag_modes.o = true;
                text.clear();
            },

            Some("ee") => {
                drag_modes.falsify();
                text.clear();
            },

            Some("Et") => {
                drag_modes.t = true;
                text.clear();
            },
            Some("Er") => {
                drag_modes.r = true;
                text.clear();
            },
            Some("En") => {
                drag_modes.n = true;
                text.clear();
            },
            Some("Eh") => {
                drag_modes.h = true;
                text.clear();
            },
            Some("Es") => {
                drag_modes.s = true;
                text.clear();
            },
            Some("El") => {
                drag_modes.l = true;
                text.clear();
            },
            Some("Ea") => {
                drag_modes.a = true;
                text.clear();
            },
            Some("eO") => {
                drag_modes.o = true;
                text.clear();
            },

            Some("ht") => {
                for id in access.selected_query.iter() {
                    if let Ok(mut wh) = access.white_hole_query.get_mut(id) {
                        wh.open = !wh.open;
                    }
                }
                text.clear();
            },
            Some("hf") => {
                if *in_progress {
                    for id in access.selected_query.iter() {
                        if let Ok(mut wh) = access.white_hole_query.get_mut(id) {
                            wh.open = false;
                        }
                    }
                    text.clear();
                    *in_progress = false;
                } else {
                    for id in access.selected_query.iter() {
                        if let Ok(mut wh) = access.white_hole_query.get_mut(id) {
                            wh.open = true;
                        }
                    }
                    *in_progress = true;
                }
            },
            Some("o") => {
                *text = ":set op ".to_string();
            },
            Some("l") => {
                *text = ":lt ".to_string();
            }
            Some("[") | Some("]") => {
                text.clear();
            },
            _ => {},
        }
    }
}

fn str_to_id(s: &str) -> Result<Entity, &str> {
    let mut e = s.split('v');
    if let Some(i) = e.next() {
        if let Some(g) = e.next() {
            if let Ok(index) = i.parse::<u64>() {
                if let Ok(gen) = g.parse::<u64>() {
                    let bits = gen << 32 | index;
                    return Ok(Entity::from_bits(bits));
                }
            }
        }
    }
    return Err("errrrr");
}
