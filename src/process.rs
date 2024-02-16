use bevy::{
    ecs::system::SystemParam,
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
    order_query: Query<'w, 's, &'static mut Order>,
    op_query: Query<'w, 's, &'static mut Op>,
    bloom: Query<'w, 's, & 'static mut BloomSettings, With<Camera>>,
    num_query: Query<'w, 's, &'static mut crate::components::Num>,
    radius_query: Query<'w, 's, &'static mut Radius>,
    trans_query: Query<'w, 's, &'static mut Transform>,
    arr_query: Query<'w, 's, &'static mut Arr>,
    tonemapping: Query<'w, 's, &'static mut Tonemapping, With<Camera>>,
    net_query: Query<'w, 's, &'static mut Network>,
    net_ins_query: Query<'w, 's, &'static mut NetIns>,
    col_query: Query<'w, 's, &'static mut Col>,
    order_change: EventWriter<'w, OrderChange>,
    vertices_query: Query<'w, 's, &'static mut Vertices>,
    net_changed_query: Query<'w, 's, &'static mut NetChanged>,
    //gained_wh_query: Query<'w, 's, &'static mut GainedWH>,
    lost_wh_query: Query<'w, 's, &'static mut LostWH>,
}

pub fn process(
    queue: Res<Queue>,
    children_query: Query<Ref<Children>>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<&BlackHole>,
    mut access: Access,
    mut slot: ResMut<Slot>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut char_input_events: EventReader<ReceivedCharacter>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
) {
    'entity: for id in queue.0.iter().flatten() {
        if let Ok(children) = &children_query.get(*id) {
            match access.op_query.get(*id).unwrap().0.as_str() {
                "mouse_x" => {
                    let (cam, cam_transform) = camera_query.single();
                    if let Some(cursor_pos) = windows.single().cursor_position() {
                        if let Some(point) = cam.viewport_to_world_2d(cam_transform, cursor_pos) {
                            access.num_query.get_mut(*id).unwrap().0 = point.x;
                        }
                    }
                },
                "mouse_y" => {
                    let (cam, cam_transform) = camera_query.single();
                    if let Some(cursor_pos) = windows.single().cursor_position() {
                        if let Some(point) = cam.viewport_to_world_2d(cam_transform, cursor_pos) {
                            access.num_query.get_mut(*id).unwrap().0 = point.y;
                        }
                    }
                },
                "lmb_pressed" => {
                    if mouse_button_input.pressed(MouseButton::Left) {
                        access.num_query.get_mut(*id).unwrap().0 = 1.;
                    } else {
                        access.num_query.get_mut(*id).unwrap().0 = 0.;
                    }
                },
                "mmb_pressed" => {
                    if mouse_button_input.pressed(MouseButton::Middle) {
                        access.num_query.get_mut(*id).unwrap().0 = 1.;
                    } else {
                        access.num_query.get_mut(*id).unwrap().0 = 0.;
                    }
                },
                "rmb_pressed" => {
                    if mouse_button_input.pressed(MouseButton::Right) {
                        access.num_query.get_mut(*id).unwrap().0 = 1.;
                    } else {
                        access.num_query.get_mut(*id).unwrap().0 = 0.;
                    }
                },
                "lmb_just_pressed" | "lmb_just_released" |
                "mmb_just_pressed" | "mmb_just_released" |
                "rmb_just_pressed" | "rmb_just_released" => {
                    let mut zero = false;
                    let mut one = false;
                    if access.num_query.get(*id).unwrap().0 != 0. {
                        access.num_query.get_mut(*id).unwrap().bypass_change_detection().0 = 0.;
                        zero = true;
                    }
                    match access.op_query.get(*id).unwrap().0.as_str() {
                        "lmb_just_pressed" => {
                            if mouse_button_input.just_pressed(MouseButton::Left) { one = true; }
                        }
                        "lmb_just_released" => {
                            if mouse_button_input.just_released(MouseButton::Left) { one = true; }
                        }
                        "mmb_just_pressed" => {
                            if mouse_button_input.just_pressed(MouseButton::Middle) { one = true; }
                        }
                        "mmb_just_released" => {
                            if mouse_button_input.just_released(MouseButton::Middle) { one = true; }
                        }
                        "rmb_just_pressed" => {
                            if mouse_button_input.just_pressed(MouseButton::Right) { one = true; }
                        }
                        "rmb_just_released" => {
                            if mouse_button_input.just_released(MouseButton::Right) { one = true; }
                        }
                        _ => {}
                    }
                    if one {
                        access.num_query.get_mut(*id).unwrap().bypass_change_detection().0 = 1.;
                    }
                    if zero || one {
                        for child in children {
                            if let Ok(bh) = black_hole_query.get(*child) {
                                if let Ok(wh) = white_hole_query.get_mut(bh.wh) {
                                    if wh.link_types.0 == -1 {
                                        white_hole_query.get_mut(bh.wh).unwrap().open = true;
                                    }
                                }
                            }
                        }
                    }
                },
                "butt" => {
                    if mouse_button_input.just_pressed(MouseButton::Left) {
                        let t = access.trans_query.get(*id).unwrap().translation.xy();
                        let r = access.radius_query.get(*id).unwrap().0;
                        if cursor.i.distance_squared(t) < r*r {
                            access.num_query.get_mut(*id).unwrap().0 = 1.;
                        }
                    }
                    if mouse_button_input.just_released(MouseButton::Left) {
                        access.num_query.get_mut(*id).unwrap().0 = 0.;
                    }
                },
                "toggle" => {
                    if mouse_button_input.just_pressed(MouseButton::Left) {
                        let t = access.trans_query.get(*id).unwrap().translation.xy();
                        let r = access.radius_query.get(*id).unwrap().0;
                        if cursor.i.distance_squared(t) < r*r {
                            let n = access.num_query.get(*id).unwrap().0;
                            access.num_query.get_mut(*id).unwrap().0 = 1. - n;
                        }
                    }
                },
                "tbutt" => {
                    let mut changed = false;
                    if mouse_button_input.just_pressed(MouseButton::Left) {
                        let t = access.trans_query.get(*id).unwrap().translation.xy();
                        let r = access.radius_query.get(*id).unwrap().0;
                        if cursor.i.distance_squared(t) < r*r {
                            access.num_query.get_mut(*id).unwrap().bypass_change_detection().0 = 1.;
                            changed = true;
                        }
                    } else if access.num_query.get(*id).unwrap().0 != 0. {
                        access.num_query.get_mut(*id).unwrap().bypass_change_detection().0 = 0.;
                        changed = true;
                    }
                    if changed {
                        for child in children {
                            if let Ok(bh) = black_hole_query.get(*child) {
                                if let Ok(wh) = white_hole_query.get_mut(bh.wh) {
                                    if wh.link_types.0 == -1 {
                                        white_hole_query.get_mut(bh.wh).unwrap().open = true;
                                    }
                                }
                            }
                        }
                    }
                },
                "key" => {
                    for event in char_input_events.read() {
                        access.num_query.get_mut(*id).unwrap().0 = (event.char as i32) as f32;
                    }
                },
                "semi_ratio" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.open && wh.link_types == (-1, 1) {
                                let input = access.num_query.get(wh.bh_parent).unwrap().0;
                                access.num_query.get_mut(*id).unwrap().0 = semitone_ratio(input);
                            }
                        }
                    }
                },
                "pass" => {
                    for child in children {
                        if let Ok(white_hole) = white_hole_query.get(*child) {
                            if white_hole.link_types == (-1, 1) {
                                if access.num_query.get(white_hole.bh_parent).unwrap().0 == 0. {
                                    continue 'entity;
                                }
                            }
                        }
                    }
                },
                "cold" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-1, 1) {
                                let n = access.num_query.get(wh.bh_parent).unwrap().0;
                                access.num_query.get_mut(*id).unwrap().bypass_change_detection().0 = n;
                            }
                        }
                    }
                },
                "sum" => {
                    let mut out = 0.;
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-1, 1) {
                                out += access.num_query.get(wh.bh_parent).unwrap().0;
                            }
                        }
                    }
                    access.num_query.get_mut(*id).unwrap().0 = out;
                },
                "product" => {
                    let mut out = 1.;
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-1, 1) {
                                out *= access.num_query.get(wh.bh_parent).unwrap().0;
                            }
                        }
                    }
                    access.num_query.get_mut(*id).unwrap().0 = out;
                },
                "tonemapping" => {
                    let mut tm = access.tonemapping.single_mut();
                    for child in children {
                        if let Ok(white_hole) = white_hole_query.get(*child) {
                            if white_hole.link_types == (-1, 1) {
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
                "bloom" => {
                    let mut bloom_settings = access.bloom.single_mut();
                    for child in children {
                        if let Ok(white_hole) = white_hole_query.get(*child) {
                            let input = access.num_query.get(white_hole.bh_parent).unwrap().0 / 100.;
                            match white_hole.link_types {
                                (-1, 1) => bloom_settings.intensity = input,
                                (-1, 2) => bloom_settings.low_frequency_boost = input,
                                (-1, 3) => bloom_settings.low_frequency_boost_curvature = input,
                                (-1, 4) => bloom_settings.high_pass_frequency = input,
                                (-1, 5) => bloom_settings.composite_mode = if input > 0. {
                                BloomCompositeMode::Additive } else { BloomCompositeMode::EnergyConserving },
                                (-1, 6) => bloom_settings.prefilter_settings.threshold = input,
                                (-1, 7) => bloom_settings.prefilter_settings.threshold_softness = input,
                                _ => {},
                            }
                        }
                    }
                },
                "set" => {
                    for child in children {
                        if let Ok(white_hole) = white_hole_query.get(*child) {
                            if white_hole.link_types.0 == -1 {
                                let index = Ord::max(white_hole.link_types.1, 0) as usize;
                                let arr = &mut access.arr_query.get_mut(*id).unwrap().0;
                                if arr.len() <= index { arr.resize(index + 1, 0.); }
                                arr[index] = access.num_query.get(white_hole.bh_parent).unwrap().0;
                            }
                        }
                    }
                },
                "get" => {
                    for child in children {
                        if let Ok(white_hole) = white_hole_query.get(*child) {
                            if white_hole.link_types.1 == -1 {
                               let arr = &access.arr_query.get(white_hole.bh_parent).unwrap().0;
                               // TODO(amy): with negative indexing get in reverse
                               if let Some(input) = arr.get(Ord::max(white_hole.link_types.0, 0) as usize) {
                                   access.num_query.get_mut(*id).unwrap().0 = *input;
                               }
                            }
                        }
                    }
                },
                "empty" => {},
                "var()" => {
                    // use is_changed
                    let num = access.num_query.get(*id).unwrap().0;
                    if let Some(var) = &access.net_ins_query.get(*id).unwrap().0.get(0) {
                        var.set_value(num);
                    }
                },
                "monitor()" | "timer()" => {
                    if let Some(var) = access.net_ins_query.get(*id).unwrap().0.get(0) {
                        access.num_query.get_mut(*id).unwrap().0 = var.value();
                    }
                },
                "sum()" | "product()" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut inputs = Vec::new();
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types.0 == 0 {
                                inputs.push(&access.net_query.get(wh.bh_parent).unwrap().0);
                            }
                            if wh.open {
                                wh.open = false;
                                changed = true;
                            }
                        }
                    }
                    if changed || lost || net_changed {
                        let mut graph = Net32::new(0,0);
                        let mut empty = true;
                        for i in inputs {
                            if empty {
                                graph = i.clone();
                                empty = false;
                            } else if graph.outputs() == i.outputs() {
                                if access.op_query.get(*id).unwrap().0 == "sum()" {
                                    graph = graph + i.clone();
                                } else {
                                    graph = graph * i.clone();
                                }
                            }
                        }
                        access.net_changed_query.get_mut(*id).unwrap().0 = true;
                        let output = &mut access.net_query.get_mut(*id).unwrap().0;
                        *output = Net32::wrap(Box::new(graph));
                    }
                },
                "stack()" => {
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut inputs = Vec::new();
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types.0 == 0 {
                                let index = wh.link_types.1 as usize;
                                if index >= inputs.len() {
                                    inputs.resize(index+1, None);
                                }
                                inputs[index] = Some(&access.net_query.get(wh.bh_parent).unwrap().0);
                            }
                            if wh.open {
                                wh.open = false;
                                changed = true;
                            }
                        }
                    }
                    if changed || lost {
                        let mut graph = Net32::new(0,0);
                        let mut empty = true;
                        for i in inputs {
                            if let Some(i) = i {
                                if empty {
                                    graph = i.clone();
                                    empty = false;
                                } else {
                                    graph = graph | i.clone();
                                }
                            }
                        }
                        access.net_changed_query.get_mut(*id).unwrap().0 = true;
                        access.net_query.get_mut(*id).unwrap().0 = Net32::wrap(Box::new(graph));
                    }
                },
                "pipe()" => {
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut inputs = Vec::new();
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types.0 == 0 {
                                let index = wh.link_types.1 as usize;
                                if index >= inputs.len() {
                                    inputs.resize(index+1, None);
                                }
                                inputs[index] = Some(&access.net_query.get(wh.bh_parent).unwrap().0);
                            }
                            if wh.open {
                                wh.open = false;
                                changed = true;
                            }
                        }
                    }
                    if changed || lost {
                        let mut graph = Net32::new(0,0);
                        let mut empty = true;
                        for i in inputs {
                            if let Some(i) = i {
                                if empty {
                                    graph = i.clone();
                                    empty = false;
                                }
                                else if graph.outputs() == i.inputs() {
                                    graph = graph >> i.clone();
                                }
                            }
                        }
                        access.net_changed_query.get_mut(*id).unwrap().0 = true;
                        access.net_query.get_mut(*id).unwrap().0 = Net32::wrap(Box::new(graph));
                    }
                },
                "out()" | "dac()" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    //let gained = access.gained_wh_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut net = None;
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types == (0, 1) {
                                net = Some(access.net_query.get(wh.bh_parent).unwrap().0.clone());
                            }
                            if wh.open {
                                wh.open = false;
                                changed = true;
                            }
                        }
                    }
                    if /*gained ||*/ lost || net_changed || changed {
                        if let Some(net) = net {
                            if net.outputs() == 1 && net.inputs() == 0 {
                                slot.0.set(Fade::Smooth, 0.1, Box::new(net.clone() | dc(0.)));
                            } else if net.outputs() == 2 && net.inputs() == 0 {
                                slot.0.set(Fade::Smooth, 0.1, Box::new(net.clone()));
                            }
                        } else {
                            slot.0.set(Fade::Smooth, 0.1, Box::new(dc(0.) | dc(0.)));
                        }
                    }
                },
                "outputs()" | "inputs()" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    //let gained = access.gained_wh_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut n: Option<usize> = None;
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types == (0, 1) {
                                if access.op_query.get(*id).unwrap().0 == "outputs()" {
                                    n = Some(access.net_query.get(wh.bh_parent).unwrap().0.outputs());
                                } else {
                                    n = Some(access.net_query.get(wh.bh_parent).unwrap().0.inputs());
                                }
                                if wh.open {
                                    wh.open = false;
                                    changed = true;
                                }
                                break;
                            }
                        }
                    }
                    if /*gained ||*/ lost || net_changed || changed {
                        if let Some(n) = n {
                            access.num_query.get_mut(*id).unwrap().0 = n as f32;
                        } else {
                            access.num_query.get_mut(*id).unwrap().0 = 0.;
                        }
                    }
                },
                // TODO(amy): turn this into a command instead
                "info()" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut input_entity = None;
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types == (0, 1) {
                                input_entity = Some(wh.bh_parent);
                                if wh.open {
                                    wh.open = false;
                                    changed = true;
                                }
                                break;
                            }
                        }
                    }
                    if net_changed || changed {
                        if let Some(e) = input_entity {
                            println!("> {}", access.op_query.get(e).unwrap().0);
                            println!("{}", access.net_query.get(e).unwrap().0.clone().display());
                        }
                    }
                },
                //"probe()" => {
                //    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                //    //let gained = access.gained_wh_query.get(*id).unwrap().0;
                //    let lost = access.lost_wh_query.get(*id).unwrap().0;
                //    let mut changed = false;
                //    let mut net = None;
                //    for child in children {
                //        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                //            if wh.link_types == (0, 1) {
                //                net = Some(access.net_query.get(wh.bh_parent).unwrap().0.clone());
                //            }
                //            if wh.open {
                //                wh.open = false;
                //                changed = true;
                //            }
                //        }
                //    }
                //    if /*gained ||*/ lost || net_changed || changed {
                //        if let Some(net) = net {
                //            if (net.outputs() == 1 || net.outputs() == 2) && net.inputs() == 0 {
                //                access.net_query.get_mut(*id).unwrap().0 = net;
                //            }
                //        } else {
                //            access.net_query.get_mut(*id).unwrap().0 = Net32::wrap(Box::new(dc(0.)));
                //        }
                //    }
                //    let net = &mut access.net_query.get_mut(*id).unwrap().0;
                //    // 44100/60 (samples in a visual frame) (we just use the last one)
                //    // this will never be accurate
                //    for _ in 0..734 { net.get_mono(); }
                //    access.num_query.get_mut(*id).unwrap().set_if_neq(Num(net.get_mono()));
                //},
                _ => {},
            }
            // open all white holes reading from this changed net
            if access.net_changed_query.get(*id).unwrap().0 {
                for child in children {
                    if let Ok(bh) = black_hole_query.get(*child) {
                        if let Ok(wh) = white_hole_query.get_mut(bh.wh) {
                            if wh.link_types.0 == 0 {
                                white_hole_query.get_mut(bh.wh).unwrap().open = true;
                            }
                        }
                    }
                }
            }
            access.net_changed_query.get_mut(*id).unwrap().0 = false;
            //access.gained_wh_query.get_mut(*id).unwrap().0 = false;
            access.lost_wh_query.get_mut(*id).unwrap().0 = false;
            for child in children {
                let mut lt_to_open = 0;
                if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                    if !white_hole.open { continue; }
                    let mut input = 0.;
                    match white_hole.link_types.0 {
                        // num
                        -1 => { input = access.num_query.get(white_hole.bh_parent).unwrap().0; },
                        // radius
                        -2 => { input = access.radius_query.get(white_hole.bh_parent).unwrap().0; },
                        // x
                        -3 => { input = access.trans_query.get(white_hole.bh_parent).unwrap().translation.x; },
                        // y
                        -4 => { input = access.trans_query.get(white_hole.bh_parent).unwrap().translation.y; },
                        // z
                        -5 => { input = access.trans_query.get(white_hole.bh_parent).unwrap().translation.z; },
                        // hue
                        -6 => { input = access.col_query.get(white_hole.bh_parent).unwrap().0.h(); },
                        // saturation
                        -7 => { input = access.col_query.get(white_hole.bh_parent).unwrap().0.s(); },
                        // lightness
                        -8 => { input = access.col_query.get(white_hole.bh_parent).unwrap().0.l(); },
                        // alpha
                        -9 => { input = access.col_query.get(white_hole.bh_parent).unwrap().0.a(); },
                        // order
                        -10 => { input = access.order_query.get(white_hole.bh_parent).unwrap().0 as f32; },
                        // vertices
                        -11 => { input = access.vertices_query.get(white_hole.bh_parent).unwrap().0 as f32; },
                        // rotation
                        -12 => { input = access.trans_query.get(white_hole.bh_parent)
                                               .unwrap().rotation.to_euler(EulerRot::XYZ).2; },
                        _ => {},
                    }
                    match white_hole.link_types.1 {
                        -1 => { access.num_query.get_mut(*id).unwrap().0 = input; },
                        -2 => { access.radius_query.get_mut(*id).unwrap().0 = input.max(0.); },
                        -3 => { access.trans_query.get_mut(*id).unwrap().translation.x = input; },
                        -4 => { access.trans_query.get_mut(*id).unwrap().translation.y = input; },
                        -5 => { access.trans_query.get_mut(*id).unwrap().translation.z = input; },
                        -6 => { access.col_query.get_mut(*id).unwrap().0.set_h(input); },
                        -7 => { access.col_query.get_mut(*id).unwrap().0.set_s(input); },
                        -8 => { access.col_query.get_mut(*id).unwrap().0.set_l(input); },
                        -9 => { access.col_query.get_mut(*id).unwrap().0.set_a(input); },
                        -10 => {
                            access.order_query.get_mut(*id).unwrap().0 = input as usize;
                            access.order_change.send_default();
                        },
                        -11 => { access.vertices_query.get_mut(*id).unwrap().0 = input.max(3.) as usize; },
                        -12 => {
                            let q = Quat::from_euler(EulerRot::XYZ, 0., 0., input);
                            access.trans_query.get_mut(*id).unwrap().rotation = q;
                        },
                        _ => {},
                    }
                    lt_to_open = white_hole.link_types.1;
                    white_hole.open = false;
                }
                // open all white holes reading whatever just changed
                if lt_to_open != 0 {
                    for child in children {
                        if let Ok(bh) = black_hole_query.get(*child) {
                            if let Ok(wh) = white_hole_query.get(bh.wh) {
                                if wh.link_types.0 == lt_to_open {
                                    white_hole_query.get_mut(bh.wh).unwrap().open = true;
                                }
                            }
                        }
                    }
                }
            }
        // no children (connections)
        } else {
            match access.op_query.get(*id).unwrap().0.as_str() {
                "out()" => {
                    slot.0.set(Fade::Smooth, 0.1, Box::new(dc(0.) | dc(0.)));
                },
                "probe()" => {
                    access.net_query.get_mut(*id).unwrap().0 = Net32::wrap(Box::new(dc(0.)));
                },
                "sum()" | "product()" | "pipe()" | "stack()" => {
                    access.net_query.get_mut(*id).unwrap().0 = Net32::new(0,0);
                },
                "inputs()" | "outputs()" => {
                    access.num_query.get_mut(*id).unwrap().0 = 0.;
                },
                _ => {},
            }
            // go back to oder 0 (doesn't get processed)
            access.order_query.get_mut(*id).unwrap().0 = 0;
            access.order_change.send_default();
        }
    }
}

