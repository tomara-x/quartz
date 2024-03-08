use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
        },
    utils::Duration,
    winit::{WinitSettings, UpdateMode},
    tasks::IoTaskPool,
    scene::SceneInstance,
    render::view::RenderLayers,
    prelude::*};

use bevy_pancam::{PanCam, PanCamPlugin};
//use bevy_inspector_egui::quick::WorldInspectorPlugin;

use std::{fs::File, io::Write};

mod components;
mod process;
mod cursor;
mod connections;
mod circles;
mod audio;
mod commands;
mod nodes;
mod functions;
mod meshes;
use {components::*, process::*, cursor::*, connections::*,
     circles::*, audio::*, commands::*, functions::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
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
            ..default()
        })

        .add_plugins(PanCamPlugin::default())
        //.add_plugins(WorldInspectorPlugin::new())

        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(DefaultDrawColor(Color::hsl(1.,1.,0.84)))
        .insert_resource(DefaultDrawVerts(4))
        .insert_resource(HighlightColor(Color::hsl(0.0,1.0,0.5)))
        .insert_resource(ConnectionColor(Color::hsla(0., 1., 1., 0.7)))
        .insert_resource(Msaa::Sample4)

        .add_systems(Startup, setup)
        .add_systems(Startup, ext_thread)
        
        .add_systems(Update, toggle_pan)
        .init_state::<Mode>()
        .add_systems(Update, save_scene)
        .add_systems(Update, post_load)
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
        .add_systems(Update, update_mesh)
        .add_systems(Update, update_num.after(update_selection).run_if(in_state(Mode::Edit)))
        .add_systems(Update, highlight_selected)
        .add_systems(PreUpdate, transform_highlights)
        .add_systems(Update, update_order.run_if(in_state(Mode::Edit)))
        .add_systems(Update, rotate_selected.after(update_selection).run_if(in_state(Mode::Edit)))
        .add_systems(Update, (delete_selected_holes, delete_selected_circles).chain().run_if(in_state(Mode::Edit)))
        // text
        .add_systems(PreUpdate, update_info_text)
        // events
        .add_event::<SaveCommand>()
        // connections
        .add_systems(Update, connect.run_if(in_state(Mode::Connect)))
        .add_systems(Update, target.run_if(in_state(Mode::Connect)))
        .add_systems(PreUpdate, update_connection_arrows)
        .add_systems(Update, draw_connecting_arrow.run_if(in_state(Mode::Connect)))
        .add_systems(Update, mark_children_change)
        // order
        .init_resource::<Queue>()
        .init_resource::<LoopQueue>()
        .add_event::<OrderChange>()
        .add_systems(PostUpdate, sort_by_order.before(process).run_if(on_event::<OrderChange>()))
        .add_systems(PostUpdate, prepare_loop_queue.after(sort_by_order))
        // process
        .add_systems(PostUpdate, process)
        // commands
        .add_systems(Update, command_parser)

        // type registry
        .register_type::<DragModes>()
        .register_type::<Queue>()
        .register_type::<Radius>()
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
        .register_type::<(i32, i32)>()
        .register_type::<Vertices>()
        .register_type::<Targets>()
        .register_type::<GainedWH>()
        .register_type::<LostWH>()
        .register_type::<DefaultDrawColor>()
        .register_type::<DefaultDrawVerts>()
        .register_type::<HighlightColor>()
        .register_type::<ConnectionColor>()
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // camera
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_translation(Vec3::Z),
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
                mesh: meshes.add(RegularPolygon::new(0.1, 12)).into(),
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
                mesh: meshes.add(RegularPolygon::new(0.1, 3)).into(),
                material: materials.add(ColorMaterial::from(Color::hsla(0., 1., 0.5, 0.3))),
                transform: Transform::from_translation(Vec3::Z),
                ..default()
            },
            Col(Color::hsla(0., 1., 0.5, 0.3)),
        )).id();
        commands.insert_resource(ConnectingLine(id));
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
        let scene = DynamicSceneBuilder::from_world(&world)
            .allow::<Radius>()
            .allow::<Col>()
            .allow::<Transform>()
            .allow::<Op>()
            .allow::<Number>()
            .allow::<Arr>()
            .allow::<Save>()
            .allow::<Order>()
            .allow::<BlackHole>()
            .allow::<WhiteHole>()
            .allow::<Parent>()
            .allow::<Children>()
            .allow::<Vertices>()
            .allow::<Targets>()
            .allow_resource::<DefaultDrawColor>()
            .allow_resource::<DefaultDrawVerts>()
            .allow_resource::<HighlightColor>()
            .allow_resource::<ConnectionColor>()
            .allow_resource::<ClearColor>()
            .extract_entities(query.iter(&world))
            .extract_resources()
            .build();
        let type_registry = world.resource::<AppTypeRegistry>();
        let serialized_scene = scene.serialize_ron(type_registry).unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        IoTaskPool::get()
            .spawn(async move {
                File::create(format!("assets/{}.scn.ron", name))
                    .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                    .expect("Error while writing scene to file");
            })
            .detach();
    }
}

