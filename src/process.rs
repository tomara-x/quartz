use bevy::{
    ecs::system::SystemParam,
    sprite::Mesh2dHandle,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
        },
    prelude::*};

use fundsp::hacker32::*;

use crate::components::*;

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

#[derive(SystemParam)]
pub struct Access<'w, 's> {
    op_query: Query<'w, 's, &'static mut Op>,
    bloom: Query<'w, 's, & 'static mut BloomSettings, With<Camera>>,
    num_query: Query<'w, 's, &'static mut crate::components::Num>,
    mats: ResMut<'w, Assets<ColorMaterial>>,
    material_ids: Query<'w, 's, &'static Handle<ColorMaterial>>,
    radius_query: Query<'w, 's, &'static mut Radius>,
    meshes: ResMut<'w, Assets<Mesh>>,
    mesh_ids: Query<'w, 's, &'static Mesh2dHandle>,
    trans_query: Query<'w, 's, &'static mut Transform>,
    arr_query: Query<'w, 's, &'static mut Arr>,
    tonemapping: Query<'w, 's, &'static mut Tonemapping, With<Camera>>,
    op_changed_query: Query<'w, 's, &'static mut OpChanged>,
    net_query: Query<'w, 's, &'static mut Network>,
    net_ins_query: Query<'w, 's, &'static mut NetIns>,
}