pub fn update_net(
    mut query: Query<(&Op, &mut NetChanged, &mut Network, &mut NetIns), Changed<Op>>,
) {
    for (op, mut net_changed, mut n, mut inputs) in query.iter_mut() {
        net_changed.0 = true;
        inputs.0.clear();
        // "cat()" -> ["cat", "", ""],  "cat(mew, mrp)" -> ["cat", "mew, mrp", ""]
        let args: Vec<&str> = op.0.as_str().split(['(', ')']).collect();
        // parse the parameters (between parentheses)
        let mut p = Vec::new();
        if let Some(params) = args.get(1) {
            let params = params.split(',').collect::<Vec<&str>>();
            for s in params {
                if let Ok(n) = s.parse::<f32>() {
                    p.push(n);
                }
            }
        } else { continue; } // no parentheses
        match args[0] {
            "var" => {
                let input = shared(0.);
                n.0 = Net32::wrap(Box::new(var(&input)));
                inputs.0.push(input);
            },
            // TODO(amy): add the other modes
            "monitor" => {
                let s = shared(0.);
                n.0 = Net32::wrap(Box::new(monitor(&s, Meter::Sample)));
                inputs.0.push(s);
            },
            "timer" => {
                let s = shared(0.);
                n.0 = Net32::wrap(Box::new(timer(&s)));
                inputs.0.push(s);
            },
            "sink" => { n.0 = Net32::wrap(Box::new(sink())); },
            "pass" => { n.0 = Net32::wrap(Box::new(pass())); },
            "stack" => { n.0 = Net32::new(0,0); },
            "pipe" => { n.0 = Net32::new(0,0); },

            "panner" => { n.0 = Net32::wrap(Box::new(panner())); },

            "sine" => { n.0 = Net32::wrap(Box::new(sine())); },
            "saw" => { n.0 = Net32::wrap(Box::new(saw())); },
            "square" => { n.0 = Net32::wrap(Box::new(square())); },
            "triangle" => { n.0 = Net32::wrap(Box::new(triangle())); },
            "organ" => { n.0 = Net32::wrap(Box::new(organ())); },

            "pulse" => { n.0 = Net32::wrap(Box::new(pulse())); },
            "brown" => { n.0 = Net32::wrap(Box::new(brown())); },
            "pink" => { n.0 = Net32::wrap(Box::new(pink())); },
            "white" | "noise" => { n.0 = Net32::wrap(Box::new(white())); },

            "allpass" => { n.0 = Net32::wrap(Box::new(allpass())); },
            "allpole" => { n.0 = Net32::wrap(Box::new(allpole())); },
            "bandpass" => { n.0 = Net32::wrap(Box::new(bandpass())); },
            "bandrez" => { n.0 = Net32::wrap(Box::new(bandrez())); },
            "bell" => { n.0 = Net32::wrap(Box::new(bell())); },
            "butterpass" => { n.0 = Net32::wrap(Box::new(butterpass())); },
            "clip" => { n.0 = Net32::wrap(Box::new(clip())); },
            "dcblock" => { n.0 = Net32::wrap(Box::new(dcblock())); },
            "declick" => { n.0 = Net32::wrap(Box::new(declick())); },
            "dsf_saw" => { n.0 = Net32::wrap(Box::new(dsf_saw())); },
            "dsf_square" => { n.0 = Net32::wrap(Box::new(dsf_square())); },
            "hammond" => { n.0 = Net32::wrap(Box::new(hammond())); },
            "highpass" => { n.0 = Net32::wrap(Box::new(highpass())); },
            "highpole" => { n.0 = Net32::wrap(Box::new(highpole())); },
            "highshelf" => { n.0 = Net32::wrap(Box::new(highshelf())); },
            "lorenz" => { n.0 = Net32::wrap(Box::new(lorenz())); },
            "lowpass" => { n.0 = Net32::wrap(Box::new(lowpass())); },
            "lowpole" => { n.0 = Net32::wrap(Box::new(lowpole())); },
            "lowrez" => { n.0 = Net32::wrap(Box::new(lowrez())); },
            "lowshelf" => { n.0 = Net32::wrap(Box::new(lowshelf())); },
            "mls" => { n.0 = Net32::wrap(Box::new(mls())); },
            "moog" => { n.0 = Net32::wrap(Box::new(moog())); },
            "morph" => { n.0 = Net32::wrap(Box::new(morph())); },
            "notch" => { n.0 = Net32::wrap(Box::new(notch())); },
            "peak" => { n.0 = Net32::wrap(Box::new(peak())); },
            "pinkpass" => { n.0 = Net32::wrap(Box::new(pinkpass())); }
            "resonator" => { n.0 = Net32::wrap(Box::new(resonator())); }
            "rossler" => { n.0 = Net32::wrap(Box::new(rossler())); }
            "soft_saw" => { n.0 = Net32::wrap(Box::new(soft_saw())); }
            "tick" => { n.0 = Net32::wrap(Box::new(tick())); }
            "zero" => { n.0 = Net32::wrap(Box::new(zero())); }

            "pan" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(pan(*p)));
                }
            },
            "sine_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(sine_hz(*p)));
                }
            },
            "saw_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(saw_hz(*p)));
                }
            },
            "square_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(square_hz(*p)));
                }
            },
            "triangle_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(triangle_hz(*p)));
                }
            },
            "organ_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(organ_hz(*p)));
                }
            },

            "add" => {
                //if let Some(p) = p.get(0) {
                //    n.0 = Net32::wrap(Box::new(add(*p)));
                //}
                match p[..] {
                    [p0, p1, p2, p3, ..] => { n.0 = Net32::wrap(Box::new(add((p0, p1, p2, p3)))); },
                    [p0, p1, p2, ..] => { n.0 = Net32::wrap(Box::new(add((p0, p1, p2)))); },
                    [p0, p1, ..] => { n.0 = Net32::wrap(Box::new(add((p0, p1)))); },
                    [p0, ..] => { n.0 = Net32::wrap(Box::new(add(p0))); },
                    _ => { n.0 = Net32::wrap(Box::new(add(0.))); },
                }
            },
            "adsr" => {
                if let Some(p) = p.get(0..4) {
                    n.0 = Net32::wrap(Box::new(adsr_live(p[0], p[1], p[2], p[3])));
                }
            },
            "allpass_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(allpass_hz(p[0], p[1])));
                }
            },
            "allpass_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(allpass_q(*p)));
                }
            },
            "allpole_delay" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(allpole_delay(*p)));
                }
            },
            "bandpass_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(bandpass_hz(p[0], p[1])));
                }
            },
            "bandpass_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(bandpass_q(*p)));
                }
            },
            "bandrez_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(bandrez_hz(p[0], p[1])));
                }
            },
            "bandrez_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(bandrez_q(*p)));
                }
            },
            "bell_hz" => {
                if let Some(p) = p.get(0..3) {
                    n.0 = Net32::wrap(Box::new(bell_hz(p[0], p[1], p[2])));
                }
            },
            "bell_q" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(bell_q(p[0], p[1])));
                }
            },
            "biquad" => {
                if let Some(p) = p.get(0..5) {
                    n.0 = Net32::wrap(Box::new(biquad(p[0],p[1],p[2],p[3],p[4])));
                }
            },
            "butterpass_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(butterpass_hz(*p)));
                }
            },
            "chorus" => {
                if let Some(p) = p.get(0..4) {
                    n.0 = Net32::wrap(Box::new(chorus(p[0] as i64, p[1], p[2], p[3])));
                }
            },
            "clip_to" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(clip_to(p[0], p[1])));
                }
            },
            "constant" | "dc" => {
                match p[..] {
                    [p0, p1, p2, p3, ..] => { n.0 = Net32::wrap(Box::new(constant((p0, p1, p2, p3)))); },
                    [p0, p1, p2, ..] => { n.0 = Net32::wrap(Box::new(constant((p0, p1, p2)))); },
                    [p0, p1, ..] => { n.0 = Net32::wrap(Box::new(constant((p0, p1)))); },
                    [p0, ..] => { n.0 = Net32::wrap(Box::new(constant(p0))); },
                    _ => { n.0 = Net32::wrap(Box::new(constant(0.))); },
                }
                // try to figure out this size type stuff
                // let x: &Frame<f32,U4> = Frame::from_slice(p.as_slice());
                // n.0 = Net32::wrap(Box::new(constant(*x)));
            },
            "dcblock_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(dcblock_hz(*p)));
                }
            },
            "declick_s" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(declick_s(*p)));
                }
            },
            "delay" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(delay(*p)));
                }
            },
            "dsf_saw_r" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(dsf_saw_r(*p)));
                }
            },
            "dsf_square_r" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(dsf_square_r(*p)));
                }
            },
            "fir" => {
                match p[..] {
                    [p0,p1,p2,p3,p4,p5,p6,p7,p8,p9,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5,p6,p7,p8,p9)))); },
                    [p0,p1,p2,p3,p4,p5,p6,p7,p8,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5,p6,p7,p8)))); },
                    [p0,p1,p2,p3,p4,p5,p6,p7,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5,p6,p7)))); },
                    [p0,p1,p2,p3,p4,p5,p6,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5,p6)))); },
                    [p0,p1,p2,p3,p4,p5,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5)))); },
                    [p0,p1,p2,p3,p4,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4)))); },
                    [p0,p1,p2,p3,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3)))); },
                    [p0,p1,p2,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2)))); },
                    [p0,p1,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1)))); },
                    [p0,..] => { n.0 = Net32::wrap(Box::new(fir(p0))); },
                    _ => {},
                }
            },
            "fir3" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(An(fir3(*p))));
                }
            },
            "follow" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(follow((p[0], p[1]))));
                } else if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(follow(*p)));
                }
            },
            "hammond_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(hammond_hz(*p)));
                }
            },
            "highpass_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(highpass_hz(p[0], p[1])));
                }
            },
            "highpass_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(highpass_q(*p)));
                }
            },
            "highpole_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(highpole_hz(*p)));
                }
            },
            "highshelf_hz" => {
                if let Some(p) = p.get(0..3) {
                    n.0 = Net32::wrap(Box::new(highshelf_hz(p[0], p[1], p[2])));
                }
            },
            "highshelf_q" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(highshelf_q(p[0], p[1])));
                }
            },
            "hold" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(hold(*p)));
                }
            },
            "hold_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(hold_hz(p[0], p[1])));
                }
            },
            "join" => {
                if let Some(p) = p.get(0) {
                    match *p as usize {
                        2 => { n.0 = Net32::wrap(Box::new(join::<U2>())); },
                        4 => { n.0 = Net32::wrap(Box::new(join::<U4>())); },
                        8 => { n.0 = Net32::wrap(Box::new(join::<U8>())); },
                        _ => {},
                    }
                }
            },
            "split" => {
                if let Some(p) = p.get(0) {
                    match *p as usize {
                        2 => { n.0 = Net32::wrap(Box::new(split::<U2>())); },
                        4 => { n.0 = Net32::wrap(Box::new(split::<U4>())); },
                        8 => { n.0 = Net32::wrap(Box::new(split::<U8>())); },
                        _ => {},
                    }
                }
            },
            "reverse" => {
                if let Some(p) = p.get(0) {
                    match *p as usize {
                        2 => { n.0 = Net32::wrap(Box::new(reverse::<U2>())); },
                        3 => { n.0 = Net32::wrap(Box::new(reverse::<U3>())); },
                        4 => { n.0 = Net32::wrap(Box::new(reverse::<U4>())); },
                        _ => {},
                    }
                }
            },
            "limiter" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(limiter((p[0], p[1]))));
                } else if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(limiter(*p)));
                }
            },
            "limiter_stereo" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(limiter_stereo((p[0], p[1]))));
                } else if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(limiter_stereo(*p)));
                }
            },
            "lowpass_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(lowpass_hz(p[0], p[1])));
                }
            },
            "lowpass_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(lowpass_q(*p)));
                }
            },
            "lowpole_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(lowpole_hz(*p)));
                }
            },
            "lowrez_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(lowrez_hz(p[0], p[1])));
                }
            },
            "lowrez_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(lowrez_q(*p)));
                }
            },
            "lowshelf_hz" => {
                if let Some(p) = p.get(0..3) {
                    n.0 = Net32::wrap(Box::new(lowshelf_hz(p[0], p[1], p[2])));
                }
            },
            "lowshelf_q" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(lowshelf_q(p[0], p[1])));
                }
            },
            "mls_bits" => {
                if let Some(p) = p.get(0) {
                    let p = *p as i64;
                    if p >= 1 && p <= 31 {
                        n.0 = Net32::wrap(Box::new(mls_bits(p)));
                    }
                }
            },
            "moog_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(moog_hz(p[0], p[1])));
                }
            },
            "moog_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(moog_q(*p)));
                }
            },
            "morph_hz" => {
                if let Some(p) = p.get(0..3) {
                    n.0 = Net32::wrap(Box::new(morph_hz(p[0], p[1], p[2])));
                }
            },
            "mul" => {
                match p[..] {
                    [p0, p1, p2, p3, ..] => { n.0 = Net32::wrap(Box::new(mul((p0, p1, p2, p3)))); },
                    [p0, p1, p2, ..] => { n.0 = Net32::wrap(Box::new(mul((p0, p1, p2)))); },
                    [p0, p1, ..] => { n.0 = Net32::wrap(Box::new(mul((p0, p1)))); },
                    [p0, ..] => { n.0 = Net32::wrap(Box::new(mul(p0))); },
                    _ => { n.0 = Net32::wrap(Box::new(mul(1.))); },
                }
            },
            "notch_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(notch_hz(p[0], p[1])));
                }
            },
            "notch_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(notch_q(*p)));
                }
            },
            "peak_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(peak_hz(p[0], p[1])));
                }
            },
            "peak_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(peak_q(*p)));
                }
            },
            "pluck" => {
                if let Some(p) = p.get(0..3) {
                    n.0 = Net32::wrap(Box::new(pluck(p[0], p[1], p[2])));
                }
            },
            "resonator_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(resonator_hz(p[0], p[1])));
                }
            },
            "reverb_stereo" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(reverb_stereo(p[0].into(), p[1].into())));
                }
            },
            "soft_saw_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(soft_saw_hz(*p)));
                }
            },
            "tap" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(tap(p[0], p[1])));
                }
            },


            "ramp" => {
                n.0 = Net32::wrap(
                    Box::new(
                        lfo_in(|t, i: &Frame<f32, U1>| (t*i[0]).rem_euclid(1.))
                    )
                );
            },

            ">" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| if i[0]>i[1] {1.} else {0.}))); },
            "<" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| if i[0]<i[1] {1.} else {0.}))); },
            "==" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| if i[0]==i[1] {1.} else {0.}))); },
            "!=" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| if i[0]!=i[1] {1.} else {0.}))); },
            ">=" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| if i[0]>=i[1] {1.} else {0.}))); },
            "<=" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| if i[0]<=i[1] {1.} else {0.}))); },
            _ => { n.0 = Net32::wrap(Box::new(dc(0.))); },
        }
    }
}

