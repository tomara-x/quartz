use bevy::{
    prelude::*};

use crate::{circles::*, cursor::*};

pub struct ConnectionsPlugin;

impl Plugin for ConnectionsPlugin {
    fn build(&self, app: &mut App) {
        //app.add_systems(Update, create_connection);
    }
}

// they mirro each other
#[derive(Component, Debug)]
struct Inputs(Vec<(Entity, String, String)>);

#[derive(Component, Debug)]
struct Outputs(Vec<(Entity, String, String)>);

//operations as marker components
#[derive(Component)]
struct Add; //"inputA" + "inputB" (if an input is block, block output)
#[derive(Component)]
struct Mult;
#[derive(Component)]
struct Get; //takes a vector to "input" and a num index to "index"

// a Ready component for entities
// query all with no inputs and set them to ready
// then loop until all are ready

/*
fn create_connection(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    mut query: Query<(Entity, &Radius, &Pos, Option<&mut Connection>), With<Visible>>,
    cursor: Res<CursorInfo>,
) {
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if ctrl && mouse_button_input.just_released(MouseButton::Left) {
        let Option(source_entity);
        let Option(sink_entity);
        for (e, r, p, mut c) in query.iter_mut() {
            if cursor.i.distance(p.value.xy()) < r.value { source_entity = e };
            let sink_entity = if cursor.f.distance(p.value.xy()) < r.value { e };
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
                continue;
            }
                match c {
                    None => {commands.entity(e).insert(Connection {
                                src: vec![],
                                dst: vec![e],
                                src_target: Target::Color,
                                dst_target: Target::Color,
                            });
                    ()},
                    Some(ref mut connection) => connection.dst.push(e),
                }
                println!("{:?}", c);
                break;
            }
        }
    }
}
*/
