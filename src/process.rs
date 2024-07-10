use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    ecs::system::SystemParam,
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
    render::view::{screenshot::ScreenshotManager, RenderLayers},
    utils::Duration,
    winit::{UpdateMode, WinitSettings},
};

use std::str::FromStr;

use fundsp::hacker32::*;

use crate::{components::*, functions::*, nodes::*, osc::*};

pub fn sort_by_order(query: Query<(Entity, &Order), With<Network>>, mut queue: ResMut<Queue>) {
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
                if op_query.contains(*t) {
                    loopq.0.push(*t);
                }
            }
        }
    }
}

#[derive(SystemParam)]
pub struct Access<'w, 's> {
    order_query: Query<'w, 's, &'static mut Order>,
    op_query: Query<'w, 's, &'static mut Op>,
    bloom: Query<'w, 's, &'static mut BloomSettings, With<Camera>>,
    num_query: Query<'w, 's, &'static mut Number>,
    trans_query: Query<'w, 's, &'static mut Transform>,
    arr_query: Query<'w, 's, &'static mut Arr>,
    tonemapping: Query<'w, 's, &'static mut Tonemapping, With<Camera>>,
    net_query: Query<'w, 's, &'static mut Network>,
    net_ins_query: Query<'w, 's, &'static mut NetIns>,
    net_chan_query: Query<'w, 's, &'static mut NetChannel>,
    col_query: Query<'w, 's, &'static mut Col>,
    order_change: EventWriter<'w, OrderChange>,
    vertices_query: Query<'w, 's, &'static mut Vertices>,
    op_changed_query: Query<'w, 's, &'static mut OpChanged>,
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
    connection_width: ResMut<'w, ConnectionWidth>,
    arrow_query: Query<'w, 's, &'static ConnectionArrow>,
    indicator: Res<'w, Indicator>,
    indicator_color: ResMut<'w, IndicatorColor>,
    command_line_text: Query<'w, 's, &'static mut Text, With<CommandText>>,
    command_color: ResMut<'w, CommandColor>,
    selected_query: Query<'w, 's, Entity, With<Selected>>,
    delete_event: EventWriter<'w, DeleteCommand>,
    polygon_handles: Res<'w, PolygonHandles>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
    connection_mat: ResMut<'w, ConnectionMat>,
    connect_command: EventWriter<'w, ConnectCommand>,
    info_text_query: Query<'w, 's, &'static InfoText>,
    text_size: ResMut<'w, TextSize>,
    osc_sender: ResMut<'w, OscSender>,
    osc_receiver: ResMut<'w, OscReceiver>,
    osc_messages: Local<'s, Vec<rosc::OscMessage>>,
    node_limit: Res<'w, NodeLimit>,
    input_receivers: Res<'w, InputReceivers>,
    op_num_query: Query<'w, 's, &'static OpNum>,
    key_event: EventReader<'w, 's, KeyboardInput>,
    ortho: Query<'w, 's, &'static mut OrthographicProjection>,
}