// open the white holes reading any changed value
// it's gonna overlap with whatever `process` changed, but that's okay
// process needs to do things in order, but this is to catch any external change
// TODO(amy): bypass_change_detection in `process`
pub fn open_white_holes(
    num_query: Query<&Children, Changed<crate::components::Num>>,
    radius_query: Query<&Children, Changed<Radius>>,
    trans_query: Query<&Children, Changed<Transform>>,
    col_query: Query<&Children, Changed<Col>>,
    order_query: Query<&Children, Changed<Order>>,
    vertices_query: Query<&Children, Changed<Vertices>>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<&BlackHole>,
) {
    for children in num_query.iter() {
        for child in children {
            if let Ok(bh) = black_hole_query.get(*child) {
                if let Ok(wh) = white_hole_query.get(bh.wh) {
                    if wh.link_types.0 == -1 {
                        white_hole_query.get_mut(bh.wh).unwrap().open = true;
                    }
                }
            }
        }
    }
    for children in radius_query.iter() {
        for child in children {
            if let Ok(bh) = black_hole_query.get(*child) {
                if let Ok(wh) = white_hole_query.get(bh.wh) {
                    if wh.link_types.0 == -2 {
                        white_hole_query.get_mut(bh.wh).unwrap().open = true;
                    }
                }
            }
        }
    }
    for children in trans_query.iter() {
        for child in children {
            if let Ok(bh) = black_hole_query.get(*child) {
                if let Ok(wh) = white_hole_query.get(bh.wh) {
                    if wh.link_types.0 == -3
                    || wh.link_types.0 == -4
                    || wh.link_types.0 == -5
                    || wh.link_types.0 == -12 {
                        white_hole_query.get_mut(bh.wh).unwrap().open = true;
                    }
                }
            }
        }
    }
    for children in col_query.iter() {
        for child in children {
            if let Ok(bh) = black_hole_query.get(*child) {
                if let Ok(wh) = white_hole_query.get(bh.wh) {
                    if wh.link_types.0 == -6
                    || wh.link_types.0 == -7
                    || wh.link_types.0 == -8
                    || wh.link_types.0 == -9 {
                        white_hole_query.get_mut(bh.wh).unwrap().open = true;
                    }
                }
            }
        }
    }
    for children in order_query.iter() {
        for child in children {
            if let Ok(bh) = black_hole_query.get(*child) {
                if let Ok(wh) = white_hole_query.get(bh.wh) {
                    if wh.link_types.0 == -10 {
                        white_hole_query.get_mut(bh.wh).unwrap().open = true;
                    }
                }
            }
        }
    }
    for children in vertices_query.iter() {
        for child in children {
            if let Ok(bh) = black_hole_query.get(*child) {
                if let Ok(wh) = white_hole_query.get(bh.wh) {
                    if wh.link_types.0 == -11 {
                        white_hole_query.get_mut(bh.wh).unwrap().open = true;
                    }
                }
            }
        }
    }
}
