use bevy::{prelude::*, window::ReceivedCharacter};

use crate::components::*;

pub fn command_parser(
    keyboard_input: Res<Input<KeyCode>>,
    mut display: Query<&mut Text, With<CommandText>>,
    mut char_input_events: EventReader<ReceivedCharacter>,
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
            _ => {},
        }
        text.clear();
    }
    // key commands
    let mut command = text.as_str().split_ascii_whitespace();
    match command.next() {
        Some("hi") => {text.clear();},
        _ => {},
    }
}
