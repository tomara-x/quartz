use bevy::{
    ecs::system::SystemParam,
    winit::{WinitSettings, UpdateMode},
    utils::Duration,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
        },
    render::view::screenshot::ScreenshotManager,
    prelude::*
};

use fundsp::hacker32::*;

use crate::{
    components::*,
    nodes::*,
};

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
    targets_query: Query<'w, 's, &'static mut Targets>,
    screensot_manager: ResMut<'w, ScreenshotManager>,
    winit_settings: ResMut<'w, WinitSettings>,
}

pub fn process(
    queue: Res<Queue>,
    children_query: Query<Ref<Children>>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<&BlackHole>,
    mut access: Access,
    mut slot: ResMut<Slot>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut char_input_events: EventReader<ReceivedCharacter>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<(Entity, &Window)>,
) {
    for id in queue.0.iter().flatten() {
        if let Ok(children) = &children_query.get(*id) {
            match access.op_query.get(*id).unwrap().0.as_str() {
                "update_rate" => {
                    if access.num_query.get_mut(*id).unwrap().is_changed() {
                        let n = access.num_query.get(*id).unwrap().0;
                        access.winit_settings.focused_mode = UpdateMode::ReactiveLowPower {
                            wait: Duration::from_secs_f64((1.0 / n.max(0.01)).into()),
                        }
                    }
                }
                "screenshot" => {
                    if access.num_query.get(*id).unwrap().0 == 1. {
                        let win = windows.single().0;
                        let epoch = std::time::UNIX_EPOCH;
                        let now = std::time::SystemTime::now();
                        if let Ok(dur) = now.duration_since(epoch) {
                            let time = dur.as_millis();
                            let path = format!("screenshots/{:?}.png", time);
                            access.screensot_manager.save_screenshot_to_disk(win, path).unwrap();
                        }
                    }
                }
                "mouse_x" => {
                    let (cam, cam_transform) = camera_query.single();
                    if let Some(cursor_pos) = windows.single().1.cursor_position() {
                        if let Some(point) = cam.viewport_to_world_2d(cam_transform, cursor_pos) {
                            access.num_query.get_mut(*id).unwrap().0 = point.x;
                        }
                    }
                }
                "mouse_y" => {
                    let (cam, cam_transform) = camera_query.single();
                    if let Some(cursor_pos) = windows.single().1.cursor_position() {
                        if let Some(point) = cam.viewport_to_world_2d(cam_transform, cursor_pos) {
                            access.num_query.get_mut(*id).unwrap().0 = point.y;
                        }
                    }
                }
                "lmb_pressed" => {
                    if mouse_button_input.pressed(MouseButton::Left) {
                        access.num_query.get_mut(*id).unwrap().0 = 1.;
                    } else {
                        access.num_query.get_mut(*id).unwrap().0 = 0.;
                    }
                }
                "mmb_pressed" => {
                    if mouse_button_input.pressed(MouseButton::Middle) {
                        access.num_query.get_mut(*id).unwrap().0 = 1.;
                    } else {
                        access.num_query.get_mut(*id).unwrap().0 = 0.;
                    }
                }
                "rmb_pressed" => {
                    if mouse_button_input.pressed(MouseButton::Right) {
                        access.num_query.get_mut(*id).unwrap().0 = 1.;
                    } else {
                        access.num_query.get_mut(*id).unwrap().0 = 0.;
                    }
                }
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
                }
                "toggle" => {
                    if mouse_button_input.just_pressed(MouseButton::Left) {
                        let t = access.trans_query.get(*id).unwrap().translation.xy();
                        let r = access.radius_query.get(*id).unwrap().0;
                        if cursor.i.distance_squared(t) < r*r {
                            let n = access.num_query.get(*id).unwrap().0;
                            access.num_query.get_mut(*id).unwrap().0 = 1. - n;
                        }
                    }
                }
                // this would be useful if we only ever read from open wh
                "change" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-1, 1) {
                                let input = access.num_query.get(wh.bh_parent).unwrap().0;
                                access.num_query.get_mut(*id).unwrap().set_if_neq(Num(input));
                            }
                        }
                    }
                }
                // uses the array to store prevous num value
                "rise" | "fall" => {
                    if access.arr_query.get(*id).unwrap().0.len() != 1 {
                        access.arr_query.get_mut(*id).unwrap().0 = vec!(0.);
                    }
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-1, 1) {
                                let input = access.num_query.get(wh.bh_parent).unwrap().0;
                                let arr = &mut access.arr_query.get_mut(*id).unwrap().0;
                                if access.op_query.get(*id).unwrap().0 == "rise" {
                                    if input > arr[0] { access.num_query.get_mut(*id).unwrap().0 = 1.; }
                                } else {
                                    if input < arr[0] { access.num_query.get_mut(*id).unwrap().0 = 1.; }
                                }
                                if input == arr[0] {
                                    access.num_query.get_mut(*id).unwrap().0 = 0.
                                }
                                arr[0] = input;
                            }
                        }
                    }
                }
                "key" => {
                    for event in char_input_events.read() {
                        if let Some(c) = event.char.chars().nth(0) {
                            access.num_query.get_mut(*id).unwrap().0 = (c as i32) as f32;
                        }
                    }
                }
                "semi_ratio" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.open && wh.link_types == (-1, 1) {
                                let input = access.num_query.get(wh.bh_parent).unwrap().0;
                                access.num_query.get_mut(*id).unwrap().0 = semitone_ratio(input);
                            }
                        }
                    }
                }
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
                }
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
                }
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
                }
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
                }
                "count" => {
                    let mut trig = None;
                    let mut high = None;
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-1, 1) {
                                trig = Some(access.num_query.get(wh.bh_parent).unwrap().0);
                            }
                            if wh.link_types == (-1, 2) {
                                high = Some(access.num_query.get(wh.bh_parent).unwrap().0);
                            }
                        }
                    }
                    if let Some(trig) = trig {
                        let num = &mut access.num_query.get_mut(*id).unwrap().0;
                        *num += trig;
                        if let Some(high) = high {
                            if *num >= high { *num = 0.; }
                        }
                    }
                }
                "set" => {
                    let mut ndx = None;
                    let mut val = None;
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-1, 1) { ndx = Some(wh.bh_parent); }
                            if wh.link_types == (-1, 2) { val = Some(wh.bh_parent); }
                        }
                    }
                    if let (Some(ndx), Some(val)) = (ndx, val) {
                        if !access.num_query.get_mut(val).unwrap().is_changed() { continue; }
                        let ndx = access.num_query.get(ndx).unwrap().0.max(0.) as usize;
                        let val = access.num_query.get(val).unwrap().0;
                        if let Some(i) = access.arr_query.get_mut(*id).unwrap().0.get_mut(ndx) {
                            *i = val;
                        }
                    }
                }
                "get" => {
                    let mut arr = None;
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-13, 1) { arr = Some(wh.bh_parent); }
                            if wh.link_types == (-1, 2) {
                                let n = access.num_query.get_mut(wh.bh_parent).unwrap();
                                if n.is_changed() {
                                    if let Some(arr) = arr {
                                        if let Some(v) = access.arr_query.get(arr).unwrap().0.get(n.0 as usize) {
                                            access.num_query.get_mut(*id).unwrap().0 = *v;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "nums_to_arr" => {
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut inputs = Vec::new();
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types.0 == -1 {
                                let index = Ord::max(wh.link_types.1, 0) as usize;
                                if index >= inputs.len() {
                                    inputs.resize(index+1, None);
                                }
                                inputs[index] = Some(access.num_query.get(wh.bh_parent).unwrap().0);
                            }
                            if wh.open {
                                wh.open = false;
                                changed = true;
                            }
                        }
                    }
                    if changed || lost {
                        let arr = &mut access.arr_query.get_mut(*id).unwrap().0;
                        arr.clear();
                        for i in inputs {
                            if let Some(i) = i {
                                arr.push(i);
                            }
                        }
                    }
                }
                // distribute input array among targets' values
                "distro" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types.0 == -13 {
                                if access.arr_query.get_mut(wh.bh_parent).unwrap().is_changed() {
                                    let arr = &access.arr_query.get(wh.bh_parent).unwrap().0;
                                    let targets = &access.targets_query.get(*id).unwrap().0;
                                    let len = Ord::min(arr.len(), targets.len());
                                    for i in 0..len {
                                        if access.radius_query.get(targets[i]).is_err() { continue; }
                                        // input link type determines what property to write to in targets
                                        match wh.link_types.1 {
                                            -1 => { access.num_query.get_mut(targets[i]).unwrap().0 = arr[i]; }
                                            -2 => { access.radius_query.get_mut(targets[i]).unwrap().0 = arr[i].max(0.1); }
                                            -3 => { access.trans_query.get_mut(targets[i]).unwrap().translation.x = arr[i]; }
                                            -4 => { access.trans_query.get_mut(targets[i]).unwrap().translation.y = arr[i]; }
                                            -5 => { access.trans_query.get_mut(targets[i]).unwrap().translation.z = arr[i]; }
                                            -6 => { access.col_query.get_mut(targets[i]).unwrap().0.set_h(arr[i]); }
                                            -7 => { access.col_query.get_mut(targets[i]).unwrap().0.set_s(arr[i]); }
                                            -8 => { access.col_query.get_mut(targets[i]).unwrap().0.set_l(arr[i]); }
                                            -9 => { access.col_query.get_mut(targets[i]).unwrap().0.set_a(arr[i]); }
                                            -10 => {
                                                access.order_query.get_mut(targets[i]).unwrap().0 = arr[i] as usize;
                                                access.order_change.send_default();
                                            }
                                            -11 => {
                                                access.vertices_query.get_mut(targets[i]).unwrap().0 = arr[i].max(3.) as usize;
                                            }
                                            -12 => {
                                                let q = Quat::from_euler(EulerRot::XYZ, 0., 0., arr[i]);
                                                access.trans_query.get_mut(targets[i]).unwrap().rotation = q;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "apply" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (0, 1) && wh.open {
                                // TODO(amy): would be nice if we applied input on net change
                                let input_net = &access.net_query.get(wh.bh_parent).unwrap().0;
                                access.net_query.get_mut(*id).unwrap().0 = input_net.clone();
                            }
                            if wh.link_types == (-13, 2) && wh.open {
                                let input = access.arr_query.get(wh.bh_parent).unwrap().0.clone();
                                let output = &mut access.arr_query.get_mut(*id).unwrap().0;
                                let net = &mut access.net_query.get_mut(*id).unwrap().0;
                                if net.inputs() == input.len() {
                                    output.resize(net.outputs(), 0.);
                                    net.tick(input.as_slice(), output.as_mut_slice());
                                }
                            }
                        }
                    }
                }
                "render" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (0, 1) && wh.open {
                                let input_net = &access.net_query.get(wh.bh_parent).unwrap().0;
                                access.net_query.get_mut(*id).unwrap().0 = input_net.clone();
                            }
                            if wh.link_types == (-1, 2) {
                                // interesting thought, using the num as a gate
                                if access.num_query.get(*id).unwrap().0 == 1. {
                                    let len = access.num_query.get(wh.bh_parent).unwrap().0 / 44100.;
                                    let output = &mut access.arr_query.get_mut(*id).unwrap().0;
                                    let net = &mut access.net_query.get_mut(*id).unwrap().0;
                                    if net.inputs() == 0 && net.outputs() > 0 && len >= 0. {
                                        let wave = Wave32::render(44100., len.into(), net);
                                        *output = wave.channel(0).clone();
                                    }
                                }
                            }
                        }
                    }
                }
                "empty" => {}
                "var()" => {
                    // use is_changed
                    let num = access.num_query.get(*id).unwrap().0;
                    if let Some(var) = &access.net_ins_query.get(*id).unwrap().0.get(0) {
                        var.set_value(num);
                    }
                }
                "monitor()" | "timer()" => {
                    if let Some(var) = access.net_ins_query.get(*id).unwrap().0.get(0) {
                        access.num_query.get_mut(*id).unwrap().0 = var.value();
                    }
                }
                "select()" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut inputs = Vec::new();
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types.0 == 0 {
                                let index = Ord::max(wh.link_types.1, 0) as usize;
                                if index >= inputs.len() {
                                    inputs.resize(index+1, None);
                                }
                                inputs[index] = Some(wh.bh_parent);
                            }
                            if wh.open {
                                wh.open = false;
                                changed = true;
                            }
                        }
                    }
                    if lost || net_changed || changed {
                        let mut nets = Vec::new();
                        for i in inputs {
                            if let Some(i) = i {
                                let net = &access.net_query.get(i).unwrap().0;
                                if net.inputs() == 0 && net.outputs() == 1 {
                                    nets.push(net.clone());
                                }
                            }
                        }
                        access.net_query.get_mut(*id).unwrap().0 = Net32::wrap(Box::new(An(Select::new(nets))));
                        access.net_changed_query.get_mut(*id).unwrap().0 = true;
                    }
                }
                "get()" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-13, 1) && wh.open {
                                let arr = access.arr_query.get(wh.bh_parent).unwrap().0.clone();
                                let net = &mut access.net_query.get_mut(*id).unwrap().0;
                                *net = Net32::wrap(Box::new(An(ArrGet::new(arr))));
                                access.net_changed_query.get_mut(*id).unwrap().0 = true;
                            }
                        }
                    }
                }
                "feedback()" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut net = None;
                    let mut del = None;
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types == (0, 1) {
                                net = Some(Box::new(access.net_query.get(wh.bh_parent).unwrap().0.clone()));
                            } else if wh.link_types == (-1, 2) {
                                del = Some(access.num_query.get(wh.bh_parent).unwrap().0);
                            }
                            if wh.open {
                                wh.open = false;
                                changed = true;
                            }
                        }
                    }
                    if lost || net_changed || changed {
                        if let Some(net) = net {
                            if net.outputs() == net.inputs() {
                                if let Some(del) = del {
                                    let feedback = Net32::wrap(Box::new(Feedback32::new(del, net)));
                                    access.net_query.get_mut(*id).unwrap().0 = feedback;
                                } else {
                                    let feedback = Net32::wrap(Box::new(Feedback32::new(0., net)));
                                    access.net_query.get_mut(*id).unwrap().0 = feedback;
                                }
                                access.net_changed_query.get_mut(*id).unwrap().0 = true;
                            }
                        }
                    }
                }
                "seq()" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut inputs = Vec::new();
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types.0 == 0 {
                                let index = Ord::max(wh.link_types.1, 0) as usize;
                                if index >= inputs.len() {
                                    inputs.resize(index+1, None);
                                }
                                inputs[index] = Some(wh.bh_parent);
                            }
                            if wh.open {
                                wh.open = false;
                                changed = true;
                            }
                        }
                    }
                    if changed || lost || net_changed {
                        let mut nets = Vec::new();
                        for i in inputs {
                            if let Some(i) = i {
                                let net = &access.net_query.get(i).unwrap().0;
                                if net.inputs() == 0 && net.outputs() == 1 {
                                    nets.push(net.clone());
                                }
                            }
                        }
                        access.net_query.get_mut(*id).unwrap().0 = Net32::wrap(Box::new(An(Seq::new(nets))));
                        access.net_changed_query.get_mut(*id).unwrap().0 = true;
                    }
                }
                "wave()" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-13, 1) && wh.open {
                                let arr = &access.arr_query.get(wh.bh_parent).unwrap().0;
                                let net = &mut access.net_query.get_mut(*id).unwrap().0;
                                *net = Net32::wrap(Box::new(
                                    wave32(&std::sync::Arc::new(Wave32::from_samples(44100., arr)), 0, Some(0))
                                ));
                                access.net_changed_query.get_mut(*id).unwrap().0 = true;
                            }
                        }
                    }
                }
                "SUM" | "+" | "PRO" | "*" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut inputs = Vec::new();
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types.0 == 0 {
                                inputs.push(wh.bh_parent);
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
                            let net = access.net_query.get(i).unwrap().0.clone();
                            if empty {
                                graph = net;
                                empty = false;
                            } else if graph.outputs() == net.outputs() {
                                let op = &access.op_query.get(*id).unwrap().0;
                                if op == "+" || op == "SUM" {
                                    graph = graph + net;
                                } else {
                                    graph = graph * net;
                                }
                            }
                        }
                        access.net_changed_query.get_mut(*id).unwrap().0 = true;
                        let output = &mut access.net_query.get_mut(*id).unwrap().0;
                        *output = Net32::wrap(Box::new(graph));
                    }
                }
                "STA" | "|" => {
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut inputs = Vec::new();
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types.0 == 0 {
                                let index = Ord::max(wh.link_types.1, 0) as usize;
                                if index >= inputs.len() {
                                    inputs.resize(index+1, None);
                                }
                                inputs[index] = Some(wh.bh_parent);
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
                                let net = access.net_query.get(i).unwrap().0.clone();
                                if empty {
                                    graph = net;
                                    empty = false;
                                } else {
                                    graph = graph | net;
                                }
                            }
                        }
                        access.net_changed_query.get_mut(*id).unwrap().0 = true;
                        access.net_query.get_mut(*id).unwrap().0 = Net32::wrap(Box::new(graph));
                    }
                }
                "PIP" | ">>" => {
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut inputs = Vec::new();
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types.0 == 0 {
                                let index = Ord::max(wh.link_types.1, 0) as usize;
                                if index >= inputs.len() {
                                    inputs.resize(index+1, None);
                                }
                                inputs[index] = Some(wh.bh_parent);
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
                                let net = access.net_query.get(i).unwrap().0.clone();
                                if empty {
                                    graph = net;
                                    empty = false;
                                }
                                else if graph.outputs() == net.inputs() {
                                    graph = graph >> net;
                                }
                            }
                        }
                        access.net_changed_query.get_mut(*id).unwrap().0 = true;
                        access.net_query.get_mut(*id).unwrap().0 = Net32::wrap(Box::new(graph));
                    }
                }
                "out()" | "dac()" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    //let gained = access.gained_wh_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut net = None;
                    for child in children {
                        if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                            if wh.link_types == (0, 1) {
                                net = Some(wh.bh_parent);
                            }
                            if wh.open {
                                wh.open = false;
                                changed = true;
                            }
                        }
                    }
                    if /*gained ||*/ lost || net_changed || changed {
                        if let Some(net) = net {
                            let net = access.net_query.get(net).unwrap().0.clone();
                            if net.outputs() == 1 && net.inputs() == 0 {
                                slot.0.set(Fade::Smooth, 0.1, Box::new(net | dc(0.)));
                            } else if net.outputs() == 2 && net.inputs() == 0 {
                                slot.0.set(Fade::Smooth, 0.1, Box::new(net));
                            }
                        } else {
                            slot.0.set(Fade::Smooth, 0.1, Box::new(dc(0.) | dc(0.)));
                        }
                    }
                }
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
                        -1 => { input = access.num_query.get(white_hole.bh_parent).unwrap().0; }
                        // radius
                        -2 => { input = access.radius_query.get(white_hole.bh_parent).unwrap().0; }
                        // x
                        -3 => { input = access.trans_query.get(white_hole.bh_parent).unwrap().translation.x; }
                        // y
                        -4 => { input = access.trans_query.get(white_hole.bh_parent).unwrap().translation.y; }
                        // z
                        -5 => { input = access.trans_query.get(white_hole.bh_parent).unwrap().translation.z; }
                        // hue
                        -6 => { input = access.col_query.get(white_hole.bh_parent).unwrap().0.h(); }
                        // saturation
                        -7 => { input = access.col_query.get(white_hole.bh_parent).unwrap().0.s(); }
                        // lightness
                        -8 => { input = access.col_query.get(white_hole.bh_parent).unwrap().0.l(); }
                        // alpha
                        -9 => { input = access.col_query.get(white_hole.bh_parent).unwrap().0.a(); }
                        // order
                        -10 => { input = access.order_query.get(white_hole.bh_parent).unwrap().0 as f32; }
                        // vertices
                        -11 => { input = access.vertices_query.get(white_hole.bh_parent).unwrap().0 as f32; }
                        // rotation
                        -12 => { input = access.trans_query.get(white_hole.bh_parent)
                                               .unwrap().rotation.to_euler(EulerRot::XYZ).2; }
                        _ => {}
                    }
                    match white_hole.link_types.1 {
                        -1 => { access.num_query.get_mut(*id).unwrap().0 = input; }
                        // TODO(amy): why doesn't this panics on 0?
                        -2 => { access.radius_query.get_mut(*id).unwrap().0 = input.max(0.); }
                        -3 => { access.trans_query.get_mut(*id).unwrap().translation.x = input; }
                        -4 => { access.trans_query.get_mut(*id).unwrap().translation.y = input; }
                        -5 => { access.trans_query.get_mut(*id).unwrap().translation.z = input; }
                        -6 => { access.col_query.get_mut(*id).unwrap().0.set_h(input); }
                        -7 => { access.col_query.get_mut(*id).unwrap().0.set_s(input); }
                        -8 => { access.col_query.get_mut(*id).unwrap().0.set_l(input); }
                        -9 => { access.col_query.get_mut(*id).unwrap().0.set_a(input); }
                        -10 => {
                            access.order_query.get_mut(*id).unwrap().0 = input as usize;
                            access.order_change.send_default();
                        }
                        -11 => { access.vertices_query.get_mut(*id).unwrap().0 = input.max(3.) as usize; }
                        -12 => {
                            let q = Quat::from_euler(EulerRot::XYZ, 0., 0., input);
                            access.trans_query.get_mut(*id).unwrap().rotation = q;
                        }
                        _ => {}
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
                }
                "sum()" | "product()" | "pipe()" | "stack()" => {
                    access.net_query.get_mut(*id).unwrap().0 = Net32::new(0,0);
                }
                _ => {}
            }
            // go back to oder 0 (doesn't get processed)
            access.order_query.get_mut(*id).unwrap().0 = 0;
            access.order_change.send_default();
        }
    }
}

