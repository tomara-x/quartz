use bevy::{
    prelude::*,
    ecs::system::SystemParam,
    window::ReceivedCharacter,
    sprite::Mesh2dHandle,
};

use crate::components::*;

#[derive(SystemParam)]
pub struct Access<'w, 's> {
    op_query: Query<'w, 's, &'static mut Op>,
    num_query: Query<'w, 's, &'static mut Num>,
    radius_query: Query<'w, 's, (Entity, &'static mut Radius)>,
    col_query: Query<'w, 's, (Entity, &'static mut Col)>,
    trans_query: Query<'w, 's, &'static mut Transform>,
    arr_query: Query<'w, 's, &'static mut Arr>,
    order_query: Query<'w, 's, &'static mut Order>,
    selected_query: Query<'w, 's, Entity, With<Selected>>,
    radius_change_event: EventWriter<'w, RadiusChange>,
    color_change_event: EventWriter<'w, ColorChange>,
}

pub fn command_parser(
    keyboard_input: Res<Input<KeyCode>>,
    mut display: Query<&mut Text, With<CommandText>>,
    mut char_input_events: EventReader<ReceivedCharacter>,
    mut commands: Commands,
    entities: Query<Entity, With<Radius>>,
    mut white_hole_query: Query<&mut WhiteHole, With<Selected>>,
    mut access: Access,
    mut mode: Local<i32>,
    mut next_state: ResMut<NextState<Mode>>,
    mut drag_modes: ResMut<DragModes>,
) {
    if char_input_events.is_empty() { return; }
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
            let mut command = text.as_str().split_ascii_whitespace();
            match command.next() {
                Some(":hi") => {
                    match command.next() {
                        Some("hey") => {
                            text.push_str(" hiiiiiiiii");
                            info!(text);
                        },
                        _ => {},
                    }
                },
                Some(":d") => {
                    if let Some(s) = command.next() {
                        if let Ok(e) = str_to_id(s) {
                            if entities.contains(e) {
                                commands.add(DespawnCircle(e));
                            }
                        }
                    }
                },
                Some(":lt") => {
                    if let Some(b) = command.next() {
                        if let Some(w) = command.next() {
                            if let (Ok(b), Ok(w)) = (b.parse::<i32>(), w.parse::<i32>()) {
                                for mut wh in white_hole_query.iter_mut() {
                                    wh.link_types = (b, w);
                                }
                            }
                        }
                    }
                },
                Some(":set") => {
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
                                                radius.1.0 = n;
                                                access.radius_change_event.send(RadiusChange(e, radius.1.0));
                                            }
                                        }
                                    }
                                } else if let Ok(n) = s.parse::<f32>() {
                                    for id in access.selected_query.iter() {
                                        if let Ok(mut radius) = access.radius_query.get_mut(id) {
                                            radius.1.0 = n;
                                            access.radius_change_event.send(RadiusChange(id, n));
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
                        Some("op") => {},
                        Some("ord") => {},
                        _ => {},
                    }
                },
                _ => {},
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

            Some("ht") => {
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
