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
    net_query: Query<'w, 's, &'static mut Network>,
    net_ins_query: Query<'w, 's, &'static mut NetIns>,
}

pub fn process(
    queue: Res<Queue>,
    children_query: Query<&Children>,
    mut black_hole_query: Query<&mut BlackHole>,
    mut white_hole_query: Query<&mut WhiteHole>,
    mut access: Access,
    mut slot: ResMut<Slot>,
    mut had_l: Local<bool>,
    mut had_r: Local<bool>,
) {
    for id in queue.0.iter().flatten() {
        let children = children_query.get(*id).unwrap();
        match access.op_query.get(*id).unwrap().0 {
            -7 => { // tonemapping
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
            -6 => { // bloom
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
            -5 => { // get
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
            -4 => { // separate outputs from trans/color/radius
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
                                        // color has 4 outputs, everything else is less
                                        // we update anything reading from outputs 0..4
                                        if (0..4).contains(&black_hole.link_type) {
                                            white_hole_query.get_mut(black_hole.wh).unwrap().changed = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
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
            0 => {},
            1 => { // Var
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        let op_changed = &mut access.op_changed_query.get_mut(*id).unwrap().0;
                        let black_hole = &mut black_hole_query.get_mut(white_hole.bh).unwrap();
                        if white_hole.link_type == 1 && black_hole.link_type == -4 {
                            if white_hole.changed {
                                white_hole.changed = false;
                                let input = access.num_query.get(black_hole.parent).unwrap().0;
                                let var = &access.net_ins_query.get(*id).unwrap().0[0];
                                var.set_value(input);
                            }
                            if white_hole.new_lt || black_hole.new_lt || *op_changed {
                                white_hole.new_lt = false; black_hole.new_lt = false; *op_changed = false;
                                mark_changed!(0, children, black_hole_query, white_hole_query);
                            }
                        }
                    }
                }
            },
            2 => { // Oscil
                for child in children {
                    if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                        let black_hole = &mut black_hole_query.get_mut(white_hole.bh).unwrap();
                        if white_hole.link_type == 1 && black_hole.link_type == 0 {
                            let op_changed = &mut access.op_changed_query.get_mut(*id).unwrap().0;
                            if white_hole.changed || white_hole.new_lt || black_hole.new_lt || *op_changed {
                                white_hole.changed = false;
                                white_hole.new_lt = false;
                                black_hole.new_lt = false;
                                *op_changed = false;
                                let var = access.net_query.get(black_hole.parent).unwrap().0.clone();
                                let out = &mut access.net_query.get_mut(*id).unwrap().0;
                                // TODO: use a second input to set wave shape
                                *out = Net32::wrap(Box::new(var >> sine()));
                                mark_changed!(0, children, black_hole_query, white_hole_query);
                            }
                        }
                    }
                }
            },
            // FIXME(amy): this and product are now broken
            3 => { // Sum
                let mut changed = false;
                let mut inputs = Vec::new();
                for child in children {
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                        if black_hole.link_type == 0 {
                            inputs.push(&access.net_query.get(black_hole.parent).unwrap().0);
                        }
                        let in_op_changed = &mut access.op_changed_query.get_mut(black_hole.parent).unwrap().0;
                        if *in_op_changed {
                            *in_op_changed = false;
                            changed = true;
                            access.op_changed_query.get_mut(*id).unwrap().0 = true;
                        }
                    }
                }
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
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                        // grab everything connected through a correct connection
                        if black_hole.link_type == 0 {
                            inputs.push(&access.net_query.get(black_hole.parent).unwrap().0);
                        }
                        // something is new so we'll update our output
                        let in_op_changed = &mut access.op_changed_query.get_mut(black_hole.parent).unwrap().0;
                        if *in_op_changed {
                            *in_op_changed = false;
                            changed = true;
                            // this entity's op has "changed"
                            // TODO(amy): figure out better names for this
                            // the op hasn't changed, our output netork has
                            access.op_changed_query.get_mut(*id).unwrap().0 = true;
                        }
                    }
                }
                if changed {
                    let mut graph = Net32::wrap(Box::new(dc(1.)));
                    for i in inputs {
                        graph = graph * i.clone();
                    }
                    let output = &mut access.net_query.get_mut(*id).unwrap().0;
                    *output = Net32::wrap(Box::new(graph));
                }
            },
            5 => { // Out
                let mut has_l = false;
                let mut has_r = false;
                for child in children {
                    if let Ok(white_hole) = white_hole_query.get(*child) {
                        let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                        if white_hole.link_type == 1 && black_hole.link_type == 0 { has_l = true; }
                        if white_hole.link_type == 2 && black_hole.link_type == 0 { has_r = true; }
                    }
                }
                if has_l || has_r { // we have inputs to 1 or 2
                    for child in children {
                        if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                            let black_hole = &mut black_hole_query.get_mut(white_hole.bh).unwrap();
                            // if an input has a new net, we re-assign that slot
                            if white_hole.link_type == 1 && black_hole.link_type == 0 &&
                                (white_hole.changed || white_hole.new_lt || black_hole.new_lt) {
                                let l = &access.net_query.get(black_hole.parent).unwrap().0;
                                slot.0.set(Fade::Smooth, 0.1, Box::new(l.clone()));
                                white_hole.changed = false;
                                white_hole.new_lt = false; black_hole.new_lt = false;
                                *had_l = true;
                            }
                            if white_hole.link_type == 2 && black_hole.link_type == 0 &&
                                (white_hole.changed || white_hole.new_lt || black_hole.new_lt) {
                                let r = &access.net_query.get(black_hole.parent).unwrap().0;
                                slot.1.set(Fade::Smooth, 0.1, Box::new(r.clone()));
                                white_hole.changed = false;
                                white_hole.new_lt = false; black_hole.new_lt = false;
                                *had_r = true;
                            }
                        }
                    }
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