pub fn process(
    queue: Res<Queue>,
    loopq: Res<LoopQueue>,
    holes_query: Query<&Holes>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<&BlackHole>,
    mut access: Access,
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(Entity, &Camera, &GlobalTransform)>,
    windows: Query<(Entity, &Window)>,
    mut commands: Commands,
    mut slot: ResMut<SlotRes>,
) {
    let key_event = access.key_event.read().collect::<Vec<_>>();
    for id in queue.0.iter().flatten().chain(loopq.0.iter()) {
        let holes = &holes_query.get(*id).unwrap().0;
        for hole in holes {
            let mut lt_to_open = 0;
            if let Ok(wh) = white_hole_query.get(*hole) {
                if !wh.open {
                    continue;
                }
                if wh.link_types.0 == -13 && wh.link_types.1 != -13 {
                    continue;
                }
                let mut input = 0.;
                match wh.link_types.0 {
                    -1 => input = access.num_query.get(wh.bh_parent).unwrap().0,
                    -2 => input = access.trans_query.get(wh.bh_parent).unwrap().scale.x,
                    -3 => input = access.trans_query.get(wh.bh_parent).unwrap().translation.x,
                    -4 => input = access.trans_query.get(wh.bh_parent).unwrap().translation.y,
                    -5 => input = access.trans_query.get(wh.bh_parent).unwrap().translation.z,
                    -6 => input = access.col_query.get(wh.bh_parent).unwrap().0.hue,
                    -7 => input = access.col_query.get(wh.bh_parent).unwrap().0.saturation,
                    -8 => input = access.col_query.get(wh.bh_parent).unwrap().0.lightness,
                    -9 => input = access.col_query.get(wh.bh_parent).unwrap().0.alpha,
                    -11 => input = access.vertices_query.get(wh.bh_parent).unwrap().0 as f32,
                    -12 => {
                        input = access
                            .trans_query
                            .get(wh.bh_parent)
                            .unwrap()
                            .rotation
                            .to_euler(EulerRot::XYZ)
                            .2
                    }
                    _ => {}
                }
                match wh.link_types.1 {
                    -1 => access.num_query.get_mut(*id).unwrap().0 = input,
                    -2 => {
                        access.trans_query.get_mut(*id).unwrap().scale.x = input.max(0.);
                        access.trans_query.get_mut(*id).unwrap().scale.y = input.max(0.);
                    }
                    -3 => access.trans_query.get_mut(*id).unwrap().translation.x = input,
                    -4 => access.trans_query.get_mut(*id).unwrap().translation.y = input,
                    -5 => access.trans_query.get_mut(*id).unwrap().translation.z = input,
                    -6 => access.col_query.get_mut(*id).unwrap().0.hue = input,
                    -7 => access.col_query.get_mut(*id).unwrap().0.saturation = input,
                    -8 => access.col_query.get_mut(*id).unwrap().0.lightness = input,
                    -9 => access.col_query.get_mut(*id).unwrap().0.alpha = input,
                    -11 => {
                        access.vertices_query.get_mut(*id).unwrap().0 =
                            (input as usize).clamp(3, 64)
                    }
                    -12 => {
                        let q = Quat::from_euler(EulerRot::XYZ, 0., 0., input);
                        access.trans_query.get_mut(*id).unwrap().rotation = q;
                    }
                    _ => {}
                }
                if wh.link_types == (-13, -13) {
                    let arr = access.arr_query.get(wh.bh_parent).unwrap().0.clone();
                    access.arr_query.get_mut(*id).unwrap().0 = arr;
                }
                if wh.link_types == (-14, -14) {
                    let arr = access.targets_query.get(wh.bh_parent).unwrap().0.clone();
                    access.targets_query.get_mut(*id).unwrap().0 = arr;
                }
                lt_to_open = wh.link_types.1;
            }
            // open all white holes reading whatever just changed
            if lt_to_open != 0 {
                for hole in holes {
                    if let Ok(bh) = black_hole_query.get(*hole) {
                        if let Ok(wh) = white_hole_query.get(bh.wh) {
                            if wh.link_types.0 == lt_to_open {
                                white_hole_query.get_mut(bh.wh).unwrap().open = true;
                            }
                        }
                    }
                }
            }
        }
        let mut lt_to_open = None;
        let op = access.op_query.get(*id).unwrap().0.as_str();
        let op_num = access.op_num_query.get(*id).unwrap().0;
        match op_num {
            0 => {}
            // -------------------- targets --------------------
            // open_target | close_target
            1 | 2 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1)
                            && access.num_query.get(wh.bh_parent).unwrap().0 != 0.
                        {
                            let targets = &access.targets_query.get(*id).unwrap().0;
                            for t in targets {
                                if let Ok(mut wh) = white_hole_query.get_mut(*t) {
                                    wh.open = op_num == 1;
                                }
                            }
                        }
                    }
                }
            }
            // open_nth
            3 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1) && wh.open {
                            let n = access.num_query.get(wh.bh_parent).unwrap().0;
                            let targets = &access.targets_query.get(*id).unwrap().0;
                            if let Some(nth) = targets.get(n as usize) {
                                if let Ok(mut wh) = white_hole_query.get_mut(*nth) {
                                    wh.open = true;
                                }
                            }
                        }
                    }
                }
            }
            // del_target
            4 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1)
                            && access.num_query.get(wh.bh_parent).unwrap().0 != 0.
                        {
                            for e in access.selected_query.iter() {
                                commands.entity(e).remove::<Selected>();
                            }
                            for t in &access.targets_query.get(*id).unwrap().0 {
                                if access.vertices_query.contains(*t) {
                                    commands.entity(*t).insert(Selected);
                                }
                            }
                            access.targets_query.get_mut(*id).unwrap().0.clear();
                            access.delete_event.send_default();
                        }
                    }
                }
            }
            // select_target
            5 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1) && wh.open {
                            let n = access.num_query.get(wh.bh_parent).unwrap().0;
                            if n != 0. {
                                for t in &access.targets_query.get(*id).unwrap().0 {
                                    if access.vertices_query.contains(*t) {
                                        commands.entity(*t).insert(Selected);
                                    }
                                }
                            } else {
                                for t in &access.targets_query.get(*id).unwrap().0 {
                                    if access.vertices_query.contains(*t) {
                                        commands.entity(*t).remove::<Selected>();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // spin_target
            6 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1)
                            && access.num_query.get(wh.bh_parent).unwrap().0 != 0.
                        {
                            let point = access.trans_query.get(*id).unwrap().translation;
                            let n = access.num_query.get(*id).unwrap().0;
                            let rotation = Quat::from_euler(EulerRot::XYZ, 0., 0., n);
                            for t in &access.targets_query.get(*id).unwrap().0 {
                                if let Ok(mut trans) = access.trans_query.get_mut(*t) {
                                    trans.rotate_around(point, rotation);
                                }
                            }
                        }
                    }
                }
            }
            // reorder
            7 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
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
            // spawn
            8 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1)
                            && access.num_query.get(wh.bh_parent).unwrap().0 != 0.
                        {
                            let targets = &mut access.targets_query.get_mut(*id).unwrap().0;
                            let v = access.vertices_query.get(*id).unwrap().0;
                            let trans = access.trans_query.get(*id).unwrap();
                            let t = trans.translation.xy();
                            let r = trans.scale.x;
                            let depth = trans.translation.z + (targets.len() + 1) as f32 * 0.01;
                            let color = access.col_query.get(*id).unwrap().0;
                            let new = commands
                                .spawn((
                                    ColorMesh2dBundle {
                                        mesh: access.polygon_handles.0[v].clone().unwrap(),
                                        material: access
                                            .materials
                                            .add(ColorMaterial::from_color(color)),
                                        transform: Transform {
                                            translation: t.extend(depth),
                                            rotation: trans.rotation,
                                            scale: Vec3::new(r, r, 1.),
                                        },
                                        ..default()
                                    },
                                    Vertices(v),
                                    Col(color),
                                    Number(0.),
                                    Arr(Vec::new()),
                                    Op("empty".to_string()),
                                    Targets(Vec::new()),
                                    Holes(Vec::new()),
                                    Order(0),
                                    (
                                        OpNum(0),
                                        Network(Net::new(0, 0)),
                                        NetIns(Vec::new()),
                                        OpChanged(false),
                                        GainedWH(false),
                                        LostWH(false),
                                    ),
                                    RenderLayers::layer(1),
                                    Save,
                                ))
                                .id();
                            targets.push(new);
                            lt_to_open = Some(-14);
                        }
                    }
                }
            }
            // connect_target
            9 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1)
                            && wh.open
                            && access.num_query.get(wh.bh_parent).unwrap().0 != 0.
                        {
                            access
                                .targets_query
                                .get_mut(*id)
                                .unwrap()
                                .0
                                .retain(|x| holes_query.contains(*x));
                            access.connect_command.send(ConnectCommand(*id));
                        }
                    }
                }
            }
            // isolate_target
            10 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1)
                            && wh.open
                            && access.num_query.get(wh.bh_parent).unwrap().0 != 0.
                        {
                            for e in access.selected_query.iter() {
                                commands.entity(e).remove::<Selected>();
                            }
                            for t in &access.targets_query.get(*id).unwrap().0 {
                                if let Ok(holes) = holes_query.get(*t) {
                                    for hole in &holes.0 {
                                        commands.entity(*hole).insert(Selected);
                                    }
                                }
                            }
                            access.delete_event.send_default();
                        }
                    }
                }
            }
            // target_lt
            11 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1) && wh.open {
                            let n = access.num_query.get(wh.bh_parent).unwrap().0;
                            for t in &access.targets_query.get(*id).unwrap().0 {
                                if let Ok(mut wh) = white_hole_query.get_mut(*t) {
                                    wh.link_types.1 = n as i8;
                                } else if let Ok(bh) = black_hole_query.get(*t) {
                                    white_hole_query.get_mut(bh.wh).unwrap().link_types.0 = n as i8;
                                }
                            }
                        }
                    }
                }
            }
            // distro
            12 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types.0 == -13 && wh.open {
                            let arr = &access.arr_query.get(wh.bh_parent).unwrap().0;
                            let targets = &access.targets_query.get(*id).unwrap().0;
                            let len = Ord::min(arr.len(), targets.len());
                            for i in 0..len {
                                if access.vertices_query.get(targets[i]).is_err() {
                                    continue;
                                }
                                // input link type determines what property to write to in targets
                                match wh.link_types.1 {
                                    -1 => {
                                        if let Ok(mut n) = access.num_query.get_mut(targets[i]) {
                                            n.0 = arr[i];
                                        }
                                    }
                                    -2 => {
                                        access.trans_query.get_mut(targets[i]).unwrap().scale.x =
                                            arr[i].max(0.1);
                                        access.trans_query.get_mut(targets[i]).unwrap().scale.y =
                                            arr[i].max(0.1);
                                    }
                                    -3 => {
                                        access
                                            .trans_query
                                            .get_mut(targets[i])
                                            .unwrap()
                                            .translation
                                            .x = arr[i];
                                    }
                                    -4 => {
                                        access
                                            .trans_query
                                            .get_mut(targets[i])
                                            .unwrap()
                                            .translation
                                            .y = arr[i];
                                    }
                                    -5 => {
                                        access
                                            .trans_query
                                            .get_mut(targets[i])
                                            .unwrap()
                                            .translation
                                            .z = arr[i];
                                    }
                                    -6 => {
                                        access.col_query.get_mut(targets[i]).unwrap().0.hue =
                                            arr[i];
                                    }
                                    -7 => {
                                        access
                                            .col_query
                                            .get_mut(targets[i])
                                            .unwrap()
                                            .0
                                            .saturation = arr[i];
                                    }
                                    -8 => {
                                        access.col_query.get_mut(targets[i]).unwrap().0.lightness =
                                            arr[i];
                                    }
                                    -9 => {
                                        access.col_query.get_mut(targets[i]).unwrap().0.alpha =
                                            arr[i];
                                    }
                                    -10 => {
                                        if let Ok(mut ord) = access.order_query.get_mut(targets[i]) {
                                            ord.0 = arr[i] as usize;
                                            access.order_change.send_default();
                                        }
                                    }
                                    -11 => {
                                        let v = arr[i].max(3.) as usize;
                                        access.vertices_query.get_mut(targets[i]).unwrap().0 = v;
                                    }
                                    -12 => {
                                        let q = Quat::from_euler(EulerRot::XYZ, 0., 0., arr[i]);
                                        access.trans_query.get_mut(targets[i]).unwrap().rotation =
                                            q;
                                    }
                                    _ => {}
                                }
                            }
                            let lt = wh.link_types.1;
                            for t in &access.targets_query.get(*id).unwrap().0 {
                                if let Ok(holes) = &holes_query.get(*t) {
                                    for hole in &holes.0 {
                                        if let Ok(bh) = black_hole_query.get(*hole) {
                                            if let Ok(wh) = white_hole_query.get_mut(bh.wh) {
                                                if wh.link_types.0 == lt {
                                                    white_hole_query.get_mut(bh.wh).unwrap().open =
                                                        true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // repeat
            13 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-14, 1) && wh.open {
                            let n = access.num_query.get(*id).unwrap().0 as usize;
                            let targets = &access.targets_query.get(wh.bh_parent).unwrap().0;
                            access.targets_query.get_mut(*id).unwrap().0 = targets.repeat(n);
                            lt_to_open = Some(-14);
                        }
                    }
                }
            }
            // -------------------- arrays --------------------
            // zip
            14 => {
                let mut arr1 = None;
                let mut arr2 = None;
                let mut changed = false;
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-13, 1) {
                            arr1 = Some(wh.bh_parent);
                        } else if wh.link_types == (-13, 2) {
                            arr2 = Some(wh.bh_parent);
                        }
                        if wh.open {
                            changed = true;
                        }
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
                            if let Some(x) = a1.get(i) {
                                out.push(*x);
                            }
                            if let Some(y) = a2.get(i) {
                                out.push(*y);
                            }
                        }
                        lt_to_open = Some(-13);
                    }
                }
            }
            // unzip
            15 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-13, 1) && wh.open {
                            let input = &access.arr_query.get(wh.bh_parent).unwrap().0;
                            let mut l = Vec::new();
                            let mut r = Vec::new();
                            for (i, v) in input.iter().enumerate() {
                                if i & 1 == 0 {
                                    l.push(*v);
                                } else {
                                    r.push(*v);
                                }
                            }
                            access.arr_query.get_mut(wh.bh_parent).unwrap().0 = l;
                            access.arr_query.get_mut(*id).unwrap().0 = r;
                            lt_to_open = Some(-13);
                        }
                    }
                }
            }
            // push
            16 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1) && wh.open {
                            let n = access.num_query.get_mut(wh.bh_parent).unwrap().0;
                            access.arr_query.get_mut(*id).unwrap().0.push(n);
                            lt_to_open = Some(-13);
                        }
                    }
                }
            }
            // pop
            17 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1)
                            && access.num_query.get(wh.bh_parent).unwrap().0 != 0.
                        {
                            if let Some(n) = access.arr_query.get_mut(*id).unwrap().0.pop() {
                                access.num_query.get_mut(*id).unwrap().0 = n;
                                lt_to_open = Some(-1);
                            }
                        }
                    }
                }
            }
            // len
            18 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-13, 1) && wh.open {
                            let len = access.arr_query.get(wh.bh_parent).unwrap().0.len() as f32;
                            access.num_query.get_mut(*id).unwrap().0 = len;
                            lt_to_open = Some(-1);
                        } else if wh.link_types == (-14, 1) && wh.open {
                            let len =
                                access.targets_query.get(wh.bh_parent).unwrap().0.len() as f32;
                            access.num_query.get_mut(*id).unwrap().0 = len;
                            lt_to_open = Some(-1);
                        }
                    }
                }
            }
            // append
            19 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-13, 1) && wh.open {
                            let arr = &mut access.arr_query.get(wh.bh_parent).unwrap().0.clone();
                            access.arr_query.get_mut(*id).unwrap().0.append(arr);
                            lt_to_open = Some(-13);
                        }
                    }
                }
            }
            // slice
            20 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-13, 1) && wh.open {
                            let arr = &mut access.arr_query.get_mut(wh.bh_parent).unwrap();
                            let n = access.num_query.get(*id).unwrap().0 as usize;
                            if n <= arr.0.len() {
                                let slice = arr.0.split_off(n);
                                access.arr_query.get_mut(*id).unwrap().0 = slice;
                                lt_to_open = Some(-13);
                            }
                        }
                    }
                }
            }
            // resize
            21 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1) && wh.open {
                            let n = access.num_query.get(wh.bh_parent).unwrap().0 as usize;
                            access.arr_query.get_mut(*id).unwrap().0.resize(n, 0.);
                            lt_to_open = Some(-13);
                        }
                    }
                }
            }
            // contains
            22 => {
                let mut changed = false;
                let mut arr = None;
                let mut n = None;
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-13, 1) {
                            arr = Some(wh.bh_parent);
                            if wh.open {
                                changed = true;
                            }
                        }
                        if wh.link_types == (-1, 2) {
                            n = Some(wh.bh_parent);
                            if wh.open {
                                changed = true;
                            }
                        }
                    }
                }
                if changed {
                    if let (Some(arr), Some(n)) = (arr, n) {
                        let n = access.num_query.get(n).unwrap().0;
                        if access.arr_query.get(arr).unwrap().0.contains(&n) {
                            access.num_query.get_mut(*id).unwrap().0 = 1.;
                        } else {
                            access.num_query.get_mut(*id).unwrap().0 = 0.;
                        }
                        lt_to_open = Some(-1);
                    }
                }
            }
            // set
            23 => {
                let mut changed = false;
                let mut ndx = None;
                let mut val = None;
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        // with "store" these act as "cold inlets"
                        if wh.link_types == (-1, 1) {
                            ndx = Some(wh.bh_parent);
                            if wh.open {
                                changed = true;
                            }
                        } else if wh.link_types == (-1, 2) {
                            val = Some(wh.bh_parent);
                            if wh.open {
                                changed = true;
                            }
                        }
                    }
                }
                if changed {
                    if let (Some(ndx), Some(val)) = (ndx, val) {
                        let ndx = access.num_query.get(ndx).unwrap().0.max(0.) as usize;
                        let val = access.num_query.get(val).unwrap().0;
                        if let Some(i) = access.arr_query.get_mut(*id).unwrap().0.get_mut(ndx) {
                            *i = val;
                            lt_to_open = Some(-13);
                        }
                    }
                }
            }
            // get
            24 => {
                let mut arr = None;
                let mut n = None;
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-13, 1) {
                            arr = Some(wh.bh_parent);
                        }
                        if wh.link_types == (-1, 2) && wh.open {
                            n = Some(access.num_query.get(wh.bh_parent).unwrap().0);
                        }
                    }
                }
                if let (Some(arr), Some(n)) = (arr, n) {
                    if let Some(v) = access.arr_query.get(arr).unwrap().0.get(n as usize) {
                        access.num_query.get_mut(*id).unwrap().0 = *v;
                        lt_to_open = Some(-1);
                    }
                }
            }
            // collect
            25 => {
                let lost = access.lost_wh_query.get(*id).unwrap().0;
                let mut changed = false;
                let mut inputs = Vec::new();
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types.0 == -1 {
                            let index = Ord::max(wh.link_types.1, 0) as usize;
                            if index >= inputs.len() {
                                inputs.resize(index + 1, None);
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
                    for i in inputs.iter().flatten() {
                        arr.push(*i);
                    }
                    lt_to_open = Some(-13);
                }
            }
            // -------------------- settings --------------------
            // clear_color
            26 => {
                let color = access.col_query.get_mut(*id).unwrap();
                if color.is_changed() {
                    access.clear_color.0 = color.0.into();
                }
            }
            // draw_verts
            27 => {
                let verts = access.vertices_query.get_mut(*id).unwrap();
                if verts.is_changed() {
                    access.default_verts.0 = verts.0;
                }
            }
            // draw_color
            28 => {
                let color = access.col_query.get_mut(*id).unwrap();
                if color.is_changed() {
                    access.default_color.0 = color.0;
                }
            }
            // highlight_color
            29 => {
                let color = access.col_query.get_mut(*id).unwrap();
                if color.is_changed() {
                    access.highlight_color.0 = color.0;
                }
            }
            // indicator_color
            30 => {
                let color = access.col_query.get_mut(*id).unwrap();
                if color.is_changed() {
                    access.indicator_color.0 = color.0;
                    let id = access.indicator.0;
                    access.col_query.get_mut(id).unwrap().0 = color.0;
                }
            }
            // connection_color
            31 => {
                let color = access.col_query.get_mut(*id).unwrap();
                if color.is_changed() {
                    access.connection_color.0 = color.0;
                    let mat_id = &access.connection_mat.0;
                    access.materials.get_mut(mat_id).unwrap().color = color.0.into();
                }
            }
            // command_color
            32 => {
                let color = access.col_query.get_mut(*id).unwrap();
                if color.is_changed() {
                    access.command_color.0 = color.0;
                    let clt = &mut access.command_line_text.single_mut();
                    clt.sections[0].style.color = color.0.into();
                }
            }
            // connection_width
            33 => {
                let n = access.num_query.get_mut(*id).unwrap();
                if n.is_changed() {
                    access.connection_width.0 = n.0;
                    for e in access.arrow_query.iter() {
                        access.trans_query.get_mut(e.0).unwrap().scale.x = n.0;
                    }
                }
            }
            // text_size
            34 => {
                let n = access.num_query.get_mut(*id).unwrap();
                if n.is_changed() {
                    let size = n.0.max(0.1) / 120.;
                    access.text_size.0 = size;
                    for e in access.info_text_query.iter() {
                        let scale = &mut access.trans_query.get_mut(e.0).unwrap().scale;
                        scale.x = size;
                        scale.y = size;
                    }
                }
            }
            // tonemapping
            35 => {
                let mut tm = access.tonemapping.single_mut();
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1) && wh.open {
                            let input = access.num_query.get(wh.bh_parent).unwrap().0;
                            match input as usize {
                                0 => *tm = Tonemapping::None,
                                1 => *tm = Tonemapping::Reinhard,
                                2 => *tm = Tonemapping::ReinhardLuminance,
                                3 => *tm = Tonemapping::AcesFitted,
                                4 => *tm = Tonemapping::AgX,
                                5 => *tm = Tonemapping::SomewhatBoringDisplayTransform,
                                6 => *tm = Tonemapping::TonyMcMapface,
                                7 => *tm = Tonemapping::BlenderFilmic,
                                _ => {}
                            }
                        }
                    }
                }
            }
            // bloom
            36 => {
                let mut bloom_settings = access.bloom.single_mut();
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if !wh.open {
                            continue;
                        }
                        let input = access.num_query.get(wh.bh_parent).unwrap().0;
                        match wh.link_types {
                            (-1, 1) => bloom_settings.intensity = input,
                            (-1, 2) => bloom_settings.low_frequency_boost = input,
                            (-1, 3) => bloom_settings.low_frequency_boost_curvature = input,
                            (-1, 4) => bloom_settings.high_pass_frequency = input,
                            (-1, 5) => {
                                bloom_settings.composite_mode = if input > 0. {
                                    BloomCompositeMode::Additive
                                } else {
                                    BloomCompositeMode::EnergyConserving
                                }
                            }
                            (-1, 6) => bloom_settings.prefilter_settings.threshold = input,
                            (-1, 7) => bloom_settings.prefilter_settings.threshold_softness = input,
                            _ => {}
                        }
                    }
                }
            }
            // -------------------- utils --------------------
            // cam
            37 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types.0 == -1 && wh.open {
                            let n = access.num_query.get(wh.bh_parent).unwrap().0;
                            let id = camera_query.single().0;
                            let t = &mut access.trans_query.get_mut(id).unwrap();
                            match wh.link_types.1 {
                                1 => t.translation.x = n,
                                2 => t.translation.y = n,
                                3 => t.translation.z = n,
                                4 => t.rotation = Quat::from_euler(EulerRot::XYZ, 0., 0., n),
                                5 => access.ortho.single_mut().scale = n.clamp(0.005, 80.),
                                _ => {}
                            }
                        }
                    }
                }
            }
            // update_rate
            38 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1) && wh.open {
                            let n = access.num_query.get(wh.bh_parent).unwrap().0;
                            access.winit_settings.focused_mode = UpdateMode::reactive_low_power(
                                Duration::from_secs_f64((1.0 / n.max(0.01)).into()),
                            );
                        } else if wh.link_types == (-1, 2) && wh.open {
                            let n = access.num_query.get(wh.bh_parent).unwrap().0;
                            access.winit_settings.unfocused_mode = UpdateMode::reactive_low_power(
                                Duration::from_secs_f64((1.0 / n.max(0.01)).into()),
                            );
                        }
                    }
                }
            }
            // command
            39 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (0, 1) && wh.open {
                            let input = access.op_query.get(wh.bh_parent).unwrap().0.clone();
                            access.command_line_text.single_mut().sections[0].value = input;
                        }
                    }
                }
            }
            // screenshot
            40 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1)
                            && access.num_query.get(wh.bh_parent).unwrap().0 != 0.
                        {
                            let win = windows.single().0;
                            let epoch = std::time::UNIX_EPOCH;
                            let now = std::time::SystemTime::now();
                            if let Ok(dur) = now.duration_since(epoch) {
                                let time = dur.as_millis();
                                let path = format!("screenshots/{:?}.png", time);
                                access
                                    .screensot_manager
                                    .save_screenshot_to_disk(win, path)
                                    .unwrap();
                            }
                        }
                    }
                }
            }
            // osc
            41 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1) && wh.open {
                            let port = access.num_query.get(wh.bh_parent).unwrap().0.max(1.) as u16;
                            access.osc_receiver.init(port);
                        } else if wh.link_types == (0, 2) && wh.open {
                            let ip = access.op_query.get(wh.bh_parent).unwrap().0.clone();
                            if std::net::Ipv4Addr::from_str(&ip).is_ok() {
                                access.osc_sender.host = ip;
                            }
                        } else if wh.link_types == (-1, 3) && wh.open {
                            let port = access.num_query.get(wh.bh_parent).unwrap().0.max(1.) as u16;
                            access.osc_sender.port = port;
                        }
                    }
                }
                access.osc_messages.clear();
                if let Some(socket) = &access.osc_receiver.socket {
                    for _ in 0..10 {
                        if let Some(packet) = receive_packet(socket) {
                            let mut buffer = Vec::new();
                            unpacket(packet, &mut buffer);
                            access.osc_messages.append(&mut buffer);
                        }
                    }
                }
            }
            // osc_r
            42 => {
                for message in &access.osc_messages {
                    if op.contains(&message.addr) {
                        let arr = &mut access.arr_query.get_mut(*id).unwrap().0;
                        arr.clear();
                        for arg in message.args.clone() {
                            if let rosc::OscType::Float(f) = arg {
                                arr.push(f);
                            }
                        }
                        lt_to_open = Some(-13);
                    }
                }
            }
            // osc_s
            43 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-13, 1) && wh.open {
                            if let Some(address) = op.get(6..) {
                                let arr = access.arr_query.get(wh.bh_parent).unwrap().0.clone();
                                access.osc_sender.send(address, arr);
                            }
                        }
                    }
                }
            }
            // -------------------- input --------------------
            // mouse
            44 => {
                let (_, cam, cam_transform) = camera_query.single();
                if let Some(cursor_pos) = windows.single().1.cursor_position() {
                    if let Some(point) = cam.viewport_to_world_2d(cam_transform, cursor_pos) {
                        access.arr_query.get_mut(*id).unwrap().0 = point.to_array().into();
                        lt_to_open = Some(-13);
                    }
                }
            }
            // lmb_pressed
            45 => {
                if mouse_button_input.pressed(MouseButton::Left) {
                    access.num_query.get_mut(*id).unwrap().0 = 1.;
                    lt_to_open = Some(-1);
                } else {
                    access.num_query.get_mut(*id).unwrap().0 = 0.;
                    lt_to_open = Some(-1);
                }
            }
            // mmb_pressed
            46 => {
                if mouse_button_input.pressed(MouseButton::Middle) {
                    access.num_query.get_mut(*id).unwrap().0 = 1.;
                    lt_to_open = Some(-1);
                } else {
                    access.num_query.get_mut(*id).unwrap().0 = 0.;
                    lt_to_open = Some(-1);
                }
            }
            // rmb_pressed
            47 => {
                if mouse_button_input.pressed(MouseButton::Right) {
                    access.num_query.get_mut(*id).unwrap().0 = 1.;
                    lt_to_open = Some(-1);
                } else {
                    access.num_query.get_mut(*id).unwrap().0 = 0.;
                    lt_to_open = Some(-1);
                }
            }
            // butt
            48 => {
                if mouse_button_input.just_pressed(MouseButton::Left) {
                    let t = access.trans_query.get(*id).unwrap().translation.xy();
                    let r = access.trans_query.get(*id).unwrap().scale.x;
                    if cursor.i.distance_squared(t) < r * r {
                        access.num_query.get_mut(*id).unwrap().0 = 1.;
                        lt_to_open = Some(-1);
                    }
                }
                if mouse_button_input.just_released(MouseButton::Left) {
                    access.num_query.get_mut(*id).unwrap().0 = 0.;
                    lt_to_open = Some(-1);
                }
            }
            // toggle
            49 => {
                if mouse_button_input.just_pressed(MouseButton::Left) {
                    let t = access.trans_query.get(*id).unwrap().translation.xy();
                    let r = access.trans_query.get(*id).unwrap().scale.x;
                    if cursor.i.distance_squared(t) < r * r {
                        let n = &mut access.num_query.get_mut(*id).unwrap().0;
                        *n = if *n == 0. { 1. } else { 0. };
                        lt_to_open = Some(-1);
                    }
                }
            }
            // key
            50 => {
                for key in &key_event {
                    let mut n = 1729.;
                    match &key.logical_key {
                        Key::Character(c) => {
                            if let Some(c) = c.chars().next() {
                                n = (c as i32) as f32;
                            }
                        }
                        Key::Space => n = 32.,
                        Key::Escape => n = 27.,
                        Key::Enter => n = 10.,
                        Key::Tab => n = 9.,
                        Key::Delete => n = 127.,
                        Key::Backspace => n = 8.,
                        Key::Control => n = 128.,
                        Key::Shift => n = 129.,
                        Key::Alt => n = 130.,
                        Key::Super => n = 131.,
                        Key::Fn => n = 132.,
                        Key::CapsLock => n = 133.,
                        Key::NumLock => n = 134.,
                        Key::ScrollLock => n = 135.,
                        Key::End => n = 136.,
                        Key::Home => n = 137.,
                        Key::PageUp => n = 138.,
                        Key::PageDown => n = 139.,
                        Key::Insert => n = 140.,
                        Key::ContextMenu => n = 141.,
                        Key::ArrowUp => n = 200.,
                        Key::ArrowDown => n = 201.,
                        Key::ArrowLeft => n = 202.,
                        Key::ArrowRight => n = 203.,
                        Key::F1 => n = -1.,
                        Key::F2 => n = -2.,
                        Key::F3 => n = -3.,
                        Key::F4 => n = -4.,
                        Key::F5 => n = -5.,
                        Key::F6 => n = -6.,
                        Key::F7 => n = -7.,
                        Key::F8 => n = -8.,
                        Key::F9 => n = -9.,
                        Key::F10 => n = -10.,
                        Key::F11 => n = -11.,
                        Key::F12 => n = -12.,
                        _ => {}
                    }
                    let arr = &mut access.arr_query.get_mut(*id).unwrap().0;
                    if key.state.is_pressed() {
                        if !arr.contains(&n) {
                            arr.push(n);
                        }
                    } else {
                        arr.retain(|&x| x != n);
                    }
                    lt_to_open = Some(-13);
                }
            }
            // pressed
            51 => {
                for key in &key_event {
                    if let Key::Character(c) = &key.logical_key {
                        if let Some(c) = c.chars().last() {
                            if let Some(arg) = op.get(8..) {
                                if arg.contains(c) {
                                    if key.state.is_pressed() {
                                        access.num_query.get_mut(*id).unwrap().0 = 1.;
                                        lt_to_open = Some(-1);
                                    } else {
                                        access.num_query.get_mut(*id).unwrap().0 = 0.;
                                        lt_to_open = Some(-1);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // -------------------- data management --------------------
            // rise | fall
            // uses the array to store previous num value
            52 | 53 => {
                if access.arr_query.get(*id).unwrap().0.len() != 1 {
                    access.arr_query.get_mut(*id).unwrap().0 = vec![0.];
                }
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1) {
                            let input = access.num_query.get(wh.bh_parent).unwrap().0;
                            let arr = &mut access.arr_query.get_mut(*id).unwrap().0;
                            // rise
                            if (op_num == 52 && input > arr[0])
                            // fall
                            || (op_num == 53 && input < arr[0])
                            {
                                access.num_query.get_mut(*id).unwrap().0 = 1.;
                                lt_to_open = Some(-1);
                            }
                            if input == arr[0] {
                                let n = &mut access.num_query.get_mut(*id).unwrap().0;
                                if *n != 0. {
                                    *n = 0.;
                                    lt_to_open = Some(-1);
                                }
                            }
                            arr[0] = input;
                        }
                    }
                }
            }
            // store
            54 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1) && wh.open {
                            let n = access.num_query.get(wh.bh_parent).unwrap().0;
                            access.num_query.get_mut(*id).unwrap().0 = n;
                            // this op only stores the value, don't open wh
                        }
                    }
                }
            }
            // num_push
            55 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1)
                            && wh.open
                            && access.num_query.get(wh.bh_parent).unwrap().0 != 0.
                        {
                            lt_to_open = Some(-1);
                        }
                    }
                }
            }
            // sum
            56 => {
                let mut out = 0.;
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1) {
                            out += access.num_query.get(wh.bh_parent).unwrap().0;
                        }
                    }
                }
                access.num_query.get_mut(*id).unwrap().0 = out;
                lt_to_open = Some(-1);
            }
            // product
            57 => {
                let mut out = 1.;
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-1, 1) {
                            out *= access.num_query.get(wh.bh_parent).unwrap().0;
                        }
                    }
                }
                access.num_query.get_mut(*id).unwrap().0 = out;
                lt_to_open = Some(-1);
            }
            // apply
            59 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (0, 1) && wh.open {
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
                            lt_to_open = Some(-13);
                        }
                    }
                }
            }
            // render
            60 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (0, 1) && wh.open {
                            let input_net = &access.net_query.get(wh.bh_parent).unwrap().0;
                            access.net_query.get_mut(*id).unwrap().0 = input_net.clone();
                        }
                        if wh.link_types == (-1, 2)
                            && access.num_query.get(wh.bh_parent).unwrap().0 != 0.
                        {
                            let len = access.num_query.get(*id).unwrap().0 as usize;
                            let output = &mut access.arr_query.get_mut(*id).unwrap().0;
                            let net = &mut access.net_query.get_mut(*id).unwrap().0;
                            if net.inputs() == 0 && net.outputs() == 1 {
                                output.clear();
                                for _ in 0..len {
                                    let mut s = [0.0];
                                    net.tick(&[], &mut s);
                                    output.push(s[0])
                                }
                            }
                            lt_to_open = Some(-13);
                        }
                    }
                }
            }
            // -------------------- audio nodes --------------------
            // var()
            61 => {
                if access.op_changed_query.get(*id).unwrap().0 {
                    let net = &mut access.net_query.get_mut(*id).unwrap().0;
                    let inputs = &mut access.net_ins_query.get_mut(*id).unwrap().0;
                    let input = shared(0.);
                    *net = Net::wrap(Box::new(var(&input)));
                    inputs.push(input);
                    lt_to_open = Some(0);
                }
                let num = access.num_query.get_mut(*id).unwrap();
                //if num.is_changed() {
                if let Some(var) = &access.net_ins_query.get(*id).unwrap().0.first() {
                    var.set_value(num.0);
                }
                //}
            }
            // in() | adc()
            62 => {
                if access.op_changed_query.get(*id).unwrap().0 {
                    let net = &mut access.net_query.get_mut(*id).unwrap().0;
                    let lr = access.input_receivers.0.clone();
                    let rr = access.input_receivers.1.clone();
                    *net = Net::wrap(Box::new(An(InputNode::new(lr, rr))));
                    lt_to_open = Some(0);
                }
            }
            // monitor() | timer()
            63 | 64 => {
                if access.op_changed_query.get(*id).unwrap().0 {
                    let net = &mut access.net_query.get_mut(*id).unwrap().0;
                    let inputs = &mut access.net_ins_query.get_mut(*id).unwrap().0;
                    let s = shared(0.);
                    if op_num == 63 {
                        *net = Net::wrap(Box::new(monitor(&s, Meter::Sample)));
                    } else {
                        *net = Net::wrap(Box::new(timer(&s)));
                    }
                    inputs.push(s);
                    lt_to_open = Some(0);
                }
                if let Some(var) = access.net_ins_query.get(*id).unwrap().0.first() {
                    access.num_query.get_mut(*id).unwrap().0 = var.value();
                    lt_to_open = Some(-1);
                }
            }
            // get()
            65 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-13, 1) && wh.open {
                            let arr = access.arr_query.get(wh.bh_parent).unwrap().0.clone();
                            let net = &mut access.net_query.get_mut(*id).unwrap().0;
                            *net = Net::wrap(Box::new(An(ArrGet::new(arr))));
                            lt_to_open = Some(0);
                        }
                    }
                }
            }
            // quantize()
            66 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-13, 1) && wh.open {
                            let arr = &access.arr_query.get(wh.bh_parent).unwrap().0;
                            if let (Some(first), Some(last)) = (arr.first(), arr.last()) {
                                let range = *last - *first;
                                let net = &mut access.net_query.get_mut(*id).unwrap().0;
                                *net = Net::wrap(Box::new(An(Quantizer::new(arr.clone(), range))));
                                lt_to_open = Some(0);
                            }
                        }
                    }
                }
            }
            // feedback()
            67 => {
                let op_changed = access.op_changed_query.get(*id).unwrap().0;
                let lost = access.lost_wh_query.get(*id).unwrap().0;
                let mut changed = false;
                let mut net = None;
                let mut del = None;
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (0, 1) {
                            net = Some(Box::new(
                                access.net_query.get(wh.bh_parent).unwrap().0.clone(),
                            ));
                        } else if wh.link_types == (-1, 2) {
                            del = Some(access.num_query.get(wh.bh_parent).unwrap().0);
                        }
                        if wh.open {
                            changed = true;
                        }
                    }
                }
                if lost || op_changed || changed {
                    if let Some(net) = net {
                        if net.outputs() == net.inputs() {
                            if let Some(del) = del {
                                let feedback =
                                    Net::wrap(Box::new(FeedbackUnit::new(del.into(), net)));
                                access.net_query.get_mut(*id).unwrap().0 = feedback;
                            } else {
                                let feedback = Net::wrap(Box::new(FeedbackUnit::new(0., net)));
                                access.net_query.get_mut(*id).unwrap().0 = feedback;
                            }
                            lt_to_open = Some(0);
                        }
                    } else {
                        access.net_query.get_mut(*id).unwrap().0 = Net::new(0, 0);
                    }
                }
            }
            // swap()
            91 => {
                if access.op_changed_query.get(*id).unwrap().0 {
                    let (s, r) = crossbeam_channel::bounded(1);
                    let op = op.replace(' ', "");
                    let args: Vec<&str> = op.split(['(', ')']).collect();
                    let mut p = Vec::new();
                    if let Some(params) = args.get(1) {
                        let params = params.split(',').collect::<Vec<&str>>();
                        for s in params {
                            if let Ok(n) = s.parse::<usize>() {
                                p.push(n);
                            }
                        }
                    }
                    let mut net = if let Some(p) = p.get(0..2) {
                        Net::new(p[0], p[1])
                    } else {
                        Net::new(0, 0)
                    };
                    for hole in holes {
                        if let Ok(wh) = white_hole_query.get(*hole) {
                            if wh.link_types == (0, 1) {
                                let input = access.net_query.get(wh.bh_parent).unwrap().0.clone();
                                if input.inputs() == net.inputs()
                                    && input.outputs() == net.outputs()
                                {
                                    net = input;
                                }
                            }
                        }
                    }
                    let swap = Net::wrap(Box::new(SwapUnit::new(net, r.clone())));
                    access.net_query.get_mut(*id).unwrap().0 = swap;
                    lt_to_open = Some(0);
                    // store
                    commands.entity(*id).insert(NetChannel(s, r));
                }
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (0, 1) && wh.open {
                            let input = access.net_query.get(wh.bh_parent).unwrap().0.clone();
                            let net = &access.net_query.get(*id).unwrap().0;
                            if let Ok(NetChannel(s, _)) = access.net_chan_query.get(*id) {
                                if input.inputs() == net.inputs()
                                    && input.outputs() == net.outputs()
                                {
                                    let _ = s.try_send(input);
                                }
                            }
                        }
                    }
                }
            }
            // kr() | reset() | sr()
            68..=70 => {
                let op_changed = access.op_changed_query.get(*id).unwrap().0;
                let lost = access.lost_wh_query.get(*id).unwrap().0;
                let num_changed = access.num_query.get_mut(*id).unwrap().is_changed();
                let mut changed = false;
                let mut input = None;
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (0, 1) {
                            input = Some(wh.bh_parent);
                        }
                        if wh.open {
                            changed = true;
                        }
                    }
                }
                if changed || lost || op_changed || num_changed {
                    if let Some(input) = input {
                        let net = access.net_query.get(input).unwrap().0.clone();
                        let n = access.num_query.get(*id).unwrap().0;
                        let output = &mut access.net_query.get_mut(*id).unwrap().0;
                        if op_num == 68 && net.inputs() == 0 && net.outputs() == 1 {
                            // kr()
                            *output = Net::wrap(Box::new(An(Kr::new(net, n.max(1.) as usize))));
                        } else if op_num == 69 && net.inputs() == 0 && net.outputs() == 1 {
                            // reset()
                            *output = Net::wrap(Box::new(An(Reset::new(net, n))));
                        } else if op_num == 70 {
                            // sr()
                            *output = net;
                            output.set_sample_rate(n as f64);
                        }
                        lt_to_open = Some(0);
                    } else {
                        access.net_query.get_mut(*id).unwrap().0 = Net::new(0, 0);
                    }
                }
            }
            // trig_reset() | reset_v()
            71 | 72 => {
                let op_changed = access.op_changed_query.get(*id).unwrap().0;
                let lost = access.lost_wh_query.get(*id).unwrap().0;
                let mut changed = false;
                let mut input = None;
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (0, 1) {
                            input = Some(wh.bh_parent);
                        }
                        if wh.open {
                            changed = true;
                        }
                    }
                }
                if changed || lost || op_changed {
                    if let Some(input) = input {
                        let net = access.net_query.get(input).unwrap().0.clone();
                        if net.inputs() == 0 && net.outputs() == 1 {
                            let output = &mut access.net_query.get_mut(*id).unwrap().0;
                            if op_num == 71 {
                                *output = Net::wrap(Box::new(An(TrigReset::new(net))));
                            } else {
                                *output = Net::wrap(Box::new(An(ResetV::new(net))));
                            }
                            lt_to_open = Some(0);
                        }
                    } else {
                        access.net_query.get_mut(*id).unwrap().0 = Net::new(0, 0);
                    }
                }
            }
            // seq() | select()
            73 | 74 => {
                let op_changed = access.op_changed_query.get(*id).unwrap().0;
                let lost = access.lost_wh_query.get(*id).unwrap().0;
                let mut changed = false;
                let mut inputs = Vec::new();
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types.0 == 0 {
                            let index = Ord::max(wh.link_types.1, 0) as usize;
                            if index >= inputs.len() {
                                inputs.resize(index + 1, None);
                            }
                            inputs[index] = Some(wh.bh_parent);
                        }
                        if wh.open {
                            changed = true;
                        }
                    }
                }
                if changed || lost || op_changed {
                    let mut nets = Vec::new();
                    for i in inputs.iter().flatten() {
                        let net = &access.net_query.get(*i).unwrap().0;
                        if net.inputs() == 0 && net.outputs() == 1 {
                            nets.push(net.clone());
                        }
                    }
                    let n = &mut access.net_query.get_mut(*id).unwrap().0;
                    if op_num == 73 {
                        *n = Net::wrap(Box::new(An(Seq::new(nets))));
                    } else {
                        *n = Net::wrap(Box::new(An(Select::new(nets))));
                    }
                    lt_to_open = Some(0);
                }
            }
            // wave()
            75 => {
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-13, 1) && wh.open {
                            let arr = &access.arr_query.get(wh.bh_parent).unwrap().0;
                            let net = &mut access.net_query.get_mut(*id).unwrap().0;
                            *net = Net::wrap(Box::new(wavech(
                                &std::sync::Arc::new(Wave::from_samples(44100., arr)),
                                0,
                                Some(0),
                            )));
                            lt_to_open = Some(0);
                        }
                    }
                }
            }
            // branch() | bus() | pipe() | stack() | sum() | product()
            76..=81 => {
                let op_changed = access.op_changed_query.get(*id).unwrap().0;
                let lost = access.lost_wh_query.get(*id).unwrap().0;
                let mut changed = false;
                let mut arr = None;
                let mut op_str = None;
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (-13, 1) {
                            arr = Some(wh.bh_parent);
                        }
                        if wh.link_types == (0, 2) {
                            op_str = Some(wh.bh_parent);
                        }
                        if wh.open {
                            changed = true;
                        }
                    }
                }
                if changed || lost || op_changed {
                    let mut graph = Net::new(0, 0);
                    let mut empty = true;
                    if let (Some(arr), Some(op_str)) = (arr, op_str) {
                        let arr = &access.arr_query.get(arr).unwrap().0;
                        let op_str = &access.op_query.get(op_str).unwrap().0;
                        for i in arr {
                            let net = str_to_net(&op_str.replace('#', &format!("{}", i)));
                            if empty {
                                graph = net;
                                empty = false;
                            } else {
                                let (gi, go) = (graph.inputs(), graph.outputs());
                                let (ni, no) = (net.inputs(), net.outputs());
                                match op_num {
                                    76 if gi == ni => graph = graph ^ net,
                                    77 if gi == ni && go == no => graph = graph & net,
                                    78 if go == ni => graph = graph >> net,
                                    79 => graph = graph | net,
                                    80 if go == no => graph = graph + net,
                                    81 if go == no => graph = graph * net,
                                    _ => {}
                                }
                            }
                        }
                    }
                    access.net_query.get_mut(*id).unwrap().0 = graph;
                    lt_to_open = Some(0);
                }
            }
            // "SUM" | "+" | "PRO" | "*"
            82 | 83 => {
                let op_changed = access.op_changed_query.get(*id).unwrap().0;
                let lost = access.lost_wh_query.get(*id).unwrap().0;
                let num_changed = access.num_query.get_mut(*id).unwrap().is_changed();
                let mut changed = false;
                let mut inputs = Vec::new();
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types.0 == 0 {
                            // order matters because if inputs have inputs of their own
                            // these inputs will be stacked based on this order
                            let index = Ord::max(wh.link_types.1, 0) as usize;
                            if index >= inputs.len() {
                                inputs.resize(index + 1, None);
                            }
                            inputs[index] = Some(wh.bh_parent);
                        }
                        if wh.open {
                            changed = true;
                        }
                    }
                }
                if changed || lost || op_changed || num_changed {
                    let mut graph = Net::new(0, 0);
                    let mut empty = true;
                    let n = access.num_query.get(*id).unwrap().0.max(1.) as i32;
                    for _ in 0..n {
                        for i in inputs.iter().flatten() {
                            let net = access.net_query.get(*i).unwrap().0.clone();
                            if empty {
                                graph = net;
                                empty = false;
                            } else if graph.outputs() == net.outputs() {
                                if graph.size() >= access.node_limit.0 {
                                    continue;
                                }
                                if op_num == 82 {
                                    graph = graph + net;
                                } else {
                                    graph = graph * net;
                                }
                            }
                        }
                    }
                    access.net_query.get_mut(*id).unwrap().0 = graph;
                    lt_to_open = Some(0);
                }
            }
            // "-" | "SUB"
            84 => {
                let op_changed = access.op_changed_query.get(*id).unwrap().0;
                let lost = access.lost_wh_query.get(*id).unwrap().0;
                let mut changed = false;
                let mut lhs = None;
                let mut rhs = None;
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (0, 1) {
                            lhs = Some(wh.bh_parent);
                        }
                        if wh.link_types == (0, 2) {
                            rhs = Some(wh.bh_parent);
                        }
                        if wh.open {
                            changed = true;
                        }
                    }
                }
                if changed || lost || op_changed {
                    if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                        let lhs = access.net_query.get(lhs).unwrap().0.clone();
                        let rhs = access.net_query.get(rhs).unwrap().0.clone();
                        if lhs.outputs() == rhs.outputs() {
                            let graph = lhs - rhs;
                            if graph.size() < access.node_limit.0 {
                                access.net_query.get_mut(*id).unwrap().0 = graph;
                            }
                        }
                    }
                    lt_to_open = Some(0);
                }
            }
            // ">>" | "|" | "&" | "^" | "PIP" | "STA" | "BUS" | "BRA"
            85..=88 => {
                let op_changed = access.op_changed_query.get(*id).unwrap().0;
                let lost = access.lost_wh_query.get(*id).unwrap().0;
                let num_changed = access.num_query.get_mut(*id).unwrap().is_changed();
                let mut changed = false;
                let mut inputs = Vec::new();
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types.0 == 0 {
                            let index = Ord::max(wh.link_types.1, 0) as usize;
                            if index >= inputs.len() {
                                inputs.resize(index + 1, None);
                            }
                            inputs[index] = Some(wh.bh_parent);
                        }
                        if wh.open {
                            changed = true;
                        }
                    }
                }
                if changed || lost || op_changed || num_changed {
                    let mut graph = Net::new(0, 0);
                    let mut empty = true;
                    let n = access.num_query.get(*id).unwrap().0.max(1.) as i32;
                    for _ in 0..n {
                        for i in inputs.iter().flatten() {
                            let net = access.net_query.get(*i).unwrap().0.clone();
                            if empty {
                                graph = net;
                                empty = false;
                            } else {
                                if graph.size() >= access.node_limit.0 {
                                    continue;
                                }
                                let (gi, go) = (graph.inputs(), graph.outputs());
                                let (ni, no) = (net.inputs(), net.outputs());
                                match op_num {
                                    85 if go == ni => graph = graph >> net,
                                    86 => graph = graph | net,
                                    87 if gi == ni && go == no => graph = graph & net,
                                    88 if gi == ni => graph = graph ^ net,
                                    _ => {}
                                }
                            }
                        }
                    }
                    access.net_query.get_mut(*id).unwrap().0 = graph;
                    lt_to_open = Some(0);
                }
            }
            // "!" | "THR"
            89 => {
                let op_changed = access.op_changed_query.get(*id).unwrap().0;
                let lost = access.lost_wh_query.get(*id).unwrap().0;
                let mut input = None;
                let mut changed = false;
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (0, 1) {
                            input = Some(wh.bh_parent);
                        }
                        if wh.open {
                            changed = true;
                        }
                    }
                }
                if changed || lost || op_changed {
                    let mut graph = Net::new(0, 0);
                    if let Some(input) = input {
                        graph = access.net_query.get(input).unwrap().0.clone();
                    }
                    access.net_query.get_mut(*id).unwrap().0 = !graph;
                    lt_to_open = Some(0);
                }
            }
            // out() | dac()
            90 => {
                let op_changed = access.op_changed_query.get(*id).unwrap().0;
                let lost = access.lost_wh_query.get(*id).unwrap().0;
                let mut input = None;
                let mut changed = false;
                for hole in holes {
                    if let Ok(wh) = white_hole_query.get(*hole) {
                        if wh.link_types == (0, 1) {
                            input = Some(wh.bh_parent);
                        }
                        if wh.open {
                            changed = true;
                        }
                    }
                }
                if changed || lost || op_changed {
                    if let Some(input) = input {
                        let net = access.net_query.get(input).unwrap().0.clone();
                        if net.inputs() == 0 && net.outputs() == 1 {
                            slot.0.set(Fade::Smooth, 0.01, Box::new(net | dc(0.)));
                        } else if net.inputs() == 0 && net.outputs() == 2 {
                            slot.0.set(Fade::Smooth, 0.01, Box::new(net));
                        } else {
                            slot.0.set(Fade::Smooth, 0.01, Box::new(dc(0.) | dc(0.)));
                        }
                    } else {
                        slot.0.set(Fade::Smooth, 0.01, Box::new(dc(0.) | dc(0.)));
                    }
                }
            }
            _ => {}
        }
        // open all white holes reading whatever changed
        if let Some(lt) = lt_to_open {
            for hole in holes {
                if let Ok(bh) = black_hole_query.get(*hole) {
                    if let Ok(wh) = white_hole_query.get_mut(bh.wh) {
                        if wh.link_types.0 == lt {
                            white_hole_query.get_mut(bh.wh).unwrap().open = true;
                        }
                    }
                }
            }
        }
        access.op_changed_query.get_mut(*id).unwrap().0 = false;
        access.lost_wh_query.get_mut(*id).unwrap().0 = false;
        // close the white holes we just read
        for hole in holes {
            if let Ok(mut wh) = white_hole_query.get_mut(*hole) {
                wh.open = false;
            }
        }
    }
}