fn post_load(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    main_query: Query<(&Radius, &Transform, &Col, &Vertices)>,
    mut order_change: EventWriter<OrderChange>,
    mut white_hole_query: Query<&mut WhiteHole>,
    black_hole_query: Query<(), With<BlackHole>>,
    scene: Query<(Entity, &Children), With<SceneInstance>>,
    children_query: Query<&Children>,
    mut op_query: Query<&mut Op>,
    connection_color: Res<ConnectionColor>,
) {
    if let Ok((scene_entity, children)) = scene.get_single() {
        for child in children {
            if let Ok((r, t, c, v)) = main_query.get(*child) {
                let op = &op_query.get_mut(*child).unwrap().0;
                commands.entity(*child).insert((
                    ColorMesh2dBundle {
                        mesh: meshes.add(RegularPolygon::new(r.0, v.0)).into(),
                        material: materials.add(ColorMaterial::from(c.0)),
                        transform: *t,
                        ..default()
                    },
                    Network(str_to_net(op)),
                    NetIns(Vec::new()),
                    OpChanged(true),
                    GainedWH(false),
                    LostWH(false),
                    RenderLayers::layer(1),
                ));
                if let Ok(holes) = children_query.get(*child) {
                    for hole in holes {
                        if white_hole_query.contains(*hole) || black_hole_query.contains(*hole) {
                            if let Ok((r, t, c, v)) = main_query.get(*hole) {
                                commands.entity(*hole).insert(
                                    ColorMesh2dBundle {
                                        mesh: meshes.add(RegularPolygon::new(r.0, v.0)).into(),
                                        material: materials.add(ColorMaterial::from(c.0)),
                                        transform: *t,
                                        ..default()
                                    },
                                );
                            }
                        }
                        if black_hole_query.contains(*hole) {
                            commands.entity(*hole).insert(RenderLayers::layer(2));
                        }
                        if let Ok(mut wh) = white_hole_query.get_mut(*hole) {
                            wh.open = true;
                            let arrow = commands.spawn(( ColorMesh2dBundle {
                                mesh: meshes.add(RegularPolygon::new(0.1, 3)).into(),
                                material: materials.add(ColorMaterial::from(connection_color.0)),
                                transform: Transform::from_translation(Vec3::Z),
                                ..default()
                            },
                            RenderLayers::layer(4),
                            )).id();
                            commands.entity(*hole).insert((
                                ConnectionArrow(arrow),
                                RenderLayers::layer(3),
                            ));
                        }
                    }
                }
            }
            commands.entity(*child).remove_parent();
        }
        order_change.send_default();
        commands.entity(scene_entity).despawn();
    }
}

