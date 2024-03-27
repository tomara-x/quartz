use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    utils::Duration,
    winit::{WinitSettings, UpdateMode},
    tasks::IoTaskPool,
    scene::{
        SceneInstance,
        serde::SceneDeserializer,
    },
    asset::ron::Deserializer,
    render::view::RenderLayers,
    window::FileDragAndDrop::DroppedFile,
    ecs::system::SystemParam,
    prelude::*
};

use bevy_pancam::{PanCam, PanCamPlugin};
use std::{fs::File, io::Write};
use copypasta::{ClipboardContext, ClipboardProvider};
use serde::de::DeserializeSeed;

#[cfg(feature = "inspector")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod components;
mod process;
mod cursor;
mod connections;
mod circles;
mod audio;
mod commands;
mod nodes;
mod functions;
use {components::*, process::*, cursor::*, connections::*,
     circles::*, audio::*, commands::*, functions::*};

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
    .insert_resource(WinitSettings {
        focused_mode: UpdateMode::ReactiveLowPower {
            wait: Duration::from_secs_f64(1.0 / 60.0),
        },
        unfocused_mode: UpdateMode::ReactiveLowPower {
            wait: Duration::from_secs_f64(1.0 / 30.0),
        },
    })

    .add_plugins(PanCamPlugin)

    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(DefaultDrawColor(Color::hsl(1.,1.,0.84)))
    .insert_resource(DefaultDrawVerts(4))
    .insert_resource(HighlightColor(Color::hsl(0.0,1.0,0.5)))
    .insert_resource(ConnectionColor(Color::hsla(0., 1., 1., 0.7)))
    .insert_resource(CommandColor(Color::hsla(0., 0., 0.7, 1.)))
    .insert_resource(DefaultLT((0, 0)))
    .insert_resource(SystemClipboard(ClipboardContext::new().unwrap()))
    .insert_resource(Msaa::Sample4)
    .insert_resource(Version(
        format!("{} {}", env!("CARGO_PKG_VERSION"),
            String::from_utf8(std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output().unwrap().stdout).unwrap().trim()
        )
    ))
    .init_resource::<PolygonHandles>()
    .init_resource::<DacCircles>()

    .add_systems(Startup, setup)
    .add_systems(Startup, ext_thread)

    .add_systems(Update, toggle_pan)
    .init_state::<Mode>()
    .add_systems(Update, save_scene)
    .add_systems(Update, copy_scene.run_if(on_event::<CopyCommand>()))
    .add_systems(Update, paste_scene.run_if(on_event::<PasteCommand>()))
    .add_systems(Update, post_load)
    .add_systems(Update, file_drag_and_drop)
    .init_resource::<DragModes>()
    // cursor
    .insert_resource(CursorInfo::default())
    .add_systems(Update, update_cursor_info)
    // circles
    .add_systems(Update, spawn_circles.run_if(in_state(Mode::Draw)))
    .add_systems(Update, mark_visible_circles.after(update_cursor_info))
    .add_systems(Update, draw_drawing_circle.run_if(in_state(Mode::Draw)))
    .add_systems(Update, update_selection.after(mark_visible_circles).run_if(in_state(Mode::Edit)))
    .add_systems(Update, move_selected.after(update_selection).run_if(in_state(Mode::Edit)))
    .add_systems(Update, update_color.after(update_selection).run_if(in_state(Mode::Edit)))
    .add_systems(Update, update_mat)
    .add_systems(Update, update_radius.after(update_selection).run_if(in_state(Mode::Edit)))
    .add_systems(Update, update_vertices.after(update_selection).run_if(in_state(Mode::Edit)))
    .add_systems(Update, update_mesh.after(update_vertices))
    .add_systems(Update, update_num.after(update_selection).run_if(in_state(Mode::Edit)))
    .add_systems(Update, highlight_selected.after(delete_selected))
    .add_systems(Update, open_after_drag.run_if(in_state(Mode::Edit)))
    .add_systems(PreUpdate, transform_highlights)
    .add_systems(Update, rotate_selected.after(update_selection).run_if(in_state(Mode::Edit)))
    .add_systems(Update, delete_selected.run_if(on_event::<DeleteCommand>()))
    .add_systems(PreUpdate, update_info_text)
    // events
    .add_event::<SaveCommand>()
    .add_event::<CopyCommand>()
    .add_event::<PasteCommand>()
    .add_event::<DeleteCommand>()
    .add_event::<DacChange>()
    // connections
    .add_systems(Update, connect.run_if(in_state(Mode::Connect)))
    .add_systems(Update, target.run_if(in_state(Mode::Connect)))
    .add_systems(PreUpdate, update_connection_arrows)
    .add_systems(Update, draw_connecting_arrow.run_if(in_state(Mode::Connect)))
    // order
    .init_resource::<Queue>()
    .init_resource::<LoopQueue>()
    .add_event::<OrderChange>()
    .add_systems(PostUpdate, sort_by_order.before(process).run_if(on_event::<OrderChange>()))
    .add_systems(PostUpdate, prepare_loop_queue.after(sort_by_order).before(process))
    // process
    .add_systems(PostUpdate, process)
    .add_systems(Update, update_slot.run_if(on_event::<DacChange>()))
    // commands
    .add_systems(Update, command_parser)

    // type registry
    .register_type::<DragModes>()
    .register_type::<Queue>()
    .register_type::<Col>()
    .register_type::<Op>()
    .register_type::<Number>()
    .register_type::<Arr>()
    .register_type::<Vec<f32>>()
    .register_type::<Selected>()
    .register_type::<Visible>()
    .register_type::<Save>()
    .register_type::<Order>()
    .register_type::<BlackHole>()
    .register_type::<WhiteHole>()
    .register_type::<(i8, i8)>()
    .register_type::<Vertices>()
    .register_type::<Targets>()
    .register_type::<GainedWH>()
    .register_type::<LostWH>()
    .register_type::<DefaultDrawColor>()
    .register_type::<DefaultDrawVerts>()
    .register_type::<HighlightColor>()
    .register_type::<ConnectionColor>()
    .register_type::<CommandColor>()
    .register_type::<Version>()
    .register_type::<Holes>()
    ;

    #[cfg(feature = "inspector")]
    { app.add_plugins(WorldInspectorPlugin::new()); }

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    command_color: Res<CommandColor>,
) {
    // camera
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
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
        PanCam {
            enabled: false,
            max_scale: Some(80.),
            min_scale: 0.005,
            ..default()
        },
        RenderLayers::all(),
    ));

    // command line
    commands.spawn(NodeBundle {
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
                    TextStyle {
                        font_size: 13.0,
                        color: command_color.0,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(5.)),
                    align_self: AlignSelf::End,
                    ..default()
                }),
                CommandText,
            ));
        });

    // selection / drawing circle
    let id = commands.spawn((
        ColorMesh2dBundle {
            mesh: meshes.add(Triangle2d::default()).into(),
            material: materials.add(ColorMaterial::from(Color::hsla(0., 1., 0.5, 0.3))),
            transform: Transform::from_translation(Vec3::Z),
            ..default()
        },
        Col(Color::hsla(0., 1., 0.5, 0.3)),
    )).id();
    commands.insert_resource(SelectionCircle(id));

    // connecting line
    let id = commands.spawn((
        ColorMesh2dBundle {
            mesh: meshes.add(Triangle2d::default()).into(),
            material: materials.add(ColorMaterial::from(Color::hsla(0., 1., 0.5, 0.3))),
            transform: Transform::default(),
            ..default()
        },
        Col(Color::hsla(0., 1., 0.5, 0.3)),
    )).id();
    commands.insert_resource(ConnectingLine(id));

    // arrow mesh
    commands.insert_resource(ArrowHandle(meshes.add(Triangle2d::default()).into()));
}

