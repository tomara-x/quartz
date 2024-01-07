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
    offset_query: Query<'w, 's, &'static mut Offset>,
    arr_query: Query<'w, 's, &'static mut Arr>,
    tonemapping: Query<'w, 's, &'static mut Tonemapping, With<Camera>>,
    op_changed_query: Query<'w, 's, &'static mut OpChanged>,
    network_query: Query<'w, 's, &'static mut Network>,
    netb_query: Query<'w, 's, &'static mut NetB>,
    net_nodes_query: Query<'w, 's, &'static mut NetNodes>,
}

pub fn process(
    queue: Res<Queue>,
    children_query: Query<&Children>,
    black_hole_query: Query<&BlackHole>,
    mut white_hole_query: Query<&mut WhiteHole>,
    mut access: Access,
    mut slot: ResMut<Slot>,
) {
    for id in queue.0.iter().flatten() {
        let children = children_query.get(*id).unwrap();
        match access.op_query.get(*id).unwrap().0 {
            -3 => { // float to radius
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if white_hole.changed {
                            let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                            if black_hole.link_type == -4 && white_hole.link_type == 1 {
                                white_hole.changed = false;
                                let input = access.num_query.get(black_hole.parent).unwrap().0;
                                let Mesh2dHandle(mesh_id) = access.mesh_ids.get(*id).unwrap();
                                access.radius_query.get_mut(*id).unwrap().0 = input;
                                let mesh = access.meshes.get_mut(mesh_id).unwrap();
                                *mesh = bevy::prelude::shape::Circle::new(input).into();
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
                                let input = access.num_query.get(black_hole.parent).unwrap().0;
                                let mat_id = access.material_ids.get(*id).unwrap();
                                match white_hole.link_type {
                                    1 => { access.mats.get_mut(mat_id).unwrap().color.set_h(input); },
                                    2 => { access.mats.get_mut(mat_id).unwrap().color.set_s(input); },
                                    3 => { access.mats.get_mut(mat_id).unwrap().color.set_l(input); },
                                    4 => { access.mats.get_mut(mat_id).unwrap().color.set_a(input); },
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
                                let input = access.num_query.get(black_hole.parent).unwrap().0;
                                let mut t = access.trans_query.get_mut(*id).unwrap();
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
                                let input = access.num_query.get(black_hole.parent).unwrap().0;
                                access.num_query.get_mut(*id).unwrap().0 = input;
                                mark_changed!(-4, children, black_hole_query, white_hole_query);
                            }
                        }
                    }
                }
            },
            1 => { // bloom control
                let mut bloom_settings = access.bloom.single_mut();
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if !white_hole.changed { continue; }
                        let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                        if black_hole.link_type == -4 && (1..8).contains(&white_hole.link_type) {
                            white_hole.changed = false;
                            let input = access.num_query.get(black_hole.parent).unwrap().0 / 100.;
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
                let mut tm = access.tonemapping.single_mut();
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if !white_hole.changed { continue; }
                        let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                        if black_hole.link_type == -4 && white_hole.link_type == 1 {
                            white_hole.changed = false;
                            let input = access.num_query.get(black_hole.parent).unwrap().0;
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
                                let arr = &access.arr_query.get(black_hole.parent).unwrap().0;
                                if let Some(input) = arr.get(black_hole.link_type as usize) {
                                    access.num_query.get_mut(*id).unwrap().0 = *input;
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
                                let arr = &mut access.arr_query.get_mut(*id).unwrap().0;
                                match black_hole.link_type {
                                    -1 => {
                                        let t = access.trans_query.get(black_hole.parent).unwrap().translation;
                                        *arr = t.to_array().into();
                                    },
                                    -2 => {
                                        let mat_id = access.material_ids.get(black_hole.parent).unwrap();
                                        let c = access.mats.get(mat_id).unwrap().color;
                                        *arr = c.as_hsla_f32().into();
                                    },
                                    -3 => {
                                        let r = access.radius_query.get(black_hole.parent).unwrap().0;
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
            5 => { //out
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                        if white_hole.link_type == 1 && black_hole.link_type == 0 {
                            if access.op_changed_query.get(black_hole.parent).unwrap().0 {
                                access.op_changed_query.get_mut(black_hole.parent).unwrap().0 = false;
                                let backend = &access.netb_query.get(black_hole.parent).unwrap().0;
                                slot.0.set(Fade::Smooth, 0.1, Box::new(backend.clone()));
                            }
                        }
                    }
                }
            },
            6 => { //oscil
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        if white_hole.changed {
                            let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                            if black_hole.link_type == -4 {
                                match white_hole.link_type {
                                    1 => {
                                        white_hole.changed = false;
                                        let input = access.num_query.get(black_hole.parent).unwrap().0;
                                        let dc_id = access.net_nodes_query.get(*id).unwrap().0[0];
                                        let net = &mut access.network_query.get_mut(*id).unwrap().0;
                                        net.replace(dc_id, Box::new(dc(input)));
                                        net.commit();
                                        access.op_changed_query.get_mut(*id).unwrap().0 = true;
                                    },
                                    _ => {},
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
                        let input = access.trans_query.get(black_hole.parent).unwrap().translation;
                        let mut t = access.trans_query.get_mut(*id).unwrap();
                        let offset = access.offset_query.get(*id).unwrap().trans;
                        t.translation.x = input.x + offset.x;
                        t.translation.y = input.y + offset.y;
                        t.translation.z = input.z + offset.z;
                        mark_changed!(-1, children, black_hole_query, white_hole_query);
                    },
                    (-2, -2) => { // color
                        white_hole.changed = false;
                        let mat_id = access.material_ids.get(black_hole.parent).unwrap();
                        let offset = access.offset_query.get(*id).unwrap().color;
                        let mat = access.mats.get(mat_id).unwrap();
                        let input = mat.color;
                        access.mats.get_mut(
                            access.material_ids.get(*id).unwrap()
                        ).unwrap().color = input + offset;
                        mark_changed!(-2, children, black_hole_query, white_hole_query);
                    },
                    (-3, -3) => { // radius
                        white_hole.changed = false;
                        if let Ok(Mesh2dHandle(mesh_id)) = access.mesh_ids.get(*id) {
                            let offset = access.offset_query.get(*id).unwrap().radius;
                            let input = access.radius_query.get(black_hole.parent).unwrap().0;
                            access.radius_query.get_mut(*id).unwrap().0 = input + offset;
                            let mesh = access.meshes.get_mut(mesh_id).unwrap();
                            *mesh = bevy::prelude::shape::Circle::new(input + offset).into();
                        }
                        mark_changed!(-3, children, black_hole_query, white_hole_query);
                    },
                    (-4, -5) => { // number to op
                        white_hole.changed = false;
                        let input = access.num_query.get(black_hole.parent).unwrap().0;
                        access.op_query.get_mut(*id).unwrap().0 = input as i32;
                    }
                    (-1, -6) => { // position to trans offset
                        white_hole.changed = false;
                        let input = access.trans_query.get(black_hole.parent).unwrap().translation;
                        access.offset_query.get_mut(*id).unwrap().trans = input;
                    },
                    (-2, -7) => { // color to color offset
                        white_hole.changed = false;
                        let mat_id = access.material_ids.get(black_hole.parent).unwrap();
                        let mat = access.mats.get(mat_id).unwrap();
                        let input = mat.color;
                        access.offset_query.get_mut(*id).unwrap().color = input;
                    },
                    (-3, -8) => { // radius to radius offset
                        white_hole.changed = false;
                        let input = access.radius_query.get(black_hole.parent).unwrap().0;
                        access.offset_query.get_mut(*id).unwrap().radius = input;
                    }
                    _ => {},
                }
            }
        }
    }
}


