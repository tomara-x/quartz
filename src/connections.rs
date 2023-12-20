use bevy::{
    prelude::*};

use crate::{circles::*, cursor::*};

pub struct ConnectionsPlugin;

impl Plugin for ConnectionsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Inputs>();
        app.register_type::<Outputs>();
        app.add_systems(Update, connect.run_if(in_state(Mode::Connect)));
        //app.add_systems(Update, update_connected_color);
        app.add_systems(Update, draw_connections);
    }
}

// (entity-id, read-component, input-type, index-for-vec-components)
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Inputs(pub Vec<(usize, i16, i16, usize)>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Outputs(pub Vec<usize>);

#[derive(Component)]
pub struct WhiteHole(pub Entity);

#[derive(Component)]
pub struct BlackHole(pub Entity);

struct Connections {
    // inputs[4] -> inputs of entity 4
    // inputs[4][3] -> inputs of entity 4 connected to entity 3
    // inputs[4][3][2] -> inputs of 4 coming from 3 through input 2
    // outputs[3][4][2] -> outputs of 3 going to 4 through output 2
    inputs: Vec< Option<Vec< Option<Vec< Option<Vec<usize>> >> >>>,
    outputs: Vec< Option<Vec< Option<Vec< Option<Vec<usize>> >> >>>,
}



fn connect(
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    query: Query<(Entity, &Radius, &Transform), (With<Visible>, With<Index>)>,
    index_query: Query<&Index>,
    mut inputs_query: Query<&mut Inputs>,
    mut outputs_query: Query<&mut Outputs>,
    mut order_query: Query<&mut Order>,
    cursor: Res<CursorInfo>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rad_query: Query<&Radius>,
    trans_query: Query<&Transform>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        let mut source_entity: Option<Entity> = None;
        let mut sink_entity: Option<Entity> = None;
        for (e, r, t) in query.iter() {
            if cursor.i.distance(t.translation.xy()) < r.0 { source_entity = Some(e) };
            if cursor.f.distance(t.translation.xy()) < r.0 { sink_entity = Some(e) };
        }

        if let Some(src) = source_entity {
            if let Some(snk) = sink_entity {
                let src_index = index_query.get(src).unwrap().0;
                let snk_index = index_query.get(snk).unwrap().0;
                // source has outputs (we push to its outputs vector)
                if let Ok(mut outputs) = outputs_query.get_mut(src) {
                    outputs.0.push(snk_index);
                }
                else {
                    commands.entity(src).insert(Outputs(vec![snk_index]));
                }
                if let Ok(mut inputs) = inputs_query.get_mut(snk) {
                    inputs.0.push((src_index, 0, 0, 0));
                }
                else {
                    commands.entity(snk).insert(Inputs(vec![(src_index, 0, 0, 0)]));
                }

                // spawn connection circles
                let src_radius = rad_query.get(src).unwrap().0;
                let snk_radius = rad_query.get(snk).unwrap().0;
                let src_trans = trans_query.get(src).unwrap().translation;
                let snk_trans = trans_query.get(snk).unwrap().translation;

                let src_connection = commands.spawn(( ColorMesh2dBundle {
                        mesh: meshes.add(shape::Circle::new(src_radius * 0.1).into()).into(),
                        material: materials.add(ColorMaterial::from(Color::rgb(0.,0.,0.))),
                        transform: Transform::from_translation((cursor.i - src_trans.xy()).extend(0.000001)),
                        ..default()
                    },
                    Visible,
                    Radius(src_radius * 0.1),
                )).id();
                commands.entity(src).add_child(src_connection);

                let snk_connection = commands.spawn(( ColorMesh2dBundle {
                        mesh: meshes.add(shape::Circle::new(snk_radius * 0.1).into()).into(),
                        material: materials.add(ColorMaterial::from(Color::rgb(1.,1.,1.))),
                        transform: Transform::from_translation((cursor.f - snk_trans.xy()).extend(0.000001)),
                        ..default()
                    },
                    Visible,
                    Radius(snk_radius * 0.1),
                    WhiteHole(src_connection),
                )).id();
                commands.entity(snk).add_child(snk_connection);
                commands.entity(src_connection).insert(BlackHole(snk_connection));

                // order
                let src_order = order_query.get(src).unwrap().0;
                order_query.get_mut(snk).unwrap().0 = src_order + 1;
            }
        }
    }
}

fn draw_connections(
    mut gizmos: Gizmos,
    query: Query<(Entity, &WhiteHole), With<Visible>>,
    trans_query: Query<&GlobalTransform>,
) {
    for (snk, src) in query.iter() {
        let src_pos = trans_query.get(src.0).unwrap().translation().xy();
        let snk_pos = trans_query.get(snk).unwrap().translation().xy();
        gizmos.line_2d(src_pos, snk_pos, Color::PINK);
    }
}

//fn draw_connections(
//    mut gizmos: Gizmos,
//    query: Query<(&Transform, &Inputs), With<Visible>>,
//    pos_query: Query<&Transform>,
//    entity_indices: Res<EntityIndices>,
//) {
//    for (pos, inputs) in query.iter() {
//        for (input, _, _, _) in &inputs.0 {
//            let src_pos = pos_query.get(entity_indices.0[*input]).unwrap();
//            gizmos.line_2d(pos.translation.xy(), src_pos.translation.xy(), Color::BLUE);
//        }
//    }
//}

//fn update_connected_color(
//    mouse_button_input: Res<Input<MouseButton>>,
//    inputs_query: Query<(Entity, &Inputs)>,
//    entity_indices: Res<EntityIndices>,
//    material_ids: Query<&Handle<ColorMaterial>>,
//    mut mats: ResMut<Assets<ColorMaterial>>,
//) {
//    if mouse_button_input.pressed(MouseButton::Left) {
//        for (entity, inputs) in inputs_query.iter() {
//            //the first input's first field (entity index)
//            //then we find that entity id from the resource
//            if let Some(input) = inputs.0.get(0) {
//                let src_entity = entity_indices.0[input.0];
//                let src_mat = mats.get(material_ids.get(src_entity).unwrap()).unwrap();
//                let src_color = src_mat.color;
//                let snk_mat = mats.get_mut(material_ids.get(entity).unwrap()).unwrap();
//                snk_mat.color = src_color;
//            }
//        }
//    }
//}