pub fn process(
    queue: Res<Queue>,
    children_query: Query<&Children>,
    mut white_hole_query: Query<&mut WhiteHole>,
    mut access: Access,
    mut slot: ResMut<Slot>,
    mut had_l: Local<bool>,
    mut had_r: Local<bool>,
    mut oscil: Local<(u8, bool)>,
) {
    'entity: for id in queue.0.iter().flatten() {
        let children = children_query.get(*id).unwrap();
        let op_changed = &mut access.op_changed_query.get_mut(*id).unwrap().0;
        match access.op_query.get(*id).unwrap().0 {
            -10 => { // pass
                for child in children {
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        if white_hole.link_types == (-4, 1) {
                            if access.num_query.get(white_hole.bh_parent).unwrap().0 == 0. {
                                continue 'entity;
                            }
                        }
                    }
                }
            },
            -9 => { // sum
                let mut out = 0.;
                for child in children {
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        if white_hole.link_types.0 == -4 {
                            out += access.num_query.get(white_hole.bh_parent).unwrap().0;
                        }
                    }
                }
                access.num_query.get_mut(*id).unwrap().0 = out;
            },
            -8 => { // tonemapping
                let mut tm = access.tonemapping.single_mut();
                for child in children {
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        if white_hole.link_types == (-4, 1) {
                            let input = access.num_query.get(white_hole.bh_parent).unwrap().0;
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
            -7 => { // bloom
                let mut bloom_settings = access.bloom.single_mut();
                for child in children {
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        let input = access.num_query.get(white_hole.bh_parent).unwrap().0 / 100.;
                        match white_hole.link_types {
                            (-4, 1) => bloom_settings.intensity = input,
                            (-4, 2) => bloom_settings.low_frequency_boost = input,
                            (-4, 3) => bloom_settings.low_frequency_boost_curvature = input,
                            (-4, 4) => bloom_settings.high_pass_frequency = input,
                            (-4, 5) => bloom_settings.composite_mode = if input > 0. {
                            BloomCompositeMode::Additive } else { BloomCompositeMode::EnergyConserving },
                            (-4, 6) => bloom_settings.prefilter_settings.threshold = input,
                            (-4, 7) => bloom_settings.prefilter_settings.threshold_softness = input,
                            _ => {},
                        }
                    }
                }
            },
            -6 => { // set
                for child in children {
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        if white_hole.link_types.0 == -4 {
                            let index = Ord::max(white_hole.link_types.1, 0) as usize;
                            let arr = &mut access.arr_query.get_mut(*id).unwrap().0;
                            if arr.len() <= index { arr.resize(index + 1, 0.); }
                            arr[index] = access.num_query.get(white_hole.bh_parent).unwrap().0;
                        }
                    }
                }
            },
            -5 => { // get
                for child in children {
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        if white_hole.link_types.1 == -4 {
                           let arr = &access.arr_query.get(white_hole.bh_parent).unwrap().0;
                           // TODO(amy): with negative indexing get in reverse
                           if let Some(input) = arr.get(Ord::max(white_hole.link_types.0, 0) as usize) {
                               access.num_query.get_mut(*id).unwrap().0 = *input;
                           }
                        }
                    }
                }
            },
            -4 => { // separate outputs from trans/color/radius
                for child in children {
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        match white_hole.link_types {
                            (-1, 1) => {
                                let t = access.trans_query.get(white_hole.bh_parent).unwrap().translation;
                                let arr = &mut access.arr_query.get_mut(*id).unwrap().0;
                                *arr = t.to_array().into();
                            },
                            (-2, 1) => {
                                let mat_id = access.material_ids.get(white_hole.bh_parent).unwrap();
                                let c = access.mats.get(mat_id).unwrap().color;
                                let arr = &mut access.arr_query.get_mut(*id).unwrap().0;
                                *arr = c.as_hsla_f32().into();
                            },
                            (-3, 1) => {
                                let r = access.radius_query.get(white_hole.bh_parent).unwrap().0;
                                let arr = &mut access.arr_query.get_mut(*id).unwrap().0;
                                *arr = [r].into();
                            }
                            _ => {},
                        }
                    }
                }
            },
            -3 => { // float to radius
                for child in children {
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        if white_hole.link_types == (-4, 1) {
                            let input = access.num_query.get(white_hole.bh_parent).unwrap().0;
                            let Mesh2dHandle(mesh_id) = access.mesh_ids.get(*id).unwrap();
                            access.radius_query.get_mut(*id).unwrap().0 = input;
                            let mesh = access.meshes.get_mut(mesh_id).unwrap();
                            *mesh = bevy::prelude::shape::Circle::new(input).into();
                        }
                    }
                }
            },
            -2 => { // 4 floats to color
                for child in children {
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        if white_hole.link_types.0 == -4 {
                            let input = access.num_query.get(white_hole.bh_parent).unwrap().0;
                            let mat_id = access.material_ids.get(*id).unwrap();
                            match white_hole.link_types.1 {
                                1 => { access.mats.get_mut(mat_id).unwrap().color.set_h(input); },
                                2 => { access.mats.get_mut(mat_id).unwrap().color.set_s(input); },
                                3 => { access.mats.get_mut(mat_id).unwrap().color.set_l(input); },
                                4 => { access.mats.get_mut(mat_id).unwrap().color.set_a(input); },
                                _ => {},
                            }
                        }
                    }
                }
            },
            -1 => { // 3 floats to position
                for child in children {
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        if white_hole.link_types.0 == -4 {
                            let input = access.num_query.get(white_hole.bh_parent).unwrap().0;
                            let mut t = access.trans_query.get_mut(*id).unwrap();
                            match white_hole.link_types.1 {
                                1 => t.translation.x = input,
                                2 => t.translation.y = input,
                                3 => t.translation.z = input,
                                _ => {},
                            }
                        }
                    }
                }
            },
            0 => {},
            1 => { // Var
                let num = access.num_query.get(*id).unwrap().0;
                let var = &access.net_ins_query.get(*id).unwrap().0[0];
                var.set_value(num);
            },
            2 => { // Oscil
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if white_hole.link_types == (-4, 2) {
                            let input = access.num_query.get(white_hole.bh_parent).unwrap().0;
                            if (input as u8) != oscil.0 {
                                oscil.0 = input as u8;
                                oscil.1 = true; // new wave
                            }
                        }
                        if white_hole.link_types == (0, 1) && (white_hole.new_lt || oscil.1 || *op_changed) {
                            white_hole.new_lt = false;
                            oscil.1 = false;
                            let input = access.net_query.get(white_hole.bh_parent).unwrap().0.clone();
                            let net = &mut access.net_query.get_mut(*id).unwrap().0;
                            match oscil.0 {
                                0 => { *net = Net32::wrap(Box::new(input >> sine())); },
                                1 => { *net = Net32::wrap(Box::new(input >> saw())); },
                                2 => { *net = Net32::wrap(Box::new(input >> square())); },
                                _ => {},
                            }
                        }
                    }
                }
                *op_changed = false;
            },
            3 => { // Sum
                let mut changed = false;
                let mut inputs = Vec::new();
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if white_hole.link_types.0 == 0 {
                            inputs.push(&access.net_query.get(white_hole.bh_parent).unwrap().0);
                        }
                        if white_hole.new_lt || *op_changed {
                            white_hole.new_lt = false;
                            changed = true;
                        }
                    }
                }
                *op_changed = false;
                if changed {
                    let mut graph = Net32::wrap(Box::new(dc(0.)));
                    for i in inputs {
                        graph = graph + i.clone();
                    }
                    let output = &mut access.net_query.get_mut(*id).unwrap().0;
                    *output = Net32::wrap(Box::new(graph));
                }
            },
            4 => { // Product
                let mut changed = false;
                let mut inputs = Vec::new();
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        // grab everything connected through a correct connection
                        if white_hole.link_types.0 == 0 {
                            inputs.push(&access.net_query.get(white_hole.bh_parent).unwrap().0);
                        }
                        // something is new so we'll update our output
                        if white_hole.new_lt || *op_changed {
                            white_hole.new_lt = false;
                            changed = true;
                        }
                    }
                }
                *op_changed = false;
                if changed {
                    let mut graph = Net32::wrap(Box::new(dc(1.)));
                    for i in inputs {
                        graph = graph * i.clone();
                    }
                    let output = &mut access.net_query.get_mut(*id).unwrap().0;
                    *output = Net32::wrap(Box::new(graph));
                }
            },
            // you can simplify this by having 2 separate objects, out_l and out_r
            5 => { // Out
                let mut has_l = false;
                let mut has_r = false;
                for child in children {
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        if white_hole.link_types == (0, 1) { has_l = true; }
                        if white_hole.link_types == (0, 2) { has_r = true; }
                    }
                }
                if has_l || has_r { // we have inputs to 1 or 2
                    for child in children {
                        if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                            // if an input has a new net, we re-assign that slot
                            if white_hole.link_types == (0, 1) && (white_hole.new_lt || *op_changed) {
                                let l = &access.net_query.get(white_hole.bh_parent).unwrap().0;
                                slot.0.set(Fade::Smooth, 0.1, Box::new(l.clone()));
                                white_hole.new_lt = false;
                                *had_l = true;
                            }
                            if white_hole.link_types == (0, 2) && (white_hole.new_lt || *op_changed) {
                                let r = &access.net_query.get(white_hole.bh_parent).unwrap().0;
                                slot.1.set(Fade::Smooth, 0.1, Box::new(r.clone()));
                                white_hole.new_lt = false;
                                *had_r = true;
                            }
                        }
                    }
                    *op_changed = false;
                }
                // an input was here but it's now removed. we output silence
                if !has_l && *had_l {
                    slot.0.set(Fade::Smooth, 0.1, Box::new(dc(0.)));
                    *had_l = false;
                }
                if !has_r && *had_r {
                    slot.1.set(Fade::Smooth, 0.1, Box::new(dc(0.)));
                    *had_r = false;
                }
            },
            6 => { // Probe
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if white_hole.link_types == (0, 1) && (white_hole.new_lt || *op_changed) {
                            let mut input_net = access.net_query.get(white_hole.bh_parent).unwrap().0.clone();
                            input_net.set_sample_rate(120.);
                            let net = &mut access.net_query.get_mut(*id).unwrap().0;
                            *net = Net32::wrap(Box::new(input_net));
                            white_hole.new_lt = false;
                        }
                        if white_hole.link_types == (0, 1) {
                            let net = &mut access.net_query.get_mut(*id).unwrap().0;
                            let num = &mut access.num_query.get_mut(*id).unwrap().0;
                            *num = net.get_mono();
                        }
                    }
                }
                *op_changed = false;
            },
            _ => {},
        }
        for child in children {
            if let Ok(white_hole) = white_hole_query.get(*child) {
                if !white_hole.open { continue; }
                match white_hole.link_types {
                    (-1, -1) => { //trans
                        let input = access.trans_query.get(white_hole.bh_parent).unwrap().translation;
                        let mut t = access.trans_query.get_mut(*id).unwrap();
                        t.translation.x = input.x;
                        t.translation.y = input.y;
                        t.translation.z = input.z;
                    },
                    (-2, -2) => { // color
                        let mat_id = access.material_ids.get(white_hole.bh_parent).unwrap();
                        let mat = access.mats.get(mat_id).unwrap();
                        let input = mat.color;
                        access.mats.get_mut(
                            access.material_ids.get(*id).unwrap()
                        ).unwrap().color = input;
                    },
                    (-3, -3) => { // radius
                        if let Ok(Mesh2dHandle(mesh_id)) = access.mesh_ids.get(*id) {
                            let input = access.radius_query.get(white_hole.bh_parent).unwrap().0;
                            access.radius_query.get_mut(*id).unwrap().0 = input;
                            let mesh = access.meshes.get_mut(mesh_id).unwrap();
                            *mesh = bevy::prelude::shape::Circle::new(input).into();
                        }
                    },
                    (-4, -4) => { // num
                        let input = access.num_query.get(white_hole.bh_parent).unwrap().0;
                        access.num_query.get_mut(*id).unwrap().0 = input;
                    }
                    (-4, -5) => { // number to op
                        let input = access.num_query.get(white_hole.bh_parent).unwrap().0;
                        access.op_query.get_mut(*id).unwrap().0 = input as i32;
                    }
                    _ => {},
                }
            }
        }
    }
}
