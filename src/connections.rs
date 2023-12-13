use bevy::{
    prelude::*};

use crate::{circles::*, cursor::*};

pub struct ConnectionsPlugin;

impl Plugin for ConnectionsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Inputs>();
        app.register_type::<Outputs>();
        app.add_systems(Update, connect);
        app.add_systems(Update, update_connected_color);
    }
}

// they mirro each other
// use codes for input types (0 = color, 1 = ...)
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Inputs(Vec<(usize, i8, i8)>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Outputs(Vec<(usize, i8, i8)>);

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

fn connect(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    query: Query<(Entity, &Radius, &Pos), With<Visible>>,
    index_query: Query<&Index>,
    mut inputs_query: Query<&mut Inputs>,
    mut outputs_query: Query<&mut Outputs>,
    cursor: Res<CursorInfo>,
) {
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if ctrl && mouse_button_input.just_released(MouseButton::Left) {
        let mut source_entity: Option<Entity> = None;
        let mut sink_entity: Option<Entity> = None;
        for (e, r, p) in query.iter() {
            if cursor.i.distance(p.value.xy()) < r.value { source_entity = Some(e) };
            if cursor.f.distance(p.value.xy()) < r.value { sink_entity = Some(e) };
        }

        if let Some(src) = source_entity {
            if let Some(snk) = sink_entity {
                let src_index = index_query.get(src).unwrap().0;
                let snk_index = index_query.get(snk).unwrap().0;
                // source has outputs (we push to its outputs vector)
                if outputs_query.contains(src) {
                    outputs_query.get_mut(src).unwrap().0.push((snk_index, 0, 0));
                }
                else {
                    commands.entity(src).insert(Outputs(vec![(snk_index, 0, 0)]));
                }
                if inputs_query.contains(snk) {
                    inputs_query.get_mut(snk).unwrap().0.push((src_index, 0, 0));
                }
                else {
                    commands.entity(snk).insert(Inputs(vec![(src_index, 0, 0)]));
                }
            }
        }
    }
}

fn update_connected_color(
    mouse_button_input: Res<Input<MouseButton>>,
    mut inputs_query: Query<(Entity, &Inputs)>,
    entity_indices: Res<EntityIndices>,
    material_ids: Query<&Handle<ColorMaterial>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        for (entity, inputs) in inputs_query.iter() {
            //the first input's first field (entity index)
            //then we find that entity id from the resource
            let src_entity = entity_indices.0[inputs.0[0].0];
            let src_mat = mats.get(material_ids.get(src_entity).unwrap()).unwrap();
            let src_color = src_mat.color;
            let mut snk_mat = mats.get_mut(material_ids.get(entity).unwrap()).unwrap();
            snk_mat.color = src_color;
        }
    }
}
