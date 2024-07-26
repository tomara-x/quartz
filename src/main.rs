#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::{
    asset::ron::Deserializer,
    color::Hsla,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*,
    render::view::RenderLayers,
    scene::{serde::SceneDeserializer, SceneInstance},
    sprite::Mesh2dHandle,
    tasks::IoTaskPool,
    utils::Duration,
    window::{FileDragAndDrop::DroppedFile, WindowMode},
    winit::{UpdateMode, WinitSettings},
};

use bevy_pancam::{PanCam, PanCamPlugin};
use copypasta::{ClipboardContext, ClipboardProvider};
use serde::de::DeserializeSeed;
use std::{fs::File, io::Write};

#[cfg(feature = "inspector")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod audio;
mod circles;
mod commands;
mod components;
mod connections;
mod cursor;
mod functions;
mod nodes;
mod osc;
mod process;
use {
    audio::*, circles::*, commands::*, components::*, connections::*, cursor::*, functions::*,
    osc::*, process::*,
};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            //transparent: true,
            title: String::from("awawawa"),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(PanCamPlugin)
    // osc
    .insert_resource(OscSender { host: "127.0.0.1".to_string(), port: 1729 })
    .insert_resource(OscReceiver { socket: None })
    // settings
    .insert_resource(WinitSettings {
        focused_mode: UpdateMode::reactive_low_power(Duration::from_secs_f64(1.0 / 60.0)),
        unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs_f64(1.0 / 30.0)),
    })
    .insert_resource(ClearColor(Color::hsla(0., 0., 0., 1.)))
    .insert_resource(DefaultDrawColor(Hsla::new(270., 1., 0.5, 1.)))
    .insert_resource(DefaultDrawVerts(4))
    .insert_resource(HighlightColor(Hsla::new(0.0, 1.0, 0.5, 1.)))
    .insert_resource(ConnectionColor(Hsla::new(0., 1., 1., 0.7)))
    .insert_resource(ConnectionWidth(4.))
    .insert_resource(CommandColor(Hsla::new(0., 0., 0.7, 1.)))
    .insert_resource(IndicatorColor(Hsla::new(0., 1., 0.5, 0.3)))
    .insert_resource(TextSize(0.1))
    .insert_resource(NodeLimit(500))
    .insert_resource(Version(format!("{} {}", env!("CARGO_PKG_VERSION"), env!("COMMIT_HASH"))))
    .insert_resource(Msaa::Sample4)
    // audio
    .add_systems(Startup, default_out_device)
    .add_systems(Update, set_out_device)
    .add_systems(Startup, default_in_device)
    .add_systems(Update, set_in_device)
    // main
    .insert_resource(SystemClipboard(ClipboardContext::new().unwrap()))
    .insert_resource(PasteChannel(crossbeam_channel::bounded::<String>(1)))
    .add_systems(Startup, setup)
    .add_systems(Update, toggle_pan)
    .add_systems(Update, toggle_fullscreen)
    .add_systems(Update, save_scene)
    .add_systems(Update, copy_scene.run_if(on_event::<CopyCommand>()))
    .add_systems(Update, paste_scene)
    .add_systems(Update, post_load)
    .add_systems(Update, file_drag_and_drop)
    .add_systems(Update, update_indicator)
    .init_state::<Mode>()
    // cursor
    .insert_resource(CursorInfo::default())
    .add_systems(Update, update_cursor_info)
    // circles
    .insert_resource(ClickedOnSpace(true))
    .insert_resource(ShowInfoText(true, false))
    .init_resource::<PolygonHandles>()
    .init_resource::<DragModes>()
    .add_systems(Update, spawn_circles.run_if(in_state(Mode::Draw)))
    .add_systems(Update, update_selection.after(update_cursor_info).run_if(in_state(Mode::Edit)))
    .add_systems(Update, move_selected.after(update_selection).run_if(in_state(Mode::Edit)))
    .add_systems(Update, update_color.after(update_selection).run_if(in_state(Mode::Edit)))
    .add_systems(Update, update_mat)
    .add_systems(Update, update_radius.after(update_selection).run_if(in_state(Mode::Edit)))
    .add_systems(Update, update_vertices.after(update_selection).run_if(in_state(Mode::Edit)))
    .add_systems(Update, update_mesh.after(update_vertices).after(command_parser))
    .add_systems(Update, update_num.after(update_selection).run_if(in_state(Mode::Edit)))
    .add_systems(Update, highlight_selected.after(delete_selected))
    .add_systems(Update, open_after_drag.run_if(in_state(Mode::Edit)))
    .add_systems(PreUpdate, transform_highlights)
    .add_systems(Update, rotate_selected.after(update_selection).run_if(in_state(Mode::Edit)))
    .add_systems(Update, delete_selected.run_if(on_event::<DeleteCommand>()))
    .add_systems(PreUpdate, update_info_text)
    .add_systems(Update, spawn_info_text)
    // events
    .add_event::<SaveCommand>()
    .add_event::<CopyCommand>()
    .add_event::<DeleteCommand>()
    .add_event::<ConnectCommand>()
    .add_event::<OutDeviceCommand>()
    .add_event::<InDeviceCommand>()
    // connections
    .insert_resource(DefaultLT((0, 0)))
    .add_systems(Update, connect.run_if(in_state(Mode::Connect)))
    .add_systems(Update, connect_targets)
    .add_systems(Update, target.run_if(in_state(Mode::Connect)))
    .add_systems(PreUpdate, update_connection_arrows)
    // process
    .init_resource::<Queue>()
    .init_resource::<LoopQueue>()
    .add_event::<OrderChange>()
    .add_systems(PostUpdate, sort_by_order.before(process).run_if(on_event::<OrderChange>()))
    .add_systems(PostUpdate, prepare_loop_queue.after(sort_by_order).before(process))
    .add_systems(PostUpdate, process)
    // commands
    .add_systems(Update, command_parser)
    // type registry
    .register_type::<DragModes>()
    .register_type::<Queue>()
    .register_type::<Col>()
    .register_type::<Op>()
    .register_type::<Number>()
    .register_type::<Arr>()
    .register_type::<Selected>()
    .register_type::<Save>()
    .register_type::<Order>()
    .register_type::<BlackHole>()
    .register_type::<WhiteHole>()
    .register_type::<Vertices>()
    .register_type::<Targets>()
    .register_type::<LostWH>()
    .register_type::<DefaultDrawColor>()
    .register_type::<DefaultDrawVerts>()
    .register_type::<HighlightColor>()
    .register_type::<ConnectionColor>()
    .register_type::<ConnectionWidth>()
    .register_type::<CommandColor>()
    .register_type::<IndicatorColor>()
    .register_type::<TextSize>()
    .register_type::<Version>()
    .register_type::<Holes>()
    .register_type::<NodeLimit>()
    .register_type::<ShowInfoText>();

    #[cfg(feature = "inspector")]
    app.add_plugins(WorldInspectorPlugin::new());

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    command_color: Res<CommandColor>,
    connection_color: Res<ConnectionColor>,
    indicator_color: Res<IndicatorColor>,
) {
    // camera
    commands.spawn((
        Camera2dBundle {
            camera: Camera { hdr: true, ..default() },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_translation(Vec3::Z * 200.),
            ..default()
        },
        BloomSettings {
            intensity: 0.5,
            low_frequency_boost: 0.6,
            low_frequency_boost_curvature: 0.4,
            composite_mode: BloomCompositeMode::Additive,
            ..default()
        },
        PanCam { enabled: false, max_scale: Some(80.), min_scale: 0.005, ..default() },
        RenderLayers::from_layers(&[0, 1, 2, 3, 4]),
    ));

    // command line
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle { font_size: 13.0, color: command_color.0.into(), ..default() },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(5.)),
                    align_self: AlignSelf::End,
                    ..default()
                }),
                CommandText,
            ));
        });

    // selecting / drawing / connecting indicator
    let id = commands
        .spawn((
            ColorMesh2dBundle {
                mesh: meshes.add(Triangle2d::default()).into(),
                material: materials.add(ColorMaterial::from_color(indicator_color.0)),
                transform: Transform::from_translation(Vec3::Z),
                ..default()
            },
            Col(indicator_color.0),
        ))
        .id();
    commands.insert_resource(Indicator(id));

    // arrow mesh
    commands.insert_resource(ArrowHandle(meshes.add(Triangle2d::default()).into()));

    // connection material
    commands.insert_resource(ConnectionMat(
        materials.add(ColorMaterial::from_color(connection_color.0)),
    ));
}

