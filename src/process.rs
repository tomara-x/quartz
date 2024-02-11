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
    gained_wh_query: Query<'w, 's, &'static mut GainedWH>,
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
) {
    'entity: for id in queue.0.iter().flatten() {
        if let Ok(children) = &children_query.get(*id) {
            match access.op_query.get(*id).unwrap().0.as_str() {
                "butt" => {
                    if mouse_button_input.just_pressed(MouseButton::Left) {
                        let t = access.trans_query.get(*id).unwrap().translation.xy();
                        let r = access.radius_query.get(*id).unwrap().0;
                        if cursor.i.distance_squared(t) < r*r {
                            access.num_query.get_mut(*id).unwrap().bypass_change_detection().0 = 1.;
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
                    }
                    if mouse_button_input.just_released(MouseButton::Left) {
                        let t = access.trans_query.get(*id).unwrap().translation.xy();
                        let r = access.radius_query.get(*id).unwrap().0;
                        if cursor.f.distance_squared(t) < r*r {
                            access.num_query.get_mut(*id).unwrap().bypass_change_detection().0 = 0.;
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
                    }
                },
                "toggle" => {
                    if mouse_button_input.just_pressed(MouseButton::Left) {
                        let t = access.trans_query.get(*id).unwrap().translation.xy();
                        let r = access.radius_query.get(*id).unwrap().0;
                        if cursor.i.distance_squared(t) < r*r {
                            let n = access.num_query.get(*id).unwrap().0;
                            access.num_query.get_mut(*id).unwrap().bypass_change_detection().0 = 1. - n;
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
                    }
                },
                "tbutt" => {
                    if mouse_button_input.just_pressed(MouseButton::Left) {
                        let t = access.trans_query.get(*id).unwrap().translation.xy();
                        let r = access.radius_query.get(*id).unwrap().0;
                        if cursor.i.distance_squared(t) < r*r {
                            access.num_query.get_mut(*id).unwrap().bypass_change_detection().0 = 1.;
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
                    } else if access.num_query.get(*id).unwrap().0 != 0. {
                        access.num_query.get_mut(*id).unwrap().bypass_change_detection().0 = 0.;
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
                "sum" => {
                    let mut out = 0.;
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.open && wh.link_types == (-1, 1) {
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
                // TODO(mara): overhaul these 2 as well
                "sum()" => {
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
                    if changed {
                        access.net_changed_query.get_mut(*id).unwrap().0 = true;
                        let mut graph = Net32::wrap(Box::new(dc(0.)));
                        for i in inputs {
                            graph = graph + i.clone();
                        }
                        let output = &mut access.net_query.get_mut(*id).unwrap().0;
                        *output = Net32::wrap(Box::new(graph));
                    }
                },
                "product()" => {
                    let mut changed = false;
                    let mut inputs = Vec::new();
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            // grab everything connected through a correct connection
                            if wh.link_types.0 == 0 {
                                inputs.push(&access.net_query.get(wh.bh_parent).unwrap().0);
                            }
                            // something is new so we'll update our output
                            if wh.open {
                                wh.open = false;
                                changed = true;
                            }
                        }
                    }
                    if changed {
                        access.net_changed_query.get_mut(*id).unwrap().0 = true;
                        let mut graph = Net32::wrap(Box::new(dc(1.)));
                        for i in inputs {
                            graph = graph * i.clone();
                        }
                        let output = &mut access.net_query.get_mut(*id).unwrap().0;
                        *output = Net32::wrap(Box::new(graph));
                    }
                },
                // TODO(amy): try to break stack and pipe
                "stack()" => {
                    let mut changed = false;
                    let mut lhs = None;
                    let mut rhs = None;
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    let gained = access.gained_wh_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types == (0, 1) {
                                lhs = Some(access.net_query.get(wh.bh_parent).unwrap().0.clone());
                            } else if wh.link_types == (0, 2) {
                                rhs = Some(access.net_query.get(wh.bh_parent).unwrap().0.clone());
                            }
                            if wh.open {
                                wh.open = false;
                                changed = true;
                            }
                        }
                    }
                    if gained || lost || net_changed || changed {
                        if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                            access.net_changed_query.get_mut(*id).unwrap().0 = true;
                            access.net_query.get_mut(*id).unwrap().0 = Net32::stack_op(lhs, rhs);
                        } else {
                            access.net_query.get_mut(*id).unwrap().0 = Net32::new(0,0);
                        }
                    }
                },
                "pipe()" => {
                    let mut changed = false;
                    let mut lhs = None;
                    let mut rhs = None;
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    let gained = access.gained_wh_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types == (0, 1) {
                                lhs = Some(access.net_query.get(wh.bh_parent).unwrap().0.clone());
                            } else if wh.link_types == (0, 2) {
                                rhs = Some(access.net_query.get(wh.bh_parent).unwrap().0.clone());
                            }
                            if wh.open {
                                wh.open = false;
                                changed = true;
                            }
                        }
                    }
                    if gained || lost || net_changed || changed {
                        if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                            access.net_changed_query.get_mut(*id).unwrap().0 = true;
                            if lhs.outputs() == rhs.inputs() {
                                access.net_query.get_mut(*id).unwrap().0 = Net32::pipe_op(lhs, rhs);
                            }
                        } else {
                            access.net_query.get_mut(*id).unwrap().0 = Net32::new(0,0);
                        }
                    }
                },
                "out()" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    let gained = access.gained_wh_query.get(*id).unwrap().0;
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
                    if gained || lost || net_changed || changed {
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
                // TODO(mara): overhaul these 2
                "outputs()" => {
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types == (0, 1) && (wh.open) {
                                let net = &access.net_query.get(wh.bh_parent).unwrap().0;
                                access.num_query.get_mut(*id).unwrap().0 = net.outputs() as f32;
                                wh.open = false;
                                continue 'entity;
                            }
                        }
                    }
                },
                "probe()" => {
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types == (0, 1) && wh.open {
                                let mut input_net = access.net_query.get(wh.bh_parent).unwrap().0.clone();
                                input_net.set_sample_rate(120.);
                                let net = &mut access.net_query.get_mut(*id).unwrap().0;
                                *net = Net32::wrap(Box::new(input_net));
                                wh.open = false;
                            }
                            if wh.link_types == (0, 1) {
                                let net = &mut access.net_query.get_mut(*id).unwrap().0;
                                let num = &mut access.num_query.get_mut(*id).unwrap().0;
                                *num = net.get_mono();
                            }
                        }
                    }
                },
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
            access.gained_wh_query.get_mut(*id).unwrap().0 = false;
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
        match op.0.as_str() {
            "var()" => {
                let input = shared(0.);
                n.0 = Net32::wrap(Box::new(var(&input)));
                inputs.0.push(input);
            },
            "sink()" => { n.0 = Net32::wrap(Box::new(sink())); },
            "pass()" => { n.0 = Net32::wrap(Box::new(pass())); },
            "stack()" => { n.0 = Net32::new(0,0); },
            "pipe()" => { n.0 = Net32::new(0,0); },
            "sine()" => { n.0 = Net32::wrap(Box::new(sine())); },
            "saw()" => { n.0 = Net32::wrap(Box::new(saw())); },
            "square()" => { n.0 = Net32::wrap(Box::new(square())); },
            "organ()" => { n.0 = Net32::wrap(Box::new(organ())); },
            "panner()" => { n.0 = Net32::wrap(Box::new(panner())); },
            _ => { n.0 = Net32::wrap(Box::new(dc(0.))); },
        }
    }
}

// open the white holes reading any changed value
// it's gonna overlap with whatever `process` changed, but that's okay
// process needs to do things in order, but this is to catch any external change
// TODO(amy): skip change detection in `process`?
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
