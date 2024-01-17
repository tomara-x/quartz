use bevy::{prelude::*, window::ReceivedCharacter};

use crate::components::*;

pub fn command_parser(
    keyboard_input: Res<Input<KeyCode>>,
    mut display: Query<&mut Text, With<CommandText>>,
    mut char_input_events: EventReader<ReceivedCharacter>,
) {
    let text = &mut display.single_mut().sections[0].value;
    for event in char_input_events.read() {
        if !event.char.is_control() { text.push(event.char); }
    }
    if keyboard_input.just_pressed(KeyCode::Back) { text.pop(); }
    if keyboard_input.just_pressed(KeyCode::Escape) { text.clear(); }
    if keyboard_input.just_pressed(KeyCode::Return) {
        // commands starting with :
        match text.as_str() {
            ":hi" => {
                text.push_str("hiiiiiiiii");
                info!(text);
            },
            _ => {},
        }
        text.clear();
    }
    // key commands
    match text.as_str() {
        "hi" => {text.clear();},
        _ => {},
    }
}
