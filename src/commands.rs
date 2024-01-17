use bevy::{prelude::*, window::ReceivedCharacter};

use crate::components::*;

pub fn command_parser(
    keyboard_input: Res<Input<KeyCode>>,
    mut display: Query<&mut Text, With<CommandText>>,
    mut char_input_events: EventReader<ReceivedCharacter>,
) {
    for event in char_input_events.read() {
        if !event.char.is_control() {
            display.single_mut().sections[0].value.push(event.char);
        }
    }
    for c in keyboard_input.get_just_pressed() {
        match c {
            KeyCode::Escape => { display.single_mut().sections[0].value.clear(); },
            KeyCode::Return => { display.single_mut().sections[0].value.clear(); },
            KeyCode::Back => { display.single_mut().sections[0].value.pop(); },
            _ => {},
        }
    }
}