pub fn update_constant_num(
    mut query: Query<(&Op, &mut crate::components::Num), Changed<Op>>,
) {
    for (op, mut num) in query.iter_mut() {
        if let Ok(n) = parse_with_constants(op.0.as_str()) {
            num.0 = n;
        }
    }
}

fn parse_with_constants(s: &str) -> Result<f32, &str> {
    if let Ok(n) = s.parse::<f32>() {
        Ok(n)
    } else {
        match s {
            "E" => Ok(std::f32::consts::E),
            "FRAC_1_PI" => Ok(std::f32::consts::FRAC_1_PI),
            "FRAC_1_SQRT_2" => Ok(std::f32::consts::FRAC_1_SQRT_2),
            "FRAC_2_PI" => Ok(std::f32::consts::FRAC_2_PI),
            "FRAC_2_SQRT_PI" => Ok(std::f32::consts::FRAC_2_SQRT_PI),
            "FRAC_PI_2" => Ok(std::f32::consts::FRAC_PI_2),
            "FRAC_PI_3" => Ok(std::f32::consts::FRAC_PI_3),
            "FRAC_PI_4" => Ok(std::f32::consts::FRAC_PI_4),
            "FRAC_PI_6" => Ok(std::f32::consts::FRAC_PI_6),
            "FRAC_PI_8" => Ok(std::f32::consts::FRAC_PI_8),
            "LN_2" => Ok(std::f32::consts::LN_2),
            "LN_10" => Ok(std::f32::consts::LN_10),
            "LOG2_10" => Ok(std::f32::consts::LOG2_10),
            "LOG2_E" => Ok(std::f32::consts::LOG2_E),
            "LOG10_2" => Ok(std::f32::consts::LOG10_2),
            "LOG10_E" => Ok(std::f32::consts::LOG10_E),
            "PI" => Ok(std::f32::consts::PI),
            "SQRT_2" => Ok(std::f32::consts::SQRT_2),
            "TAU" => Ok(std::f32::consts::TAU),
            _ => Err("not a float nor a constant"),
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
                if let Ok(n) = parse_with_constants(s) {
                    p.push(n);
                }
            }
        } else { continue; } // no parentheses
        match args[0] {
            "var" => {
                let input = shared(0.);
                n.0 = Net32::wrap(Box::new(var(&input)));
                inputs.0.push(input);
            }
            // TODO(amy): add the other modes
            "monitor" => {
                let s = shared(0.);
                n.0 = Net32::wrap(Box::new(monitor(&s, Meter::Sample)));
                inputs.0.push(s);
            }
            "meter" => {
                if let (Some(arg), Some(p)) = (args.get(1), p.get(0)) {
                    if arg.starts_with("peak") {
                        info!("{} {}", arg, *p);
                        n.0 = Net32::wrap(Box::new(meter(Meter::Peak(*p as f64))));
                    } else if arg.starts_with("rms") {
                        info!("{} {}", arg, *p);
                        n.0 = Net32::wrap(Box::new(meter(Meter::Rms(*p as f64))));
                    }
                }
            }
            "timer" => {
                let s = shared(0.);
                n.0 = Net32::wrap(Box::new(timer(&s)));
                inputs.0.push(s);
            }
            "sink" => { n.0 = Net32::wrap(Box::new(sink())); }
            "pass" => { n.0 = Net32::wrap(Box::new(pass())); }
            "stack" => { n.0 = Net32::new(0,0); }
            "pipe" => { n.0 = Net32::new(0,0); }

            "panner" => { n.0 = Net32::wrap(Box::new(panner())); }

            "sine" => { n.0 = Net32::wrap(Box::new(sine())); }
            "saw" => { n.0 = Net32::wrap(Box::new(saw())); }
            "square" => { n.0 = Net32::wrap(Box::new(square())); }
            "triangle" => { n.0 = Net32::wrap(Box::new(triangle())); }
            "organ" => { n.0 = Net32::wrap(Box::new(organ())); }

            "pulse" => { n.0 = Net32::wrap(Box::new(pulse())); }
            "brown" => { n.0 = Net32::wrap(Box::new(brown())); }
            "pink" => { n.0 = Net32::wrap(Box::new(pink())); }
            "white" | "noise" => { n.0 = Net32::wrap(Box::new(white())); }

            "allpass" => { n.0 = Net32::wrap(Box::new(allpass())); }
            "allpole" => { n.0 = Net32::wrap(Box::new(allpole())); }
            "bandpass" => { n.0 = Net32::wrap(Box::new(bandpass())); }
            "bandrez" => { n.0 = Net32::wrap(Box::new(bandrez())); }
            "bell" => { n.0 = Net32::wrap(Box::new(bell())); }
            "butterpass" => { n.0 = Net32::wrap(Box::new(butterpass())); }
            "clip" => { n.0 = Net32::wrap(Box::new(clip())); }
            "dcblock" => { n.0 = Net32::wrap(Box::new(dcblock())); }
            "declick" => { n.0 = Net32::wrap(Box::new(declick())); }
            "dsf_saw" => { n.0 = Net32::wrap(Box::new(dsf_saw())); }
            "dsf_square" => { n.0 = Net32::wrap(Box::new(dsf_square())); }
            "hammond" => { n.0 = Net32::wrap(Box::new(hammond())); }
            "highpass" => { n.0 = Net32::wrap(Box::new(highpass())); }
            "highpole" => { n.0 = Net32::wrap(Box::new(highpole())); }
            "highshelf" => { n.0 = Net32::wrap(Box::new(highshelf())); }
            "lorenz" => { n.0 = Net32::wrap(Box::new(lorenz())); }
            "lowpass" => { n.0 = Net32::wrap(Box::new(lowpass())); }
            "lowpole" => { n.0 = Net32::wrap(Box::new(lowpole())); }
            "lowrez" => { n.0 = Net32::wrap(Box::new(lowrez())); }
            "lowshelf" => { n.0 = Net32::wrap(Box::new(lowshelf())); }
            "mls" => { n.0 = Net32::wrap(Box::new(mls())); }
            "moog" => { n.0 = Net32::wrap(Box::new(moog())); }
            "morph" => { n.0 = Net32::wrap(Box::new(morph())); }
            "notch" => { n.0 = Net32::wrap(Box::new(notch())); }
            "peak" => { n.0 = Net32::wrap(Box::new(peak())); }
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
            }
            "sine_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(sine_hz(*p)));
                }
            }
            "saw_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(saw_hz(*p)));
                }
            }
            "square_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(square_hz(*p)));
                }
            }
            "triangle_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(triangle_hz(*p)));
                }
            }
            "organ_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(organ_hz(*p)));
                }
            }

            "add" => {
                match p[..] {
                    [p0,p1,p2,p3,p4,p5,p6,p7,..] => { n.0 = Net32::wrap(Box::new(add((p0,p1,p2,p3,p4,p5,p6,p7)))); }
                    [p0,p1,p2,p3,p4,p5,p6,..] => { n.0 = Net32::wrap(Box::new(add((p0,p1,p2,p3,p4,p5,p6)))); }
                    [p0,p1,p2,p3,p4,p5,..] => { n.0 = Net32::wrap(Box::new(add((p0,p1,p2,p3,p4,p5)))); }
                    [p0,p1,p2,p3,p4,..] => { n.0 = Net32::wrap(Box::new(add((p0,p1,p2,p3,p4)))); }
                    [p0,p1,p2,p3,..] => { n.0 = Net32::wrap(Box::new(add((p0,p1,p2,p3)))); }
                    [p0,p1,p2,..] => { n.0 = Net32::wrap(Box::new(add((p0,p1,p2)))); }
                    [p0,p1,..] => { n.0 = Net32::wrap(Box::new(add((p0,p1)))); }
                    [p0,..] => { n.0 = Net32::wrap(Box::new(add(p0))); }
                    _ => { n.0 = Net32::wrap(Box::new(add(1.))); }
                }
            }
            "sub" => {
                match p[..] {
                    [p0,p1,p2,p3,p4,p5,p6,p7,..] => { n.0 = Net32::wrap(Box::new(sub((p0,p1,p2,p3,p4,p5,p6,p7)))); }
                    [p0,p1,p2,p3,p4,p5,p6,..] => { n.0 = Net32::wrap(Box::new(sub((p0,p1,p2,p3,p4,p5,p6)))); }
                    [p0,p1,p2,p3,p4,p5,..] => { n.0 = Net32::wrap(Box::new(sub((p0,p1,p2,p3,p4,p5)))); }
                    [p0,p1,p2,p3,p4,..] => { n.0 = Net32::wrap(Box::new(sub((p0,p1,p2,p3,p4)))); }
                    [p0,p1,p2,p3,..] => { n.0 = Net32::wrap(Box::new(sub((p0,p1,p2,p3)))); }
                    [p0,p1,p2,..] => { n.0 = Net32::wrap(Box::new(sub((p0,p1,p2)))); }
                    [p0,p1,..] => { n.0 = Net32::wrap(Box::new(sub((p0,p1)))); }
                    [p0,..] => { n.0 = Net32::wrap(Box::new(sub(p0))); }
                    _ => { n.0 = Net32::wrap(Box::new(sub(1.))); }
                }
            }
            "adsr" => {
                if let Some(p) = p.get(0..4) {
                    n.0 = Net32::wrap(Box::new(adsr_live(p[0], p[1], p[2], p[3])));
                }
            }
            "allpass_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(allpass_hz(p[0], p[1])));
                }
            }
            "allpass_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(allpass_q(*p)));
                }
            }
            "allpole_delay" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(allpole_delay(*p)));
                }
            }
            "bandpass_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(bandpass_hz(p[0], p[1])));
                }
            }
            "bandpass_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(bandpass_q(*p)));
                }
            }
            "bandrez_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(bandrez_hz(p[0], p[1])));
                }
            }
            "bandrez_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(bandrez_q(*p)));
                }
            }
            "bell_hz" => {
                if let Some(p) = p.get(0..3) {
                    n.0 = Net32::wrap(Box::new(bell_hz(p[0], p[1], p[2])));
                }
            }
            "bell_q" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(bell_q(p[0], p[1])));
                }
            }
            "biquad" => {
                if let Some(p) = p.get(0..5) {
                    n.0 = Net32::wrap(Box::new(biquad(p[0],p[1],p[2],p[3],p[4])));
                }
            }
            "butterpass_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(butterpass_hz(*p)));
                }
            }
            "chorus" => {
                if let Some(p) = p.get(0..4) {
                    n.0 = Net32::wrap(Box::new(chorus(p[0] as i64, p[1], p[2], p[3])));
                }
            }
            "clip_to" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(clip_to(p[0], p[1])));
                }
            }
            "constant" | "dc" => {
                match p[..] {
                    [p0,p1,p2,p3,p4,p5,p6,p7,..] => { n.0 = Net32::wrap(Box::new(constant((p0,p1,p2,p3,p4,p5,p6,p7)))); }
                    [p0,p1,p2,p3,p4,p5,p6,..] => { n.0 = Net32::wrap(Box::new(constant((p0,p1,p2,p3,p4,p5,p6)))); }
                    [p0,p1,p2,p3,p4,p5,..] => { n.0 = Net32::wrap(Box::new(constant((p0,p1,p2,p3,p4,p5)))); }
                    [p0,p1,p2,p3,p4,..] => { n.0 = Net32::wrap(Box::new(constant((p0,p1,p2,p3,p4)))); }
                    [p0,p1,p2,p3,..] => { n.0 = Net32::wrap(Box::new(constant((p0,p1,p2,p3)))); }
                    [p0,p1,p2,..] => { n.0 = Net32::wrap(Box::new(constant((p0,p1,p2)))); }
                    [p0,p1,..] => { n.0 = Net32::wrap(Box::new(constant((p0,p1)))); }
                    [p0,..] => { n.0 = Net32::wrap(Box::new(constant(p0))); }
                    _ => { n.0 = Net32::wrap(Box::new(constant(1.))); }
                }
            }
            "dcblock_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(dcblock_hz(*p)));
                }
            }
            "declick_s" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(declick_s(*p)));
                }
            }
            "delay" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(delay(*p)));
                }
            }
            "dsf_saw_r" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(dsf_saw_r(*p)));
                }
            }
            "dsf_square_r" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(dsf_square_r(*p)));
                }
            }
            "fir" => {
                match p[..] {
                    [p0,p1,p2,p3,p4,p5,p6,p7,p8,p9,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5,p6,p7,p8,p9)))); }
                    [p0,p1,p2,p3,p4,p5,p6,p7,p8,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5,p6,p7,p8)))); }
                    [p0,p1,p2,p3,p4,p5,p6,p7,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5,p6,p7)))); }
                    [p0,p1,p2,p3,p4,p5,p6,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5,p6)))); }
                    [p0,p1,p2,p3,p4,p5,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4,p5)))); }
                    [p0,p1,p2,p3,p4,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3,p4)))); }
                    [p0,p1,p2,p3,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2,p3)))); }
                    [p0,p1,p2,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1,p2)))); }
                    [p0,p1,..] => { n.0 = Net32::wrap(Box::new(fir((p0,p1)))); }
                    [p0,..] => { n.0 = Net32::wrap(Box::new(fir(p0))); }
                    _ => {}
                }
            }
            "fir3" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(An(fir3(*p))));
                }
            }
            "follow" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(follow((p[0], p[1]))));
                } else if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(follow(*p)));
                }
            }
            "hammond_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(hammond_hz(*p)));
                }
            }
            "highpass_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(highpass_hz(p[0], p[1])));
                }
            }
            "highpass_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(highpass_q(*p)));
                }
            }
            "highpole_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(highpole_hz(*p)));
                }
            }
            "highshelf_hz" => {
                if let Some(p) = p.get(0..3) {
                    n.0 = Net32::wrap(Box::new(highshelf_hz(p[0], p[1], p[2])));
                }
            }
            "highshelf_q" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(highshelf_q(p[0], p[1])));
                }
            }
            "hold" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(hold(*p)));
                }
            }
            "hold_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(hold_hz(p[0], p[1])));
                }
            }
            "join" => {
                if let Some(p) = p.get(0) {
                    match *p as usize {
                        2 => { n.0 = Net32::wrap(Box::new(join::<U2>())); }
                        3 => { n.0 = Net32::wrap(Box::new(join::<U3>())); }
                        4 => { n.0 = Net32::wrap(Box::new(join::<U4>())); }
                        5 => { n.0 = Net32::wrap(Box::new(join::<U5>())); }
                        6 => { n.0 = Net32::wrap(Box::new(join::<U6>())); }
                        7 => { n.0 = Net32::wrap(Box::new(join::<U7>())); }
                        8 => { n.0 = Net32::wrap(Box::new(join::<U8>())); }
                        _ => {}
                    }
                }
            }
            "split" => {
                if let Some(p) = p.get(0) {
                    match *p as usize {
                        2 => { n.0 = Net32::wrap(Box::new(split::<U2>())); }
                        3 => { n.0 = Net32::wrap(Box::new(split::<U3>())); }
                        4 => { n.0 = Net32::wrap(Box::new(split::<U4>())); }
                        5 => { n.0 = Net32::wrap(Box::new(split::<U5>())); }
                        6 => { n.0 = Net32::wrap(Box::new(split::<U6>())); }
                        7 => { n.0 = Net32::wrap(Box::new(split::<U7>())); }
                        8 => { n.0 = Net32::wrap(Box::new(split::<U8>())); }
                        _ => {}
                    }
                }
            }
            "reverse" => {
                if let Some(p) = p.get(0) {
                    match *p as usize {
                        2 => { n.0 = Net32::wrap(Box::new(reverse::<U2>())); }
                        3 => { n.0 = Net32::wrap(Box::new(reverse::<U3>())); }
                        4 => { n.0 = Net32::wrap(Box::new(reverse::<U4>())); }
                        5 => { n.0 = Net32::wrap(Box::new(reverse::<U5>())); }
                        6 => { n.0 = Net32::wrap(Box::new(reverse::<U6>())); }
                        7 => { n.0 = Net32::wrap(Box::new(reverse::<U7>())); }
                        8 => { n.0 = Net32::wrap(Box::new(reverse::<U8>())); }
                        _ => {}
                    }
                }
            }
            "limiter" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(limiter((p[0], p[1]))));
                } else if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(limiter(*p)));
                }
            }
            "limiter_stereo" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(limiter_stereo((p[0], p[1]))));
                } else if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(limiter_stereo(*p)));
                }
            }
            "lowpass_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(lowpass_hz(p[0], p[1])));
                }
            }
            "lowpass_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(lowpass_q(*p)));
                }
            }
            "lowpole_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(lowpole_hz(*p)));
                }
            }
            "lowrez_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(lowrez_hz(p[0], p[1])));
                }
            }
            "lowrez_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(lowrez_q(*p)));
                }
            }
            "lowshelf_hz" => {
                if let Some(p) = p.get(0..3) {
                    n.0 = Net32::wrap(Box::new(lowshelf_hz(p[0], p[1], p[2])));
                }
            }
            "lowshelf_q" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(lowshelf_q(p[0], p[1])));
                }
            }
            "mls_bits" => {
                if let Some(p) = p.get(0) {
                    let p = *p as i64;
                    if p >= 1 && p <= 31 {
                        n.0 = Net32::wrap(Box::new(mls_bits(p)));
                    }
                }
            }
            "moog_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(moog_hz(p[0], p[1])));
                }
            }
            "moog_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(moog_q(*p)));
                }
            }
            "morph_hz" => {
                if let Some(p) = p.get(0..3) {
                    n.0 = Net32::wrap(Box::new(morph_hz(p[0], p[1], p[2])));
                }
            }
            "mul" => {
                match p[..] {
                    [p0,p1,p2,p3,p4,p5,p6,p7,..] => { n.0 = Net32::wrap(Box::new(mul((p0,p1,p2,p3,p4,p5,p6,p7)))); }
                    [p0,p1,p2,p3,p4,p5,p6,..] => { n.0 = Net32::wrap(Box::new(mul((p0,p1,p2,p3,p4,p5,p6)))); }
                    [p0,p1,p2,p3,p4,p5,..] => { n.0 = Net32::wrap(Box::new(mul((p0,p1,p2,p3,p4,p5)))); }
                    [p0,p1,p2,p3,p4,..] => { n.0 = Net32::wrap(Box::new(mul((p0,p1,p2,p3,p4)))); }
                    [p0,p1,p2,p3,..] => { n.0 = Net32::wrap(Box::new(mul((p0,p1,p2,p3)))); }
                    [p0,p1,p2,..] => { n.0 = Net32::wrap(Box::new(mul((p0,p1,p2)))); }
                    [p0,p1,..] => { n.0 = Net32::wrap(Box::new(mul((p0,p1)))); }
                    [p0,..] => { n.0 = Net32::wrap(Box::new(mul(p0))); }
                    _ => { n.0 = Net32::wrap(Box::new(mul(1.))); }
                }
            }
            "div" => {
                match p[..] {
                    [p0,p1,p2,p3,p4,p5,p6,p7,..] => {
                        n.0 = Net32::wrap(Box::new(mul((1./p0,1./p1,1./p2,1./p3,1./p4,1./p5,1./p6,1./p7))));
                    }
                    [p0,p1,p2,p3,p4,p5,p6,..] => {
                        n.0 = Net32::wrap(Box::new(mul((1./p0,1./p1,1./p2,1./p3,1./p4,1./p5,1./p6))));
                    }
                    [p0,p1,p2,p3,p4,p5,..] => {
                        n.0 = Net32::wrap(Box::new(mul((1./p0,1./p1,1./p2,1./p3,1./p4,1./p5))));
                    }
                    [p0,p1,p2,p3,p4,..] => {
                        n.0 = Net32::wrap(Box::new(mul((1./p0,1./p1,1./p2,1./p3,1./p4))));
                    }
                    [p0,p1,p2,p3,..] => {
                        n.0 = Net32::wrap(Box::new(mul((1./p0,1./p1,1./p2,1./p3))));
                    }
                    [p0,p1,p2,..] => {
                        n.0 = Net32::wrap(Box::new(mul((1./p0,1./p1,1./p2))));
                    }
                    [p0,p1,..] => {
                        n.0 = Net32::wrap(Box::new(mul((1./p0,1./p1))));
                    }
                    [p0,..] => {
                        n.0 = Net32::wrap(Box::new(mul(1./p0)));
                    }
                    _ => { n.0 = Net32::wrap(Box::new(mul(1.))); }
                }
            }
            "notch_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(notch_hz(p[0], p[1])));
                }
            }
            "notch_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(notch_q(*p)));
                }
            }
            "peak_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(peak_hz(p[0], p[1])));
                }
            }
            "peak_q" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(peak_q(*p)));
                }
            }
            "pluck" => {
                if let Some(p) = p.get(0..3) {
                    n.0 = Net32::wrap(Box::new(pluck(p[0], p[1], p[2])));
                }
            }
            "resonator_hz" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(resonator_hz(p[0], p[1])));
                }
            }
            "reverb_stereo" => {
                if let Some(p) = p.get(0..2) {
                    n.0 = Net32::wrap(Box::new(reverb_stereo(p[0].into(), p[1].into())));
                }
            }
            "soft_saw_hz" => {
                if let Some(p) = p.get(0) {
                    n.0 = Net32::wrap(Box::new(soft_saw_hz(*p)));
                }
            }
            "tap" => {
                if let Some(p) = p.get(0..2) {
                    if p[0] <= p[1] {
                        n.0 = Net32::wrap(Box::new(tap(p[0], p[1])));
                    } else {
                        n.0 = Net32::wrap(Box::new(tap(p[1], p[0])));
                    }
                }
            }

            "ramp" => {
                n.0 = Net32::wrap(
                    Box::new(
                        lfo_in(|t, i: &Frame<f32, U1>| (t*i[0]).rem_euclid(1.))
                    )
                );
            }
            "clock" => {
                n.0 = Net32::wrap(Box::new(sine() >> map(|i: &Frame<f32,U1>| if i[0] > 0. {1.} else {0.})));
            }
            "rise" => {
                n.0 = Net32::wrap(Box::new((pass() ^ tick()) >> map(|i: &Frame<f32,U2>| if i[0]>i[1] {1.} else {0.})));
            }
            "fall" => {
                n.0 = Net32::wrap(Box::new((pass() ^ tick()) >> map(|i: &Frame<f32,U2>| if i[0]<i[1] {1.} else {0.})));
            }

            ">" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| if i[0]>i[1] {1.} else {0.}))); }
            "<" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| if i[0]<i[1] {1.} else {0.}))); }
            "==" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| if i[0]==i[1] {1.} else {0.}))); }
            "!=" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| if i[0]!=i[1] {1.} else {0.}))); }
            ">=" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| if i[0]>=i[1] {1.} else {0.}))); }
            "<=" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| if i[0]<=i[1] {1.} else {0.}))); }

            "abs" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].abs()))); }
            "mod" | "rem" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| i[0].rem_euclid(i[1])))); }
            "signum" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].signum()))); }
            "min" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| i[0].min(i[1])))); }
            "max" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| i[0].max(i[1])))); }
            "pow" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| i[0].pow(i[1])))); }
            "floor" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].floor()))); }
            "fract" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].fract()))); }
            "ceil" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].ceil()))); }
            "round" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].round()))); }
            "sqrt" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].sqrt()))); }
            "exp" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].exp()))); }
            "exp2" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].exp2()))); }
            "exp10" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| (exp10(i[0]))))); }
            "exp_m1" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| (i[0].ln_1p())))); }
            "ln_1p" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| (i[0].exp_m1())))); }
            "ln" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].ln()))); }
            "log" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| i[0].log(i[1])))); }
            "log2" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].log2()))); }
            "log10" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].log10()))); }
            "sin" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].sin()))); }
            "cos" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].cos()))); }
            "tan" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].tan()))); }
            "asin" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].asin()))); }
            "acos" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].acos()))); }
            "atan" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].atan()))); }
            "sinh" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].sinh()))); }
            "cosh" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].cosh()))); }
            "tanh" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].tanh()))); }
            "asinh" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].asinh()))); }
            "acosh" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].acosh()))); }
            "atanh" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0].atanh()))); }
            "squared" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0] * i[0]))); }
            "cubed" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| i[0] * i[0] * i[0]))); }
            "lerp" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| lerp(i[0], i[1], i[2])))); }
            "lerp11" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| lerp11(i[0], i[1], i[2])))); }
            "delerp" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| delerp(i[0], i[1], i[2])))); }
            "delerp11" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| delerp11(i[0], i[1], i[2])))); }
            "xerp" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| xerp(i[0], i[1], i[2])))); }
            "xerp11" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| xerp11(i[0], i[1], i[2])))); }
            "dexerp" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| dexerp(i[0], i[1], i[2])))); }
            "dexerp11" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| dexerp11(i[0], i[1], i[2])))); }
            "dissonance" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| dissonance(i[0], i[1])))); }
            "dissonance_max" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| dissonance_max(i[0])))); }
            "db_amp" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| db_amp(i[0])))); }
            "amp_db" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| amp_db(i[0])))); }
            "a_weight" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| a_weight(i[0])))); }
            "m_weight" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| m_weight(i[0])))); }
            "spline" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U5>| spline(i[0], i[1], i[2], i[3], i[4])))); }
            "spline_mone" => {n.0=Net32::wrap(Box::new(map(|i:&Frame<f32,U5>| spline_mono(i[0],i[1],i[2],i[3],i[4]))));}
            "softsign" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| softsign(i[0])))); }
            "softexp" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| softexp(i[0])))); }
            "softmix" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U3>| softmix(i[0], i[1], i[2])))); }
            "smooth3" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| smooth3(i[0])))); }
            "smooth5" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| smooth5(i[0])))); }
            "smooth7" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| smooth7(i[0])))); }
            "smooth9" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| smooth9(i[0])))); }
            "uparc" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| uparc(i[0])))); }
            "downarc" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| downarc(i[0])))); }
            "sine_ease" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| sine_ease(i[0])))); }
            "sin_hz" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| sin_hz(i[0], i[1])))); }
            "cos_hz" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| cos_hz(i[0], i[1])))); }
            "sqr_hz" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| sqr_hz(i[0], i[1])))); }
            "tri_hz" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U2>| tri_hz(i[0], i[1])))); }
            "semitone_ratio" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| semitone_ratio(i[0])))); }
            "rnd" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| rnd(i[0] as i64) as f32))); }
            "rnd2" => { n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32, U1>| rnd2(i[0] as i64) as f32))); }
            "spline_noise" => {
                n.0 = Net32::wrap(Box::new(map(|i: &Frame<f32,U2>| {
                    spline_noise(i[0] as i64, i[1]) as f32
                })));
            }
            "fractal_noise" => {
                n.0=Net32::wrap(Box::new(map(|i:&Frame<f32,U4>| {
                    fractal_noise(i[0] as i64,i[1].min(1.) as i64,i[2],i[3]) as f32
                })));
            }

            _ => { n.0 = Net32::wrap(Box::new(dc(0.))); }
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
    arr_query: Query<&Children, Changed<Arr>>,
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
    for children in arr_query.iter() {
        for child in children {
            if let Ok(bh) = black_hole_query.get(*child) {
                if let Ok(wh) = white_hole_query.get(bh.wh) {
                    if wh.link_types.0 == -13 {
                        white_hole_query.get_mut(bh.wh).unwrap().open = true;
                    }
                }
            }
        }
    }
}
