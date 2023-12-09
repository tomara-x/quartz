use bevy::{
    prelude::*};

use crate::{circles::*, cursor::*};

pub struct ConnectionsPlugin;

impl Plugin for ConnectionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, add_connection_component);
    }
}

enum Target {
    Color,
    Pos,
    Data,
    Block,
    Radius,
}

#[derive(Component)]
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

fn add_connection_component(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    query: Query<(Entity, &Radius, &Pos), With<Visible>>,
    cursor: Res<CursorInfo>,
) {
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if ctrl && mouse_button_input.just_pressed(MouseButton::Left) {
        for (e, r, p) in query.iter() {
            if cursor.i.distance(p.value.xy()) < r.value {
                commands.entity(e).insert(Connection {
                    src: vec![e],
                    dst: vec![e],
                    src_target: Target::Color,
                    dst_target: Target::Color,
                });
                break;
            }
        }
    }
}
