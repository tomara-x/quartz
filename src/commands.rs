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
    num_query: Query<'w, 's, &'static mut crate::components::Num>,
    mats: ResMut<'w, Assets<ColorMaterial>>,
    material_ids: Query<'w, 's, &'static Handle<ColorMaterial>>,
    radius_query: Query<'w, 's, &'static mut Radius>,
    meshes: ResMut<'w, Assets<Mesh>>,
    mesh_ids: Query<'w, 's, &'static Mesh2dHandle>,
    trans_query: Query<'w, 's, &'static mut Transform>,
    arr_query: Query<'w, 's, &'static mut Arr>,
    op_changed_query: Query<'w, 's, &'static mut OpChanged>,
}

pub fn command_parser(
    keyboard_input: Res<Input<KeyCode>>,
    mut display: Query<&mut Text, With<CommandText>>,
    mut char_input_events: EventReader<ReceivedCharacter>,
    mut commands: Commands,
    entities: Query<Entity, With<Radius>>,
    mut white_hole_query: Query<&mut WhiteHole, With<Selected>>,
    mut access: Access,
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
                            }
                        }
                    },
                    Some("r") => {},
                    Some("x") => {},
                    Some("y") => {},
                    Some("z") => {},
                    Some("h") => {},
                    Some("s") => {},
                    Some("l") => {},
                    Some("a") => {},
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