fn toggle_pan(mut query: Query<&mut PanCam>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        query.single_mut().enabled = true;
    } else if keyboard_input.just_released(KeyCode::Space) {
        query.single_mut().enabled = false;
    }
}

fn toggle_fullscreen(mut query: Query<&mut Window>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::F11) {
        if query.single().mode == WindowMode::Fullscreen {
            query.single_mut().mode = WindowMode::Windowed;
        } else {
            query.single_mut().mode = WindowMode::Fullscreen;
        }
    }
}

fn update_indicator(
    mode: Res<State<Mode>>,
    id: Res<Indicator>,
    mut trans_query: Query<&mut Transform>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    connection_width: Res<ConnectionWidth>,
    default_verts: Res<DefaultDrawVerts>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
    clicked_on_space: Res<ClickedOnSpace>,
) {
    if mouse_button_input.pressed(MouseButton::Left)
        && !mouse_button_input.just_pressed(MouseButton::Left)
        && !keyboard_input.pressed(KeyCode::Space)
    {
        if *mode.get() == Mode::Edit && clicked_on_space.0 {
            let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
            let mesh = meshes.get_mut(mesh_id).unwrap();
            *mesh = Rectangle::default().into();
            *trans_query.get_mut(id.0).unwrap() = Transform {
                translation: ((cursor.i + cursor.f) / 2.).extend(400.),
                scale: (cursor.f - cursor.i).abs().extend(1.),
                ..default()
            };
        } else if *mode.get() == Mode::Draw {
            let v = default_verts.0;
            let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
            let mesh = meshes.get_mut(mesh_id).unwrap();
            *mesh = RegularPolygon::new(1., v).into();
            let dist = cursor.i.distance(cursor.f);
            *trans_query.get_mut(id.0).unwrap() = Transform {
                translation: cursor.i.extend(400.),
                scale: Vec3::new(dist, dist, 1.),
                ..default()
            };
        } else if *mode.get() == Mode::Connect {
            let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
            let mesh = meshes.get_mut(mesh_id).unwrap();
            *mesh = Triangle2d::default().into();
            let perp = (cursor.i - cursor.f).perp();
            *trans_query.get_mut(id.0).unwrap() = Transform {
                translation: ((cursor.i + cursor.f) / 2.).extend(400.),
                scale: Vec3::new(connection_width.0, cursor.f.distance(cursor.i), 1.),
                rotation: Quat::from_rotation_z(perp.to_angle()),
            }
        }
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        *trans_query.get_mut(id.0).unwrap() = Transform::default();
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = Triangle2d::default().into();
    }
}