fn toggle_pan(
    mut query: Query<&mut PanCam>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let mut pancam = query.single_mut();
        pancam.enabled = true;
    }
    if keyboard_input.just_released(KeyCode::Space) {
        let mut pancam = query.single_mut();
        pancam.enabled = false;
    }
}

fn save_scene(world: &mut World) {
    let mut save_events = world.resource_mut::<Events<SaveCommand>>();
    let events: Vec<SaveCommand> = save_events.drain().collect();
    for event in events {
        let name = event.0.to_string();
        let mut query = world.query_filtered::<Entity, With<Save>>();
        let scene = DynamicSceneBuilder::from_world(world)
            .allow::<Col>()
            .allow::<Transform>()
            .allow::<Op>()
            .allow::<Number>()
            .allow::<Arr>()
            .allow::<Save>()
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
            .allow_resource::<ClearColor>()
            .allow_resource::<CommandColor>()
            .allow_resource::<Version>()
            .extract_entities(query.iter(world))
            .extract_resources()
            .build();
        let type_registry = world.resource::<AppTypeRegistry>();
        let serialized_scene = scene.serialize_ron(type_registry).unwrap();

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
        .allow::<Save>()
        .allow::<Order>()
        .allow::<BlackHole>()
        .allow::<WhiteHole>()
        .allow::<Holes>()
        .allow::<Vertices>()
        .allow::<Targets>()
        .extract_entities(query.iter(world))
        .build();
    let type_registry = world.resource::<AppTypeRegistry>();
    let serialized_scene = scene.serialize_ron(type_registry).unwrap();
    let mut ctx = world.resource_mut::<SystemClipboard>();
    ctx.0.set_contents(serialized_scene).unwrap();
}

fn paste_scene(world: &mut World) {
    if let Ok(ctx) = world.resource_mut::<SystemClipboard>().0.get_contents() {
        let bytes = ctx.into_bytes();
        let mut scene = None;
        if let Ok(mut deserializer) = Deserializer::from_bytes(&bytes) {
            let type_registry = world.resource::<AppTypeRegistry>();
            let scene_deserializer = SceneDeserializer {
                type_registry: &type_registry.read(),
            };
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
        if let DroppedFile {path_buf, ..} = event {
            commands.spawn(DynamicSceneBundle {
                scene: asset_server.load(path_buf.clone()),
                ..default()
            });
        }
    }
}


#[derive(SystemParam)]
struct MoreParams<'w> {
    command_color: Res<'w, CommandColor>,
    connection_color: Res<'w, ConnectionColor>,
    arrow_handle: Res<'w, ArrowHandle>,
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
    more: MoreParams,
) {
    for (scene_id, instance_id) in scenes.iter() {
        if scene_spawner.instance_is_ready(**instance_id) {
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
                        commands.entity(*child).insert((
                            ColorMesh2dBundle {
                                mesh: polygon_handles.0[v.0].clone().unwrap(),
                                material: materials.add(ColorMaterial::from(c.0)),
                                transform: *t,
                                ..default()
                            },
                            Visible,
                        ));
                        if let Ok(op) = op_query.get_mut(*child) {
                            commands.entity(*child).insert((
                                Network(str_to_net(&op.0)),
                                NetIns(Vec::new()),
                                OpChanged(true),
                                GainedWH(false),
                                LostWH(false),
                                RenderLayers::layer(1),
                            ));
                            let holes = &mut holes_query.get_mut(*child).unwrap().0;
                            holes.retain(|x| white_hole_query.contains(*x) || black_hole_query.contains(*x));
                            for hole in holes {
                                if let Ok(mut wh) = white_hole_query.get_mut(*hole) {
                                    if black_hole_query.contains(wh.bh) && main_query.contains(wh.bh_parent) {
                                        wh.open = true;
                                        let arrow = commands.spawn(( ColorMesh2dBundle {
                                            mesh: more.arrow_handle.0.clone(),
                                            material: materials.add(ColorMaterial::from(more.connection_color.0)),
                                            transform: Transform::default(),
                                            ..default()
                                        },
                                        RenderLayers::layer(4),
                                        )).id();
                                        commands.entity(*hole).insert((
                                            ConnectionArrow(arrow),
                                            RenderLayers::layer(3),
                                        ));
                                    } else if let Some(mut e) = commands.get_entity(*hole) {
                                        e.despawn();
                                    }
                                } else if let Ok(bh) = black_hole_query.get(*hole) {
                                    if white_hole_query.contains(bh.wh) && main_query.contains(bh.wh_parent) {
                                        commands.entity(*hole).insert(RenderLayers::layer(2));
                                    } else if let Some(mut e) = commands.get_entity(*hole) {
                                        e.despawn();
                                    }
                                }
                            }
                        }
                    }
                    commands.entity(*child).remove_parent();
                }
                order_change.send_default();
            }
            // update the command line color from resource
            let clt = &mut command_line_text.single_mut();
            clt.sections[0].style.color = more.command_color.0;
            // despawn the now empty instance
            commands.entity(scene_id).despawn();
        }
    }
}

