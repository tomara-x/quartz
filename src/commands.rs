use bevy::{prelude::*, window::ReceivedCharacter};

use crate::components::*;

pub fn command_parser(
    keyboard_input: Res<Input<KeyCode>>,
    mut display: Query<&mut Text, With<CommandText>>,
    mut char_input_events: EventReader<ReceivedCharacter>,
    mut commands: Commands,
    entities: Query<Entity, With<Radius>>,
    mut white_hole_query: Query<&mut WhiteHole, With<Selected>>,
) {
    if char_input_events.is_empty() { return; }
    let text = &mut display.single_mut().sections[0].value;
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
            Some(":yeet") => {
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
                        if let Ok(b) = b.parse::<i32>() {
                            if let Ok(w) = w.parse::<i32>() {
                                for mut wh in white_hole_query.iter_mut() {
                                    wh.link_types = (b, w);
                                }
                            }
                        }
                    }
                }
            },
            _ => {},
        }
        text.clear();
    }
    // key commands
    let mut command = text.as_str().split_ascii_whitespace();
    match command.next() {
        Some("hi") => {text.clear();},
        Some("ht") => {
        },
        _ => {},
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
