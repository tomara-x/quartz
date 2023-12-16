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
        app.add_systems(Update, draw_connections);
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Inputs(pub Vec<(usize, u8, u8)>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Outputs(pub Vec<(usize, u8, u8)>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ColorIns(pub Vec<(usize, u8)>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ColorOuts(pub Vec<(usize, u8)>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PosIns(pub Vec<(usize, u8)>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PosOuts(pub Vec<(usize, u8)>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct RadiusIns(pub Vec<(usize, u8)>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct RadiusOuts(pub Vec<(usize, u8)>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct In0(pub Vec<(usize, u8)>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Out0(pub Vec<(usize, u8)>);

#[derive(Component)]
struct Add;
#[derive(Component)]
struct Mult;
#[derive(Component)]
struct Get;

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
                if let Ok(mut outputs) = outputs_query.get_mut(src) {
                    outputs.0.push((snk_index, 255, 255));
                }
                else {
                    commands.entity(src).insert(Outputs(vec![(snk_index, 255, 255)]));
                }
                if let Ok(mut inputs) = inputs_query.get_mut(snk) {
                    inputs.0.push((src_index, 255, 255));
                }
                else {
                    commands.entity(snk).insert(Inputs(vec![(src_index, 255, 255)]));
                }
            }
        }
    }
}

fn update_connected_color(
    mouse_button_input: Res<Input<MouseButton>>,
    inputs_query: Query<(Entity, &Inputs)>,
    entity_indices: Res<EntityIndices>,
    material_ids: Query<&Handle<ColorMaterial>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        for (entity, inputs) in inputs_query.iter() {
            //the first input's first field (entity index)
            //then we find that entity id from the resource
            if let Some(input) = inputs.0.get(0) {
                let src_entity = entity_indices.0[input.0];
                let src_mat = mats.get(material_ids.get(src_entity).unwrap()).unwrap();
                let src_color = src_mat.color;
                let snk_mat = mats.get_mut(material_ids.get(entity).unwrap()).unwrap();
                snk_mat.color = src_color;
            }
        }
    }
}

fn draw_connections(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Inputs), With<Visible>>,
    pos_query: Query<&Transform>,
    entity_indices: Res<EntityIndices>,
) {
    for (pos, inputs) in query.iter() {
        for (input, _, _) in &inputs.0 {
            let src_pos = pos_query.get(entity_indices.0[*input]).unwrap();
            gizmos.line_2d(pos.translation.xy(), src_pos.translation.xy(), Color::BLUE);
        }
    }
}

