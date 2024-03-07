use bevy::{
    ecs::system::SystemParam,
    winit::{WinitSettings, UpdateMode},
    utils::Duration,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
        },
    render::view::screenshot::ScreenshotManager,
    input::keyboard::{KeyboardInput, Key},
    prelude::*
};

use fundsp::hacker32::*;

use crate::{
    components::*,
    nodes::*,
};

pub fn sort_by_order(
    query: Query<(Entity, &Order)>,
    mut queue: ResMut<Queue>,
) {
    let mut max_order: usize = 1;
    queue.0.clear();
    queue.0.push(Vec::new());
    for (entity, order) in query.iter() {
        if order.0 > 0 {
            if order.0 > max_order {
                queue.0.resize(order.0, Vec::new());
                max_order = order.0;
            }
            queue.0[order.0 - 1].push(entity); //order 1 at index 0
        }
    }
}

pub fn prepare_loop_queue(
    mut loopq: ResMut<LoopQueue>,
    queue: Res<Queue>,
    op_query: Query<&Op>,
    targets_query: Query<&Targets>,
) {
    loopq.0.clear();
    for id in queue.0.iter().flatten() {
        if op_query.get(*id).unwrap().0 == "process" {
            let targets = &targets_query.get(*id).unwrap().0;
            for t in targets {
                // only add existing circles (that aren't holes)
                if op_query.contains(*t) { loopq.0.push(*t); }
            }
        }
    }
}

#[derive(SystemParam)]
pub struct Access<'w, 's> {
    order_query: Query<'w, 's, &'static mut Order>,
    op_query: Query<'w, 's, &'static mut Op>,
    bloom: Query<'w, 's, & 'static mut BloomSettings, With<Camera>>,
    num_query: Query<'w, 's, &'static mut Number>,
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
    clear_color: ResMut<'w, ClearColor>,
    default_color: ResMut<'w, DefaultDrawColor>,
    default_verts: ResMut<'w, DefaultDrawVerts>,
    highlight_color: ResMut<'w, HighlightColor>,
    connection_color: ResMut<'w, ConnectionColor>,
    selection_circle: Res<'w, SelectionCircle>,
    connecting_line: Res<'w, ConnectingLine>,
}