fn save_scene(world: &mut World) {
    let mut save_events = world.resource_mut::<Events<SaveCommand>>();
    let events: Vec<SaveCommand> = save_events.drain().collect();
    for event in events {
        let name = event.0;
        let mut query = world.query_filtered::<Entity, With<Vertices>>();
        let scene = DynamicSceneBuilder::from_world(world)
            .allow::<Col>()
            .allow::<Transform>()
            .allow::<Op>()
            .allow::<Number>()
            .allow::<Arr>()
            .allow::<Order>()
            .allow::<BlackHole>()
            .allow::<WhiteHole>()
            .allow::<Holes>()
            .allow::<Vertices>()
            .allow::<Targets>()
            .allow_resource::<DefaultDrawColor>()
            .allow_resource::<DefaultDrawVerts>()
            .allow_resource::<HighlightColor>()
            .allow_resource::<ConnectionColor>()
            .allow_resource::<ConnectionWidth>()
            .allow_resource::<ClearColor>()
            .allow_resource::<CommandColor>()
            .allow_resource::<IndicatorColor>()
            .allow_resource::<TextSize>()
            .allow_resource::<Version>()
            .allow_resource::<NodeLimit>()
            .allow_resource::<ShowInfoText>()
            .extract_entities(query.iter(world))
            .extract_resources()
            .build();
        let type_registry = world.resource::<AppTypeRegistry>();
        let type_registry = type_registry.read();
        let serialized_scene = scene.serialize(&type_registry).unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        IoTaskPool::get()
            .spawn(async move {
                File::create(format!("assets/{}", name))
                    .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                    .expect("Error while writing scene to file");
            })
            .detach();
    }
}

fn copy_scene(world: &mut World) {
    let mut query = world.query_filtered::<Entity, With<Selected>>();
    let scene = DynamicSceneBuilder::from_world(world)
        .allow::<Col>()
        .allow::<Transform>()
        .allow::<Op>()
        .allow::<Number>()
        .allow::<Arr>()
        .allow::<Order>()
        .allow::<BlackHole>()
        .allow::<WhiteHole>()
        .allow::<Holes>()
        .allow::<Vertices>()
        .allow::<Targets>()
        .extract_entities(query.iter(world))
        .build();
    let serialized_scene = scene.serialize(&world.resource::<AppTypeRegistry>().read()).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut ctx = world.resource_mut::<SystemClipboard>();
        ctx.0.set_contents(serialized_scene).unwrap();
    }
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        if let Some(clipboard) = window.navigator().clipboard() {
            let _ = clipboard.write_text(&serialized_scene);
        }
    }
}

fn paste_scene(world: &mut World) {
    if let Ok(string) = world.resource::<PasteChannel>().0 .1.try_recv() {
        let bytes = string.into_bytes();
        let mut scene = None;
        if let Ok(mut deserializer) = Deserializer::from_bytes(&bytes) {
            let type_registry = world.resource::<AppTypeRegistry>();
            let scene_deserializer = SceneDeserializer { type_registry: &type_registry.read() };
            if let Ok(s) = scene_deserializer.deserialize(&mut deserializer) {
                scene = Some(s);
            }
        }
        if let Some(s) = scene {
            let scene = world.resource_mut::<Assets<DynamicScene>>().add(s);
            world.spawn(DynamicSceneBundle { scene, ..default() });
        }
    }
}

