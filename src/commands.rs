use bevy::{
    prelude::*,
    ecs::system::SystemParam,
    render::view::RenderLayers,
    app::AppExit,
    input::keyboard::{KeyboardInput, Key},
    window::WindowMode,
};

use crate::components::*;

use fundsp::audiounit::AudioUnit32;

#[derive(SystemParam)]
pub struct Access<'w, 's> {
    op_query: Query<'w, 's, &'static mut Op>,
    num_query: Query<'w, 's, &'static mut Num>,
    radius_query: Query<'w, 's, &'static mut Radius>,
    col_query: Query<'w, 's, &'static mut Col>,
    trans_query: Query<'w, 's, &'static mut Transform>,
    arr_query: Query<'w, 's, &'static mut Arr>,
    order_query: Query<'w, 's, &'static mut Order>,
    selected_query: Query<'w, 's, Entity, With<Selected>>,
    white_hole_query: Query<'w, 's, &'static mut WhiteHole>,
    black_hole_query: Query<'w, 's, &'static mut BlackHole>,
    order_change: EventWriter<'w, OrderChange>,
    vertices_query: Query<'w, 's, &'static mut Vertices>,
    save_event: EventWriter<'w, SaveCommand>,
    targets_query: Query<'w, 's, &'static mut Targets>,
    gained_wh_query: Query<'w, 's, &'static mut GainedWH>,
    text_query: Query<'w, 's, &'static mut Text, Without<CommandText>>,
    render_layers: Query<'w, 's, &'static mut RenderLayers, With<Camera>>,
    net_query: Query<'w, 's, &'static mut Network>,
    exit_event: EventWriter<'w, AppExit>,
    drag_modes: ResMut<'w, DragModes>,
    windows: Query<'w, 's, &'static mut Window>,
}