pub fn process(
    queue: Res<Queue>,
    loopq: Res<LoopQueue>,
    children_query: Query<Ref<Children>>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<&BlackHole>,
    mut access: Access,
    mut slot: ResMut<Slot>,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut key_event: EventReader<KeyboardInput>,
    camera_query: Query<(Entity, &Camera, &GlobalTransform)>,
    windows: Query<(Entity, &Window)>,
    mut ortho: Query<&mut OrthographicProjection>,
) {
    for id in queue.0.iter().flatten().chain(loopq.0.iter()) {
        if let Ok(children) = &children_query.get(*id) {
            match access.op_query.get(*id).unwrap().0.as_str() {
                "empty" => {}
                "open_target" | "close_targets" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-1, 1) {
                                if access.num_query.get(wh.bh_parent).unwrap().0 == 1. {
                                    let targets = &access.targets_query.get(*id).unwrap().0;
                                    for t in targets {
                                        if let Ok(mut wh) = white_hole_query.get_mut(*t) {
                                            if access.op_query.get(*id).unwrap().0 == "open_target" {
                                                wh.open = true;
                                            } else {
                                                wh.open = false;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "reorder" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-1, 1) && wh.open {
                                let n = access.num_query.get(wh.bh_parent).unwrap().0;
                                let targets = &access.targets_query.get(*id).unwrap().0;
                                for t in targets {
                                    if let Ok(mut order) = access.order_query.get_mut(*t) {
                                        order.0 = n as usize;
                                        access.order_change.send_default();
                                    }
                                }
                            }
                        }
                    }
                }
                "repeat" => {
                    let mut arr = None;
                    let mut targets = None;
                    let mut n = None;
                    let mut changed = false;
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-1, 1) {
                                n = Some(access.num_query.get(wh.bh_parent).unwrap().0 as usize);
                            } else if wh.link_types == (-13, 2) { arr = Some(wh.bh_parent); }
                            else if wh.link_types == (-14, 2) { targets = Some(wh.bh_parent); }
                            if wh.open { changed = true; }
                        }
                    }
                    if changed {
                        if let Some(n) = n {
                            if let Some(arr) = arr {
                                let repeated = access.arr_query.get(arr).unwrap().0.repeat(n);
                                access.arr_query.get_mut(*id).unwrap().0 = repeated;
                            }
                            if let Some(tar) = targets {
                                let repeated = access.targets_query.get(tar).unwrap().0.repeat(n);
                                access.targets_query.get_mut(*id).unwrap().0 = repeated;
                            }
                        }
                    }
                }
                "zip" => {
                    let mut arr1 = None;
                    let mut arr2 = None;
                    let mut changed = false;
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-13, 1) { arr1 = Some(wh.bh_parent); }
                            else if wh.link_types == (-13, 2) { arr2 = Some(wh.bh_parent); }
                            if wh.open { changed = true; }
                        }
                    }
                    if changed {
                        if let (Some(arr1), Some(arr2)) = (arr1, arr2) {
                            let a1 = access.arr_query.get(arr1).unwrap().0.clone();
                            let a2 = access.arr_query.get(arr2).unwrap().0.clone();
                            let n = Ord::max(a1.len(), a2.len());
                            let out = &mut access.arr_query.get_mut(*id).unwrap().0;
                            out.clear();
                            for i in 0..n {
                                if let Some(x) = a1.get(i) { out.push(*x); }
                                if let Some(y) = a2.get(i) { out.push(*y); }
                            }
                        }
                        // TODO(mara): zip for targets?
                    }
                }
                "unzip" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-13, 1) && wh.open {
                                let input = &access.arr_query.get(wh.bh_parent).unwrap().0;
                                let mut l = Vec::new();
                                let mut r = Vec::new();
                                for i in 0..input.len() {
                                    if i & 1 == 0 { l.push(input[i]); } else { r.push(input[i]); }
                                }
                                access.arr_query.get_mut(wh.bh_parent).unwrap()
                                    .bypass_change_detection().0 = l;
                                access.arr_query.get_mut(*id).unwrap().0 = r;
                            }
                        }
                    }
                }
                "push" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-1, 1) && wh.open {
                                let n = access.num_query.get_mut(wh.bh_parent).unwrap().0;
                                access.arr_query.get_mut(*id).unwrap().0.push(n);
                            }
                        }
                    }
                }
                "pop" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-1, 1)
                            && access.num_query.get(wh.bh_parent).unwrap().0 == 1. {
                                if let Some(n) = access.arr_query.get_mut(*id).unwrap().0.pop() {
                                    access.num_query.get_mut(*id).unwrap().0 = n;
                                }
                            }
                        }
                    }
                }
                "len" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-13, 1) && wh.open {
                                let len = access.arr_query.get(wh.bh_parent).unwrap().0.len() as f32;
                                access.num_query.get_mut(*id).unwrap().0 = len;
                            } else if wh.link_types == (-14, 1) && wh.open {
                                let len = access.targets_query.get(wh.bh_parent).unwrap().0.len() as f32;
                                access.num_query.get_mut(*id).unwrap().0 = len;
                            }
                        }
                    }
                }
                "append" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-13, 1) && wh.open {
                                let arr = &mut access.arr_query.get(wh.bh_parent).unwrap().0.clone();
                                access.arr_query.get_mut(*id).unwrap().0.append(arr);
                            }
                        }
                    }
                }
                "slice" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-13, 1) && wh.open {
                                let arr = &mut access.arr_query.get_mut(wh.bh_parent).unwrap();
                                let n = access.num_query.get(*id).unwrap().0 as usize;
                                if n <= arr.0.len() {
                                    let slice = arr.bypass_change_detection().0.split_off(n);
                                    access.arr_query.get_mut(*id).unwrap().0 = slice;
                                }
                            }
                        }
                    }
                }
                "resize" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (-1, 1) && wh.open {
                                let n = access.num_query.get(wh.bh_parent).unwrap().0 as usize;
                                access.arr_query.get_mut(*id).unwrap().0.resize(n,0.);
                            }
                        }
                    }
                }
                "clear_color" => {
                    let color = access.col_query.get_mut(*id).unwrap();
                    if color.is_changed() {
                        access.clear_color.0 = color.0;
                    }
                }
                "draw_verts" => {
                    let verts = access.vertices_query.get_mut(*id).unwrap();
                    if verts.is_changed() {
                        access.default_verts.0 = verts.0;
                    }
                }
                "draw_color" => {
                    let color = access.col_query.get_mut(*id).unwrap();
                    if color.is_changed() {
                        access.default_color.0 = color.0;
                    }
                }
                "highlight_color" => {
                    let color = access.col_query.get_mut(*id).unwrap();
                    if color.is_changed() {
                        access.highlight_color.0 = color.0;
                    }
                }
                "selection_color" => {
                    let color = access.col_query.get_mut(*id).unwrap();
                    if color.is_changed() {
                        let id = access.selection_circle.0;
                        access.col_query.get_mut(id).unwrap().0 = color.0;
                    }
                }
                "connection_color" => {
                    let color = access.col_query.get_mut(*id).unwrap();
                    if color.is_changed() {
                        access.connection_color.0 = color.0;
                    }
                }
                "connecting_line_color" => {
                    let color = access.col_query.get_mut(*id).unwrap();
                    if color.is_changed() {
                        let id = access.connecting_line.0;
                        access.col_query.get_mut(id).unwrap().0 = color.0;
                    }
                }
                "cam" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types.0 == -1 && wh.open {
                                let n = access.num_query.get(wh.bh_parent).unwrap().0;
                                let id = camera_query.single().0;
                                let t = &mut access.trans_query.get_mut(id).unwrap();
                                match wh.link_types.1 {
                                    1 => { t.translation.x = n; }
                                    2 => { t.translation.y = n; }
                                    3 => { t.translation.z = n; }
                                    4 => { t.rotation = Quat::from_euler(EulerRot::XYZ,0.,0.,n); }
                                    5 => { ortho.single_mut().scale = n.clamp(0.1, 4.); }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                // make it take input num
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
                "mouse" => {
                    let (_, cam, cam_transform) = camera_query.single();
                    if let Some(cursor_pos) = windows.single().1.cursor_position() {
                        if let Some(point) = cam.viewport_to_world_2d(cam_transform, cursor_pos) {
                            access.arr_query.get_mut(*id).unwrap().0 = point.to_array().into();
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
                    for key in key_event.read() {
                        if key.state.is_pressed() {
                            match &key.logical_key {
                                Key::Character(c) => {
                                    if let Some(c) = c.chars().nth(0) {
                                        let c = (c as i32) as f32;
                                        access.arr_query.get_mut(*id).unwrap().0.push(c);
                                    }
                                }
                                // TODO(amy): add the other variants
                                _ => {}
                            }
                        } else {
                            match &key.logical_key {
                                Key::Character(c) => {
                                    if let Some(c) = c.chars().nth(0) {
                                        let c = (c as i32) as f32;
                                        access.arr_query.get_mut(*id).unwrap().0.retain(|&x| x != c);
                                    }
                                }
                                _ => {}
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
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types.0 == -1 {
                                let index = Ord::max(wh.link_types.1, 0) as usize;
                                if index >= inputs.len() {
                                    inputs.resize(index+1, None);
                                }
                                inputs[index] = Some(access.num_query.get(wh.bh_parent).unwrap().0);
                            }
                            if wh.open {
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
                "var()" => {
                    if access.net_changed_query.get(*id).unwrap().0 {
                        let net = &mut access.net_query.get_mut(*id).unwrap().0;
                        let inputs = &mut access.net_ins_query.get_mut(*id).unwrap().0;
                        let input = shared(0.);
                        *net = Net32::wrap(Box::new(var(&input)));
                        inputs.push(input);
                    }
                    let num = access.num_query.get_mut(*id).unwrap();
                    if num.is_changed() {
                        if let Some(var) = &access.net_ins_query.get(*id).unwrap().0.get(0) {
                            var.set_value(num.0);
                        }
                    }
                }
                // TODO(amy): add the other monitor modes
                "monitor()" | "timer()" => {
                    if access.net_changed_query.get(*id).unwrap().0 {
                        let net = &mut access.net_query.get_mut(*id).unwrap().0;
                        let inputs = &mut access.net_ins_query.get_mut(*id).unwrap().0;
                        let s = shared(0.);
                        if access.op_query.get(*id).unwrap().0 == "monitor()" {
                            *net = Net32::wrap(Box::new(monitor(&s, Meter::Sample)));
                        } else {
                            *net = Net32::wrap(Box::new(timer(&s)));
                        }
                        inputs.push(s);
                    }
                    if let Some(var) = access.net_ins_query.get(*id).unwrap().0.get(0) {
                        access.num_query.get_mut(*id).unwrap().0 = var.value();
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
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (0, 1) {
                                net = Some(Box::new(access.net_query.get(wh.bh_parent).unwrap().0.clone()));
                            } else if wh.link_types == (-1, 2) {
                                del = Some(access.num_query.get(wh.bh_parent).unwrap().0);
                            }
                            if wh.open {
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
                "seq()" | "select()" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut inputs = Vec::new();
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types.0 == 0 {
                                let index = Ord::max(wh.link_types.1, 0) as usize;
                                if index >= inputs.len() {
                                    inputs.resize(index+1, None);
                                }
                                inputs[index] = Some(wh.bh_parent);
                            }
                            if wh.open {
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
                        let n = &mut access.net_query.get_mut(*id).unwrap().0;
                        if access.op_query.get(*id).unwrap().0 == "select()" {
                            *n = Net32::wrap(Box::new(An(Select::new(nets))));
                        } else {
                            *n = Net32::wrap(Box::new(An(Seq::new(nets))));
                        }
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
                "arrdc()" | "arrdelay()" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    let arr = &access.arr_query.get_mut(*id).unwrap();
                    if arr.is_changed() || net_changed {
                        let mut graph = Net32::new(0,0);
                        for i in &arr.0 {
                            if access.op_query.get(*id).unwrap().0 == "arrdc()" {
                                graph = graph | dc(*i);
                            } else {
                                graph = graph | delay(*i);
                            }
                        }
                        access.net_query.get_mut(*id).unwrap().0 = Net32::wrap(Box::new(graph));
                        access.net_changed_query.get_mut(*id).unwrap().0 = true;
                    }
                }
                "SUM" | "+" | "PRO" | "*" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let num_changed = access.num_query.get_mut(*id).unwrap().is_changed();
                    let mut changed = false;
                    let mut inputs = Vec::new();
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types.0 == 0 {
                                inputs.push(wh.bh_parent);
                            }
                            if wh.open {
                                changed = true;
                            }
                        }
                    }
                    if changed || lost || net_changed || num_changed {
                        let mut graph = Net32::new(0,0);
                        let mut empty = true;
                        let n = access.num_query.get(*id).unwrap().0.max(1.) as i32;
                        for _ in 0..n {
                            for i in &inputs {
                                let net = access.net_query.get(*i).unwrap().0.clone();
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
                        }
                        access.net_changed_query.get_mut(*id).unwrap().0 = true;
                        let output = &mut access.net_query.get_mut(*id).unwrap().0;
                        *output = Net32::wrap(Box::new(graph));
                    }
                }
                ">>" | "|" | "&" | "^" | "PIP" | "STA" | "BUS" | "BRA" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let num_changed = access.num_query.get_mut(*id).unwrap().is_changed();
                    let mut changed = false;
                    let mut inputs = Vec::new();
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types.0 == 0 {
                                let index = Ord::max(wh.link_types.1, 0) as usize;
                                if index >= inputs.len() {
                                    inputs.resize(index+1, None);
                                }
                                inputs[index] = Some(wh.bh_parent);
                            }
                            if wh.open {
                                changed = true;
                            }
                        }
                    }
                    if changed || lost || net_changed || num_changed {
                        let mut graph = Net32::new(0,0);
                        let mut empty = true;
                        let n = access.num_query.get(*id).unwrap().0.max(1.) as i32;
                        for _ in 0..n {
                            for i in &inputs {
                                if let Some(i) = i {
                                    let net = access.net_query.get(*i).unwrap().0.clone();
                                    if empty {
                                        graph = net;
                                        empty = false;
                                    }
                                    else {
                                        let (gi, go) = (graph.inputs(), graph.outputs());
                                        let (ni, no) = (net.inputs(), net.outputs());
                                        match access.op_query.get(*id).unwrap().0.as_str() {
                                            ">>" | "PIP" => {
                                                if go == ni { graph = graph >> net; }
                                            }
                                            "|" | "STA" => {
                                                graph = graph | net;
                                            }
                                            "&" | "BUS" => {
                                                if gi == ni && go == no { graph = graph & net; }
                                            }
                                            "^" | "BRA" => {
                                                if gi == ni { graph = graph ^ net; }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        access.net_changed_query.get_mut(*id).unwrap().0 = true;
                        access.net_query.get_mut(*id).unwrap().0 = Net32::wrap(Box::new(graph));
                    }
                }
                "!" | "THR" => {
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (0, 1) && wh.open {
                                let input = access.net_query.get(wh.bh_parent).unwrap().0.clone();
                                access.net_query.get_mut(*id).unwrap().0 = !input;
                            }
                        }
                    }
                }
                "out()" | "dac()" => {
                    let net_changed = access.net_changed_query.get(*id).unwrap().0;
                    //let gained = access.gained_wh_query.get(*id).unwrap().0;
                    let lost = access.lost_wh_query.get(*id).unwrap().0;
                    let mut changed = false;
                    let mut net = None;
                    for child in children {
                        if let Ok(wh) = white_hole_query.get(*child) {
                            if wh.link_types == (0, 1) {
                                net = Some(wh.bh_parent);
                            }
                            if wh.open {
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
                if let Ok(mut wh) = white_hole_query.get_mut(*child) {
                    if !wh.open { continue; }
                    let mut input = 0.;
                    match wh.link_types.0 {
                        // num
                        -1 => { input = access.num_query.get(wh.bh_parent).unwrap().0; }
                        // radius
                        -2 => { input = access.radius_query.get(wh.bh_parent).unwrap().0; }
                        // x
                        -3 => { input = access.trans_query.get(wh.bh_parent).unwrap().translation.x; }
                        // y
                        -4 => { input = access.trans_query.get(wh.bh_parent).unwrap().translation.y; }
                        // z
                        -5 => { input = access.trans_query.get(wh.bh_parent).unwrap().translation.z; }
                        // hue
                        -6 => { input = access.col_query.get(wh.bh_parent).unwrap().0.h(); }
                        // saturation
                        -7 => { input = access.col_query.get(wh.bh_parent).unwrap().0.s(); }
                        // lightness
                        -8 => { input = access.col_query.get(wh.bh_parent).unwrap().0.l(); }
                        // alpha
                        -9 => { input = access.col_query.get(wh.bh_parent).unwrap().0.a(); }
                        // vertices
                        -11 => { input = access.vertices_query.get(wh.bh_parent).unwrap().0 as f32; }
                        // rotation
                        -12 => { input = access.trans_query.get(wh.bh_parent)
                                               .unwrap().rotation.to_euler(EulerRot::XYZ).2; }
                        _ => {}
                    }
                    match wh.link_types.1 {
                        -1 => { access.num_query.get_mut(*id).unwrap().0 = input; }
                        -2 => { access.radius_query.get_mut(*id).unwrap().0 = input.max(0.); }
                        -3 => { access.trans_query.get_mut(*id).unwrap().translation.x = input; }
                        -4 => { access.trans_query.get_mut(*id).unwrap().translation.y = input; }
                        -5 => { access.trans_query.get_mut(*id).unwrap().translation.z = input; }
                        -6 => { access.col_query.get_mut(*id).unwrap().0.set_h(input); }
                        -7 => { access.col_query.get_mut(*id).unwrap().0.set_s(input); }
                        -8 => { access.col_query.get_mut(*id).unwrap().0.set_l(input); }
                        -9 => { access.col_query.get_mut(*id).unwrap().0.set_a(input); }
                        -11 => { access.vertices_query.get_mut(*id).unwrap().0 = input.max(3.) as usize; }
                        -12 => {
                            let q = Quat::from_euler(EulerRot::XYZ, 0., 0., input);
                            access.trans_query.get_mut(*id).unwrap().rotation = q;
                        }
                        _ => {}
                    }
                    if wh.link_types == (-13, -13) {
                        let arr = &access.arr_query.get(wh.bh_parent).unwrap().0;
                        access.arr_query.get_mut(*id).unwrap().0 = arr.clone();
                    }
                    if wh.link_types == (-14, -14) {
                        let arr = &access.targets_query.get(wh.bh_parent).unwrap().0;
                        access.targets_query.get_mut(*id).unwrap().0 = arr.clone();
                    }
                    lt_to_open = wh.link_types.1;
                    wh.open = false;
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
                "out()" | "dac()" => {
                    slot.0.set(Fade::Smooth, 0.1, Box::new(dc(0.) | dc(0.)));
                }
                _ => {}
            }
        }
    }
}

// open the white holes reading any changed value
// it's gonna overlap with whatever `process` changed, but that's okay
// process needs to do things in order, but this is to catch any external change
// TODO(amy): bypass_change_detection in `process`
// TODO(amy): Changed is actually expensive (== looping and checking is_changed)
// can we do better?
pub fn open_white_holes(
    num_query: Query<&Children, Changed<Number>>,
    radius_query: Query<&Children, Changed<Radius>>,
    trans_query: Query<&Children, Changed<Transform>>,
    col_query: Query<&Children, Changed<Col>>,
    order_query: Query<&Children, Changed<Order>>,
    vertices_query: Query<&Children, Changed<Vertices>>,
    arr_query: Query<&Children, Changed<Arr>>,
    targets_query: Query<&Children, Changed<Targets>>,
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
    for children in targets_query.iter() {
        for child in children {
            if let Ok(bh) = black_hole_query.get(*child) {
                if let Ok(wh) = white_hole_query.get(bh.wh) {
                    if wh.link_types.0 == -14 {
                        white_hole_query.get_mut(bh.wh).unwrap().open = true;
                    }
                }
            }
        }
    }
}