fn file_drag_and_drop(
    mut commands: Commands,
    mut events: EventReader<FileDragAndDrop>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        if let DroppedFile { path_buf, .. } = event {
            commands.spawn(DynamicSceneBundle {
                scene: asset_server.load(path_buf.clone()),
                ..default()
            });
        }
    }
}

fn post_load(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    main_query: Query<(&Transform, &Col, &Vertices)>,
    mut order_change: EventWriter<OrderChange>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<&BlackHole>,
    scenes: Query<(Entity, &SceneInstance)>,
    children_query: Query<&Children>,
    mut holes_query: Query<&mut Holes>,
    mut op_query: Query<&mut Op>,
    mut command_line_text: Query<&mut Text, With<CommandText>>,
    scene_spawner: Res<SceneSpawner>,
    mut polygon_handles: ResMut<PolygonHandles>,
    mut indicator_color_query: Query<&mut Col, Without<Vertices>>,
    (
        command_color,
        connection_color,
        arrow_handle,
        connection_mat,
        selected_query,
        indicator_color,
        indicator_id,
    ): (
        Res<CommandColor>,
        Res<ConnectionColor>,
        Res<ArrowHandle>,
        Res<ConnectionMat>,
        Query<Entity, With<Selected>>,
        Res<IndicatorColor>,
        Res<Indicator>,
    ),
) {
    for (scene_id, instance_id) in scenes.iter() {
        if scene_spawner.instance_is_ready(**instance_id) {
            for e in selected_query.iter() {
                commands.entity(e).remove::<Selected>();
            }
            // update indicator color
            indicator_color_query.get_mut(indicator_id.0).unwrap().0 = indicator_color.0;
            // update connection material from color resource
            materials.get_mut(&connection_mat.0).unwrap().color = connection_color.0.into();
            if let Ok(children) = children_query.get(scene_id) {
                for child in children {
                    if let Ok((t, c, v)) = main_query.get(*child) {
                        if polygon_handles.0.len() <= v.0 {
                            polygon_handles.0.resize(v.0 + 1, None);
                        }
                        if polygon_handles.0[v.0].is_none() {
                            let handle = meshes.add(RegularPolygon::new(1., v.0)).into();
                            polygon_handles.0[v.0] = Some(handle);
                        }
                        commands.entity(*child).try_insert((
                            ColorMesh2dBundle {
                                mesh: polygon_handles.0[v.0].clone().unwrap(),
                                material: materials.add(ColorMaterial::from_color(c.0)),
                                transform: *t,
                                ..default()
                            },
                            Selected,
                        ));
                        if let Ok(op) = op_query.get_mut(*child) {
                            commands.entity(*child).insert((
                                OpNum(str_to_op_num(&op.0)),
                                Network(str_to_net(&op.0)),
                                NetIns(Vec::new()),
                                OpChanged(true),
                                LostWH(false),
                                RenderLayers::layer(1),
                            ));
                            let holes = &mut holes_query.get_mut(*child).unwrap().0;
                            let mut new_holes = Vec::new();
                            for hole in &mut *holes {
                                if let Ok(mut wh) = white_hole_query.get_mut(*hole) {
                                    if black_hole_query.contains(wh.bh)
                                        && main_query.contains(wh.bh_parent)
                                    {
                                        wh.open = true;
                                        let arrow = commands
                                            .spawn((
                                                ColorMesh2dBundle {
                                                    mesh: arrow_handle.0.clone(),
                                                    material: connection_mat.0.clone(),
                                                    transform: Transform::default(),
                                                    ..default()
                                                },
                                                RenderLayers::layer(4),
                                            ))
                                            .id();
                                        commands.entity(*hole).insert((
                                            ConnectionArrow(arrow),
                                            RenderLayers::layer(3),
                                        ));
                                        new_holes.push(*hole);
                                        commands.entity(*hole).remove_parent();
                                    }
                                } else if let Ok(bh) = black_hole_query.get(*hole) {
                                    if white_hole_query.contains(bh.wh)
                                        && main_query.contains(bh.wh_parent)
                                    {
                                        commands.entity(*hole).insert(RenderLayers::layer(2));
                                        new_holes.push(*hole);
                                        commands.entity(*hole).remove_parent();
                                    }
                                }
                            }
                            *holes = new_holes;
                            commands.entity(*child).remove_parent();
                        }
                    }
                }
                order_change.send_default();
            }
            // update the command line color from resource
            let clt = &mut command_line_text.single_mut();
            clt.sections[0].style.color = command_color.0.into();
            // despawn the now empty instance (may contain bad state connections when pasting)
            commands.entity(scene_id).remove::<SceneInstance>();
            commands.entity(scene_id).despawn_recursive();
        }
    }
}
