use bevy::{
    sprite::Mesh2dHandle,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
        },
    prelude::*};

use crate::components::*;

macro_rules! mark_changed {
    ($n:expr, $children:expr, $bh_query:expr, $wh_query:expr) => {
        for child in $children.iter() {
            if let Ok(black_hole) = $bh_query.get(*child) {
                if black_hole.link_type == $n {
                    $wh_query.get_mut(black_hole.wh).unwrap().changed = true;
                }
            }
        }
    };
}

pub fn sort_by_order(
    query: Query<(Entity, &Order)>,
    mut max_order: Local<usize>,
    mut queue: ResMut<Queue>,
) {
    *max_order = 1;
    queue.0.clear();
    queue.0.push(Vec::new());
    for (entity, order) in query.iter() {
        if order.0 > 0 {
            if order.0 > *max_order {
                queue.0.resize(order.0, Vec::new());
                *max_order = order.0;
            }
            queue.0[order.0 - 1].push(entity); //order 1 at index 0
        }
    }
}

pub fn process(
    queue: Res<Queue>,
    children_query: Query<&Children>,
    black_hole_query: Query<&BlackHole>,
    mut white_hole_query: Query<&mut WhiteHole>,
    mut op_query: Query<&mut Op>,
    mut bloom: Query<&mut BloomSettings, With<Camera>>,
    mut num_query: Query<&mut Num>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    material_ids: Query<&Handle<ColorMaterial>>,
    mut radius_query: Query<&mut Radius>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
    mut trans_query: Query<&mut Transform>,
    mut offset_query: Query<&mut Offset>,
    mut arr_query: Query<&mut Arr>,
    mut tonemapping: Query<&mut Tonemapping, With<Camera>>,
) {
    for id in queue.0.iter().flatten() {
        let children = children_query.get(*id).unwrap();
        match op_query.get(*id).unwrap().0 {
            -3 => { // float to radius
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if white_hole.changed {
                            let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                            if black_hole.link_type == -4 && white_hole.link_type == 1 {
                                white_hole.changed = false;
                                let input = num_query.get(black_hole.parent).unwrap().0;
                                let Mesh2dHandle(mesh_id) = mesh_ids.get(*id).unwrap();
                                radius_query.get_mut(*id).unwrap().0 = input;
                                let mesh = meshes.get_mut(mesh_id).unwrap();
                                *mesh = shape::Circle::new(input).into();
                                mark_changed!(-3, children, black_hole_query, white_hole_query);
                            }
                        }
                    }
                }
            },
            -2 => { // 4 floats to color
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if white_hole.changed {
                            let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                            if black_hole.link_type == -4 && (1..5).contains(&white_hole.link_type) {
                                white_hole.changed = false;
                                let input = num_query.get(black_hole.parent).unwrap().0;
                                let mat_id = material_ids.get(*id).unwrap();
                                match white_hole.link_type {
                                    1 => { mats.get_mut(mat_id).unwrap().color.set_h(input); },
                                    2 => { mats.get_mut(mat_id).unwrap().color.set_s(input); },
                                    3 => { mats.get_mut(mat_id).unwrap().color.set_l(input); },
                                    4 => { mats.get_mut(mat_id).unwrap().color.set_a(input); },
                                    _ => {},
                                }
                                mark_changed!(-2, children, black_hole_query, white_hole_query);
                            }
                        }
                    }
                }
            },
            -1 => { // 3 floats to position
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if white_hole.changed {
                            let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                            if black_hole.link_type == -4 && (1..4).contains(&white_hole.link_type) {
                                white_hole.changed = false;
                                let input = num_query.get(black_hole.parent).unwrap().0;
                                let mut t = trans_query.get_mut(*id).unwrap();
                                match white_hole.link_type {
                                    1 => t.translation.x = input,
                                    2 => t.translation.y = input,
                                    3 => t.translation.z = input,
                                    _ => {},
                                }
                                // position has changed
                                mark_changed!(-1, children, black_hole_query, white_hole_query);
                            }
                        }
                    }
                }
            },
            0 => { // pass
                // input to num
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if white_hole.changed {
                            let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                            if black_hole.link_type == -4 && white_hole.link_type == -4 {
                                white_hole.changed = false;
                                let input = num_query.get(black_hole.parent).unwrap().0;
                                num_query.get_mut(*id).unwrap().0 = input;
                                mark_changed!(-4, children, black_hole_query, white_hole_query);
                            }
                        }
                    }
                }
            },
            1 => { // bloom control
                let mut bloom_settings = bloom.single_mut();
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if !white_hole.changed { continue; }
                        let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                        if black_hole.link_type == -4 && (1..8).contains(&white_hole.link_type) {
                            white_hole.changed = false;
                            let input = num_query.get(black_hole.parent).unwrap().0 / 100.;
                            match white_hole.link_type {
                                1 => bloom_settings.intensity = input,
                                2 => bloom_settings.low_frequency_boost = input,
                                3 => bloom_settings.low_frequency_boost_curvature = input,
                                4 => bloom_settings.high_pass_frequency = input,
                                5 => bloom_settings.composite_mode = if input > 0. {
                                BloomCompositeMode::Additive } else { BloomCompositeMode::EnergyConserving },
                                6 => bloom_settings.prefilter_settings.threshold = input,
                                7 => bloom_settings.prefilter_settings.threshold_softness = input,
                                _ => {},
                            }
                        }
                    }
                }
            },
            2 => { // tone mapping
                let mut tm = tonemapping.single_mut();
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if !white_hole.changed { continue; }
                        let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                        if black_hole.link_type == -4 && white_hole.link_type == 1 {
                            white_hole.changed = false;
                            let input = num_query.get(black_hole.parent).unwrap().0;
                            match input as usize {
                                0 => *tm = Tonemapping::None,
                                1 => *tm = Tonemapping::Reinhard,
                                2 => *tm = Tonemapping::ReinhardLuminance,
                                3 => *tm = Tonemapping::AcesFitted,
                                4 => *tm = Tonemapping::AgX,
                                5 => *tm = Tonemapping::SomewhatBoringDisplayTransform,
                                6 => *tm = Tonemapping::TonyMcMapface,
                                7 => *tm = Tonemapping::BlenderFilmic,
                                _ => {},
                            }
                        }
                    }
                }
            },
            3 => { // get
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if white_hole.changed {
                            let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                            if black_hole.link_type >= 0 && white_hole.link_type == -4 {
                                white_hole.changed = false;
                                let arr = &arr_query.get(black_hole.parent).unwrap().0;
                                if let Some(input) = arr.get(black_hole.link_type as usize) {
                                    num_query.get_mut(*id).unwrap().0 = *input;
                                    mark_changed!(-4, children, black_hole_query, white_hole_query);
                                }
                            }
                        }
                    }
                }
            },
            4 => { // trans/color/radius to outputs
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if white_hole.changed {
                            let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                            if white_hole.link_type == 1 && (-3..0).contains(&black_hole.link_type) {
                                white_hole.changed = false;
                                let arr = &mut arr_query.get_mut(*id).unwrap().0;
                                match black_hole.link_type {
                                    -1 => {
                                        let t = trans_query.get(black_hole.parent).unwrap().translation;
                                        *arr = t.to_array().into();
                                    },
                                    -2 => {
                                        let mat_id = material_ids.get(black_hole.parent).unwrap();
                                        let c = mats.get(mat_id).unwrap().color;
                                        *arr = c.as_hsla_f32().into();
                                    },
                                    -3 => {
                                        let r = radius_query.get(black_hole.parent).unwrap().0;
                                        *arr = [r].into();
                                    }
                                    _ => {},
                                }
                                // let all connections know about this change
                                for child in children.iter() {
                                    if let Ok(black_hole) = black_hole_query.get(*child) {
                                        white_hole_query.get_mut(black_hole.wh).unwrap().changed = true;
                                    }
                                }
                            }
                        }
                    }
                }
            },
            _ => {},
        }
        for child in children {
            if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                if !white_hole.changed { continue; }
                let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                match (black_hole.link_type, white_hole.link_type) {
                    (-1, -1) => { //trans
                        white_hole.changed = false;
                        let input = trans_query.get(black_hole.parent).unwrap().translation;
                        let mut t = trans_query.get_mut(*id).unwrap();
                        let offset = offset_query.get(*id).unwrap().trans;
                        t.translation.x = input.x + offset.x;
                        t.translation.y = input.y + offset.y;
                        t.translation.z = input.z + offset.z;
                        mark_changed!(-1, children, black_hole_query, white_hole_query);
                    },
                    (-2, -2) => { // color
                        white_hole.changed = false;
                        let mat_id = material_ids.get(black_hole.parent).unwrap();
                        let offset = offset_query.get(*id).unwrap().color;
                        let mat = mats.get(mat_id).unwrap();
                        let input = mat.color;
                        mats.get_mut(material_ids.get(*id).unwrap()).unwrap().color = input + offset;
                        mark_changed!(-2, children, black_hole_query, white_hole_query);
                    },
                    (-3, -3) => { // radius
                        white_hole.changed = false;
                        if let Ok(Mesh2dHandle(mesh_id)) = mesh_ids.get(*id) {
                            let offset = offset_query.get(*id).unwrap().radius;
                            let input = radius_query.get(black_hole.parent).unwrap().0;
                            radius_query.get_mut(*id).unwrap().0 = input + offset;
                            let mesh = meshes.get_mut(mesh_id).unwrap();
                            *mesh = shape::Circle::new(input + offset).into();
                        }
                        mark_changed!(-3, children, black_hole_query, white_hole_query);
                    },
                    (-4, -5) => { // number to op
                        white_hole.changed = false;
                        let input = num_query.get(black_hole.parent).unwrap().0;
                        op_query.get_mut(*id).unwrap().0 = input as i32;
                    }
                    (-1, -6) => { // position to trans offset
                        white_hole.changed = false;
                        let input = trans_query.get(black_hole.parent).unwrap().translation;
                        offset_query.get_mut(*id).unwrap().trans = input;
                    },
                    (-2, -7) => { // color to color offset
                        white_hole.changed = false;
                        let mat_id = material_ids.get(black_hole.parent).unwrap();
                        let mat = mats.get(mat_id).unwrap();
                        let input = mat.color;
                        offset_query.get_mut(*id).unwrap().color = input;
                    },
                    (-3, -8) => { // radius to radius offset
                        white_hole.changed = false;
                        let input = radius_query.get(black_hole.parent).unwrap().0;
                        offset_query.get_mut(*id).unwrap().radius = input;
                    }
                    _ => {},
                }
            }
        }
    }
}