pub fn command_parser(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut key_event: EventReader<KeyboardInput>,
    mut command_line: Query<&mut Text, With<CommandText>>,
    circles_query: Query<Entity, With<Radius>>,
    mut access: Access,
    mut mode: Local<i32>,
    mut in_progress: Local<bool>,
    mut next_state: ResMut<NextState<Mode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    info_text_query: Query<(Entity, &InfoText)>,
    mut ids_shown: Local<bool>,
    global_trans_rights: Query<&GlobalTransform>,
    parent_query: Query<&Parent>,
) {
    if key_event.is_empty()
    && !keyboard_input.just_released(KeyCode::KeyT)
    && !*in_progress { return; }

    let text = &mut command_line.single_mut().sections[0].value;

    // toggle fullscreen mode
    if keyboard_input.just_pressed(KeyCode::F11) {
        if access.windows.single().mode == WindowMode::Fullscreen {
            access.windows.single_mut().mode = WindowMode::Windowed;
        } else {
            access.windows.single_mut().mode = WindowMode::Fullscreen;
        }
    }

    // draw mode
    if *mode == 1 {
        // exit to edit
        if keyboard_input.any_just_pressed([KeyCode::Escape, KeyCode::KeyE]) {
            text.clear();
            *mode = 0;
            next_state.set(Mode::Edit);
            key_event.clear(); // we have an 'e' that we don't want
        }
        // switch to connect mode
        if keyboard_input.just_pressed(KeyCode::KeyC) {
            *text = "-- CONNECT --".to_string();
            *mode = 2;
            next_state.set(Mode::Connect);
        }
    }

    // connect mode
    if *mode == 2 {
        // exit to edit
        if keyboard_input.any_just_pressed([KeyCode::Escape, KeyCode::KeyE]) {
            text.clear();
            *mode = 0;
            next_state.set(Mode::Edit);
            key_event.clear(); // consume the 'e' when exiting to edit
        }
        // target
        if keyboard_input.just_pressed(KeyCode::KeyT) { *text = "-- TARGET --".to_string(); }
        if keyboard_input.just_released(KeyCode::KeyT) { *text = "-- CONNECT --".to_string(); }
        // switch to draw mode
        if keyboard_input.just_pressed(KeyCode::KeyD) {
            *text = "-- DRAW --".to_string();
            *mode = 1;
            next_state.set(Mode::Draw);
        }
    }

    // edit mode
    if *mode == 0 {
        for key in key_event.read() {
            if key.state.is_pressed() {
                match &key.logical_key {
                    Key::Character(c) => {
                        if let Some(c) = c.chars().nth(0) {
                            if text.chars().nth(0) == Some('>') { text.clear(); }
                            if !c.is_control() && *text != "F" { text.push(c); }
                        }
                    }
                    Key::Space => {
                        if !text.ends_with(' ') && !text.is_empty() && *text != "F" {
                            text.push(' ');
                        }
                    }
                    Key::Backspace => { text.pop(); }
                    Key::Escape => { text.clear(); }
                    // tab completion when?
                    _ => {}
                }
            }
        }
        if keyboard_input.just_pressed(KeyCode::Enter) {
            // commands starting with :
            let lines = text.as_str().split(";");
            for line in lines {
                let mut command = line.split_ascii_whitespace();
                match command.next() {
                    // open scene file
                    Some(":e") => {
                        if let Some(s) = command.next() {
                            commands.spawn(DynamicSceneBundle {
                                scene: asset_server.load(format!("{}.scn.ron", s)),
                                ..default()
                            });
                        }
                    }
                    // save scene file
                    Some(":w") => {
                        if let Some(s) = command.next() {
                            access.save_event.send(SaveCommand(s.to_string()));
                        }
                    }
                    Some(":q") => {
                        access.exit_event.send_default();
                    }
                    // white hole / black hole link type
                    // TODO(amy): set-both-ends version
                    // can be moved to :set
                    Some(":lt") | Some("lt") => {
                        if let Some(s) = command.next() {
                            if let Some(e) = str_to_id(s) {
                                if let Ok(mut wh) = access.white_hole_query.get_mut(e) {
                                    if let Some(s) = command.next() {
                                        wh.link_types.1 = str_to_lt(s);
                                        wh.open = true;
                                        let parent = parent_query.get(e).unwrap();
                                        access.gained_wh_query.get_mut(**parent).unwrap().0 = true;
                                    }
                                } else if let Ok(bh) = access.black_hole_query.get(e) {
                                    let wh = &mut access.white_hole_query.get_mut(bh.wh).unwrap();
                                    if let Some(s) = command.next() {
                                        wh.link_types.0 = str_to_lt(s);
                                        wh.open = true;
                                        let parent = parent_query.get(bh.wh).unwrap();
                                        access.gained_wh_query.get_mut(**parent).unwrap().0 = true;
                                    }
                                }
                            } else {
                                for id in access.selected_query.iter() {
                                    if let Ok(mut wh) = access.white_hole_query.get_mut(id) {
                                        wh.link_types.1 = str_to_lt(s);
                                        wh.open = true;
                                        let parent = parent_query.get(id).unwrap();
                                        access.gained_wh_query.get_mut(**parent).unwrap().0 = true;
                                    } else if let Ok(bh) = access.black_hole_query.get(id) {
                                        let wh = &mut access.white_hole_query.get_mut(bh.wh).unwrap();
                                        wh.link_types.0 = str_to_lt(s);
                                        wh.open = true;
                                        let parent = parent_query.get(id).unwrap();
                                        access.gained_wh_query.get_mut(**parent).unwrap().0 = true;
                                    }
                                }
                            }
                        }
                    }
                    // toggle open a white hole (by id)
                    Some(":ht") | Some("ht") => {
                        if let Some(s) = command.next() {
                            if let Some(e) = str_to_id(s) {
                                if let Ok(mut wh) = access.white_hole_query.get_mut(e) {
                                    wh.open = !wh.open;
                                }
                            }
                        }
                    }
                    Some(":set") | Some("set") => {
                        match command.next() {
                            Some("n") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut num) = access.num_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    num.0 = n;
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut num) = access.num_query.get_mut(id) {
                                                num.0 = n;
                                            }
                                        }
                                    }
                                }
                            }
                            Some("r") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut radius) = access.radius_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    radius.0 = n.max(0.);
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut radius) = access.radius_query.get_mut(id) {
                                                radius.0 = n.max(0.);
                                            }
                                        }
                                    }
                                }
                            }
                            Some("x") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut t) = access.trans_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    t.translation.x = n;
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut t) = access.trans_query.get_mut(id) {
                                                t.translation.x = n;
                                            }
                                        }
                                    }
                                }
                            }
                            Some("y") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut t) = access.trans_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    t.translation.y = n;
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut t) = access.trans_query.get_mut(id) {
                                                t.translation.y = n;
                                            }
                                        }
                                    }
                                }
                            }
                            Some("z") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut t) = access.trans_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    t.translation.z = n;
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut t) = access.trans_query.get_mut(id) {
                                                t.translation.z = n;
                                            }
                                        }
                                    }
                                }
                            }
                            Some("h") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut color) = access.col_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    color.0.set_h(n);
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut color) = access.col_query.get_mut(id) {
                                                color.0.set_h(n);
                                            }
                                        }
                                    }
                                }
                            }
                            Some("s") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut color) = access.col_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    color.0.set_s(n);
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut color) = access.col_query.get_mut(id) {
                                                color.0.set_s(n);
                                            }
                                        }
                                    }
                                }
                            }
                            Some("l") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut color) = access.col_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    color.0.set_l(n);
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut color) = access.col_query.get_mut(id) {
                                                color.0.set_l(n);
                                            }
                                        }
                                    }
                                }
                            }
                            Some("a") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut color) = access.col_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    color.0.set_a(n);
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut color) = access.col_query.get_mut(id) {
                                                color.0.set_a(n);
                                            }
                                        }
                                    }
                                }
                            }
                            Some("v") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut vertices) = access.vertices_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<usize>() {
                                                    vertices.0 = n.max(3);
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<usize>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut vertices) = access.vertices_query.get_mut(id) {
                                                vertices.0 = n.max(3);
                                            }
                                        }
                                    }
                                }
                            }
                            Some("o") | Some("rot") | Some("rotation") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut t) = access.trans_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    t.rotation = Quat::from_rotation_z(n);
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<f32>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut t) = access.trans_query.get_mut(id) {
                                                t.rotation = Quat::from_rotation_z(n);
                                            }
                                        }
                                    }
                                }
                            }
                            Some("op") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut op) = access.op_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                op.0 = n.to_string();
                                            }
                                        }
                                    } else {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut op) = access.op_query.get_mut(id) {
                                                op.0.clear();
                                                op.0.push_str(s);
                                                for a in command.clone() { op.0.push_str(a); }
                                            }
                                        }
                                    }
                                }
                            }
                            Some("ord") | Some("order") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut order) = access.order_query.get_mut(e) {
                                            if let Some(n) = command.next() {
                                                if let Ok(n) = n.parse::<usize>() {
                                                    order.0 = n;
                                                    access.order_change.send_default();
                                                }
                                            }
                                        }
                                    } else if let Ok(n) = s.parse::<usize>() {
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut order) = access.order_query.get_mut(id) {
                                                order.0 = n;
                                                access.order_change.send_default();
                                            }
                                        }
                                    }
                                }
                            }
                            Some("arr") | Some("array") => {
                                if let Some(s) = command.next() {
                                    if let Some(e) = str_to_id(s) {
                                        if let Ok(mut arr) = access.arr_query.get_mut(e) {
                                            arr.0.clear();
                                            for n in command {
                                                if let Ok(n) = n.parse::<f32>() {
                                                    arr.0.push(n);
                                                }
                                            }
                                        }
                                    } else {
                                        let mut tmp = Vec::new();
                                        if let Ok(n) = s.parse::<f32>() { tmp.push(n); }
                                        for n in command {
                                            if let Ok(n) = n.parse::<f32>() {
                                                tmp.push(n);
                                            }
                                        }
                                        for id in access.selected_query.iter() {
                                            if let Ok(mut arr) = access.arr_query.get_mut(id) {
                                                arr.0 = tmp.clone();
                                            }
                                        }
                                    }
                                }
                            }
                            Some("tar") | Some("targets") => {
                                let mut tmp = Vec::new();
                                for e in command {
                                    if let Some(e) = str_to_id(e) {
                                        tmp.push(e);
                                    }
                                }
                                // set the rest (crd) as the targets of first (car)
                                if access.selected_query.is_empty() {
                                    if tmp.len() != 0 {
                                        let controller = tmp.remove(0);
                                        if let Ok(mut c) = access.targets_query.get_mut(controller) {
                                            c.0 = tmp;
                                        }
                                    }
                                } else {
                                    // all selected circles get the list of entities as targets
                                    for e in access.selected_query.iter() {
                                        if let Ok(mut c) = access.targets_query.get_mut(e) {
                                            c.0 = tmp.clone();
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    // target selected
                    Some(":tsel") => {
                        if let Some(s) = command.next() {
                            if let Some(e) = str_to_id(s) {
                                access.targets_query.get_mut(e).unwrap().0.clear();
                                for selected in access.selected_query.iter() {
                                    access.targets_query.get_mut(e).unwrap().0.push(selected);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            text.clear();
        }
        // key commands
        let mut command = text.as_str().split_ascii_whitespace();
        match command.next() {
            Some("d") => {
                *mode = 1;
                next_state.set(Mode::Draw);
                *text = "-- DRAW --".to_string();
            }
            Some("c") => {
                *mode = 2;
                next_state.set(Mode::Connect);
                *text = "-- CONNECT --".to_string();
            }
            Some("et") => {
                access.drag_modes.falsify();
                access.drag_modes.t = true;
                text.clear();
            }
            Some("er") => {
                access.drag_modes.falsify();
                access.drag_modes.r = true;
                text.clear();
            }
            Some("en") => {
                access.drag_modes.falsify();
                access.drag_modes.n = true;
                text.clear();
            }
            Some("eh") => {
                access.drag_modes.falsify();
                access.drag_modes.h = true;
                text.clear();
            }
            Some("es") => {
                access.drag_modes.falsify();
                access.drag_modes.s = true;
                text.clear();
            }
            Some("el") => {
                access.drag_modes.falsify();
                access.drag_modes.l = true;
                text.clear();
            }
            Some("ea") => {
                access.drag_modes.falsify();
                access.drag_modes.a = true;
                text.clear();
            }
            Some("eo") => {
                access.drag_modes.falsify();
                access.drag_modes.o = true;
                text.clear();
            }
            Some("ev") => {
                access.drag_modes.falsify();
                access.drag_modes.v = true;
                text.clear();
            }

            Some("ee") => {
                access.drag_modes.falsify();
                text.clear();
            }

            Some("Et") => {
                access.drag_modes.t = true;
                text.clear();
            }
            Some("Er") => {
                access.drag_modes.r = true;
                text.clear();
            }
            Some("En") => {
                access.drag_modes.n = true;
                text.clear();
            }
            Some("Eh") => {
                access.drag_modes.h = true;
                text.clear();
            }
            Some("Es") => {
                access.drag_modes.s = true;
                text.clear();
            }
            Some("El") => {
                access.drag_modes.l = true;
                text.clear();
            }
            Some("Ea") => {
                access.drag_modes.a = true;
                text.clear();
            }
            Some("Eo") => {
                access.drag_modes.o = true;
                text.clear();
            }
            Some("Ev") => {
                access.drag_modes.v = true;
                text.clear();
            }
            // toggle open white holes (selected)
            Some("ht") => {
                for id in access.selected_query.iter() {
                    if let Ok(mut wh) = access.white_hole_query.get_mut(id) {
                        wh.open = !wh.open;
                    }
                }
                text.clear();
            }
            // open white hole for one frame
            Some("hf") => {
                if *in_progress {
                    for id in access.selected_query.iter() {
                        if let Ok(mut wh) = access.white_hole_query.get_mut(id) {
                            wh.open = false;
                        }
                    }
                    text.clear();
                    *in_progress = false;
                } else {
                    for id in access.selected_query.iter() {
                        if let Ok(mut wh) = access.white_hole_query.get_mut(id) {
                            wh.open = true;
                        }
                    }
                    *in_progress = true;
                }
            }
            // shortcuts
            Some("o") => {
                *text = ":set op ".to_string();
            }
            Some("l") => {
                *text = ":lt ".to_string();
            }
            // ignore
            Some("[") | Some("]") | Some("0") => {
                text.clear();
            }
            // insert info texts for selected entities
            Some("II") => {
                for e in access.selected_query.iter() {
                    if info_text_query.contains(e) { continue; }
                    let info_text = commands.spawn(
                        Text2dBundle {
                            text: Text::from_sections([
                                TextSection::new(
                                    "",
                                    TextStyle { color: Color::BLACK, ..default() },
                                ),
                                TextSection::new(
                                    "",
                                    TextStyle { color: Color::BLACK, ..default() },
                                ),
                                TextSection::new(
                                    "",
                                    TextStyle { color: Color::BLACK, ..default() },
                                ),
                                TextSection::new(
                                    "",
                                    TextStyle { color: Color::BLACK, ..default() },
                                ),
                            ]).with_justify(JustifyText::Center),
                            transform: Transform::from_translation(Vec3::ZERO),
                            ..default()
                        }
                    ).id();
                    commands.entity(e).insert(InfoText(info_text));
                }
                text.clear();
            }
            // despawn selected entities' info texts
            Some("IC") => {
                for e in access.selected_query.iter() {
                    if let Ok((_, info_text)) = info_text_query.get(e) {
                        commands.entity(info_text.0).despawn();
                        commands.entity(e).remove::<InfoText>();
                    }
                }
                text.clear();
            }
            // toggle ids in info texts
            Some("ID") => {
                if *ids_shown {
                    for (_, t) in info_text_query.iter() {
                        access.text_query.get_mut(t.0).unwrap().sections[0].value = String::new();
                    }
                } else {
                    for (e, t) in info_text_query.iter() {
                        access.text_query.get_mut(t.0).unwrap().sections[0].value = format!("{:?}\n", e);
                    }
                }
                *ids_shown = !*ids_shown;
                text.clear();
            }
            // audio node inputs / outputs number / print info
            Some("ni") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    if let Ok(n) = access.net_query.get(e) {
                        t = t + &format!("[{:?}]{}  ", e, n.0.inputs());
                    }
                }
                *text = format!(">INPUTS: {}", t);
            }
            Some("no") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    if let Ok(n) = access.net_query.get(e) {
                        t = t + &format!("[{:?}]{}  ", e, n.0.outputs());
                    }
                }
                *text = format!(">OUTPUTS: {}", t);
            }
            Some("np") => {
                for e in access.selected_query.iter() {
                    if let Ok(e) = access.op_query.get(e) { *text = format!("> {}\n", e.0); }
                    if let Ok(e) = access.net_query.get(e) { *text += &e.0.clone().display(); }
                }
            }
            // inspect commands
            Some("ii") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    t = t + &format!("{:?}  ", e);
                }
                *text = format!(">ID: {}", t);
            }
            Some("in") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    if let Ok(n) = access.num_query.get(e) {
                        t = t + &format!("[{:?}]{}  ", e, n.0);
                    }
                }
                *text = format!(">NUM: {}", t);
            }
            Some("ira") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    let ra = access.radius_query.get(e).unwrap().0;
                    t = t + &format!("[{:?}]{}  ", e, ra);
                }
                *text = format!(">RADIUS: {}", t);
            }
            Some("ix") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    let x = global_trans_rights.get(e).unwrap().translation().x;
                    t = t + &format!("[{:?}]{}  ", e, x);
                }
                *text = format!(">X: {}", t);
            }
            Some("iy") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    let y = global_trans_rights.get(e).unwrap().translation().y;
                    t = t + &format!("[{:?}]{}  ", e, y);
                }
                *text = format!(">Y: {}", t);
            }
            Some("iz") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    let z = global_trans_rights.get(e).unwrap().translation().z;
                    t = t + &format!("[{:?}]{}  ", e, z);
                }
                *text = format!(">Z: {}", t);
            }
            Some("ih") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    let h = access.col_query.get(e).unwrap().0.h();
                    t = t + &format!("[{:?}]{}  ", e, h);
                }
                *text = format!(">HUE: {}", t);
            }
            Some("is") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    let s = access.col_query.get(e).unwrap().0.s();
                    t = t + &format!("[{:?}]{}  ", e, s);
                }
                *text = format!(">SATURATION: {}", t);
            }
            Some("il") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    let l = access.col_query.get(e).unwrap().0.l();
                    t = t + &format!("[{:?}]{}  ", e, l);
                }
                *text = format!(">LIGHTNESS: {}", t);
            }
            Some("ial") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    let a = access.col_query.get(e).unwrap().0.a();
                    t = t + &format!("[{:?}]{}  ", e, a);
                }
                *text = format!(">ALPHA: {}", t);
            }
            Some("iv") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    let v = access.vertices_query.get(e).unwrap().0;
                    t = t + &format!("[{:?}]{}  ", e, v);
                }
                *text = format!(">VERTICES: {}", t);
            }
            Some("iro") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    let ro = access.trans_query.get(e).unwrap().rotation.to_euler(EulerRot::XYZ).2;
                    t = t + &format!("[{:?}]{}  ", e, ro);
                }
                *text = format!(">ROTATION: {}", t);
            }
            Some("ior") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    if let Ok(or) = access.order_query.get(e) {
                        t = t + &format!("[{:?}]{}  ", e, or.0);
                    }
                }
                *text = format!(">ORDER: {}", t);
            }
            Some("iop") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    if let Ok(op) = &access.op_query.get(e) {
                        t = t + &format!("[{:?}]{}  ", e, op.0);
                    }
                }
                *text = format!(">OP: {}", t);
            }
            Some("iar") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    if let Ok(arr) = access.arr_query.get(e) {
                        t = t + &format!("[{:?}]{:?}  ", e, arr.0);
                    }
                }
                *text = format!(">ARRAY: {}", t);
            }
            Some("it") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    if let Ok(targets) = access.targets_query.get(e) {
                        t = t + &format!("[{:?}]{:?}  ", e, targets.0);
                    }
                }
                *text = format!(">TARGETS: {}", t);
            }
            Some("iL") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    if let Ok(wh) = access.white_hole_query.get(e) {
                        t = t + &format!("[{:?}]{}  ", e, lt_to_string(wh.link_types.1));
                    }
                    if let Ok(bh) = access.black_hole_query.get(e) {
                        let wh = access.white_hole_query.get(bh.wh).unwrap();
                        t = t + &format!("[{:?}]{}  ", e, lt_to_string(wh.link_types.0));
                    }
                }
                *text = format!(">LINK TYPE: {}", t);
            }
            Some("iO") => {
                let mut t = String::new();
                for e in access.selected_query.iter() {
                    if let Ok(wh) = access.white_hole_query.get(e) {
                        t = t + &format!("[{:?}]{}  ", e, wh.open);
                    }
                }
                *text = format!(">WH_OPEN: {}", t);
            }
            Some("sa") => {
                for e in circles_query.iter() {
                    commands.entity(e).insert(Selected);
                }
                text.clear();
            }
            Some("sc") => {
                for e in circles_query.iter() {
                    if access.order_query.contains(e) {
                        commands.entity(e).insert(Selected);
                    }
                }
                text.clear();
            }
            Some("sh") => {
                for e in circles_query.iter() {
                    if !access.order_query.contains(e) {
                        commands.entity(e).insert(Selected);
                    }
                }
                text.clear();
            }
            // render layers
            Some("vv") => {
                *access.render_layers.single_mut() = RenderLayers::all();
                text.clear();
            }
            Some("vc") => {
                let mut rl = access.render_layers.single_mut();
                if rl.intersects(&RenderLayers::layer(1)) {
                    *rl = rl.without(1);
                } else {
                    *rl = rl.with(1);
                }
                text.clear();
            }
            Some("vb") => {
                let mut rl = access.render_layers.single_mut();
                if rl.intersects(&RenderLayers::layer(2)) {
                    *rl = rl.without(2);
                } else {
                    *rl = rl.with(2);
                }
                text.clear();
            }
            Some("vw") => {
                let mut rl = access.render_layers.single_mut();
                if rl.intersects(&RenderLayers::layer(3)) {
                    *rl = rl.without(3);
                } else {
                    *rl = rl.with(3);
                }
                text.clear();
            }
            Some("va") => {
                let mut rl = access.render_layers.single_mut();
                if rl.intersects(&RenderLayers::layer(4)) {
                    *rl = rl.without(4);
                } else {
                    *rl = rl.with(4);
                }
                text.clear();
            }
            // TODO(amy): yyp
            Some("\u{71}\u{75}\u{61}\u{72}\u{74}\u{7a}") => {
                *text = String::from(">drink some water!");
            }
            _ => {}
        }
    }
}

fn str_to_id(s: &str) -> Option<Entity> {
    let mut e = s.split('v');
    if let Some(i) = e.next() {
        if let Some(g) = e.next() {
            if let Ok(index) = i.parse::<u64>() {
                if let Ok(gen) = g.parse::<u64>() {
                    let bits = gen << 32 | index;
                    return Some(Entity::from_bits(bits));
                }
            }
        }
    }
    None
}
