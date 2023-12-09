use bevy::{
    prelude::*};

use crate::{circles::*, cursor::*};

pub struct ConnectionsPlugin;

impl Plugin for ConnectionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, create_connection);
    }
}

#[derive(Debug)]
enum Target {
    Color,
    Pos,
    Data,
    Block,
    Radius,
}

#[derive(Component, Debug)]
struct Connection {
    src: Vec<Entity>,
    dst: Vec<Entity>,
    src_target: Target,
    dst_target: Target,
}

//operations as marker components
#[derive(Component)]
struct Add;
#[derive(Component)]
struct Mult;

//not working
fn create_connection(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    mut query: Query<(Entity, &Radius, &Pos, Option<&mut Connection>), With<Visible>>,
    cursor: Res<CursorInfo>,
) {
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if ctrl && mouse_button_input.just_pressed(MouseButton::Left) {
        for (e, r, p, mut c) in query.iter_mut() {
            if cursor.i.distance(p.value.xy()) < r.value {
                match c {
                    None => {commands.entity(e).insert(Connection {
                                src: vec![e],
                                dst: vec![],
                                src_target: Target::Color,
                                dst_target: Target::Color,
                            });
                    ()},
                    Some(ref mut connection) => connection.src.push(e),
                }
                break;
            }
        }
    }
    if ctrl && mouse_button_input.just_released(MouseButton::Left) {
        for (e, r, p, mut c) in query.iter_mut() {
            if cursor.f.distance(p.value.xy()) < r.value {
                match c {
                    None => {commands.entity(e).insert(Connection {
                                src: vec![],
                                dst: vec![e],
                                src_target: Target::Color,
                                dst_target: Target::Color,
                            });
                    ()},
                    Some(ref mut connection) => connection.src.push(e),
                }
                println!("{:?}", c);
                break;
            }
        }
    }
}
