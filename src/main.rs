use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
        },
    sprite::Mesh2dHandle,
    utils::Duration,
    winit::{WinitSettings, UpdateMode},
    tasks::IoTaskPool,
    prelude::*};
use bevy::prelude::shape::Circle as BevyCircle;

use bevy_pancam::{PanCam, PanCamPlugin};
//use bevy_inspector_egui::quick::WorldInspectorPlugin;

use std::{fs::File, io::Write};

use fundsp::hacker32::*;

mod components;
mod process;
mod cursor;
mod connections;
mod circles;
mod audio;
mod commands;
use {components::*, process::*, cursor::*, connections::*,
     circles::*, audio::*, commands::*};

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
        .insert_resource(Msaa::Sample4)

        .add_systems(Startup, setup)
        .add_systems(Startup, ext_thread)
        
        .add_systems(Update, toggle_pan)
        .add_state::<Mode>()
        .add_systems(Update, save_scene)
        .add_systems(Update, load_scene)
        .add_systems(Update, post_load)
        .init_resource::<DragModes>()
        // cursor
        .insert_resource(CursorInfo::default())
        .add_systems(Update, update_cursor_info)
        // circles
        .add_systems(Update, draw_selection_circle.run_if(not(in_state(Mode::Connect))))
        .add_systems(Update, mark_visible.after(update_cursor_info))
        .add_systems(Update, update_selection.after(mark_visible).run_if(in_state(Mode::Edit)))
        .add_systems(Update, move_selected.after(update_selection).run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_color.after(update_selection).run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_mat_from_color.after(update_color).run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_radius.after(update_selection).run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_mesh_from_radius.after(update_radius).run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_num.after(update_selection).run_if(in_state(Mode::Edit)))
        .add_systems(Update, highlight_selected.run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_order.run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_net_from_op.run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_circle_text.run_if(in_state(Mode::Edit)))
        .add_systems(Update, select_all.run_if(in_state(Mode::Edit)))
        .add_systems(Update, duplicate_selected.run_if(in_state(Mode::Edit)))
        .add_systems(Update, update_mesh_from_vertices.run_if(in_state(Mode::Edit)))
        .add_systems(Update, rotate_selected.after(update_selection).run_if(in_state(Mode::Edit)))
        // events
        .add_event::<ColorChange>()
        .add_event::<RadiusChange>()
        .add_event::<OpChange>()
        .add_event::<OrderChange>()
        .add_event::<SceneLoaded>()
        // connections
        .add_systems(Update, connect.run_if(in_state(Mode::Connect)))
        .add_systems(Update, update_connection_arrows)
        .add_systems(Update, draw_connecting_arrow.run_if(in_state(Mode::Connect)))
        .add_systems(Update, update_link_type_text.run_if(in_state(Mode::Edit)))
        .add_systems(Update, mark_children_change)
        // order
        .add_systems(Update, (spawn_circles.run_if(in_state(Mode::Draw)),
                              remove_connections.run_if(in_state(Mode::Edit)),
                              delete_selected.run_if(in_state(Mode::Edit)),
                              apply_deferred, //to make sure the commands are applied
                              sort_by_order.run_if(on_event::<OrderChange>())).chain())
        .init_resource::<Queue>()
        // process
        .add_systems(Update, process.after(sort_by_order))
        // commands
        .add_systems(Update, command_parser)

        // type registry
        .register_type::<DragModes>()
        .register_type::<Queue>()
        .register_type::<Radius>()
        .register_type::<Col>()
        .register_type::<Op>()
        .register_type::<crate::components::Num>()
        .register_type::<Arr>()
        .register_type::<Vec<f32>>()
        .register_type::<Selected>()
        .register_type::<Visible>()
        .register_type::<Save>()
        .register_type::<Order>()
        .register_type::<OpChanged>()
        .register_type::<BlackHole>()
        .register_type::<WhiteHole>()
        .register_type::<(i32, i32)>()
        .register_type::<Vertices>()
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
                mesh: meshes.add(BevyCircle {radius: 0., vertices: 12} .into()).into(),
                material: materials.add(ColorMaterial::from(Color::hsla(0., 1., 0.5, 0.3))),
                transform: Transform::from_translation(Vec3::Z),
                ..default()
            },
        )).id();
        commands.insert_resource(SelectionCircle(id));

        // connecting line
        let id = commands.spawn((
            ColorMesh2dBundle {
                mesh: meshes.add(BevyCircle {radius: 0., vertices: 3} .into()).into(),
                material: materials.add(ColorMaterial::from(Color::hsla(0., 1., 0.5, 0.3))),
                transform: Transform::from_translation(Vec3::Z),
                ..default()
            },
        )).id();
        commands.insert_resource(ConnectingLine(id));
}

fn toggle_pan(
    mut query: Query<&mut PanCam>,
    keyboard_input: Res<Input<KeyCode>>,
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
    let keyboard_input = world.resource::<Input<KeyCode>>();
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if ctrl && keyboard_input.just_pressed(KeyCode::S) {

        let mut query = world.query_filtered::<Entity, With<Save>>();
        let scene = DynamicSceneBuilder::from_world(&world)
            //.allow_resource::<Queue>()
            .allow::<Radius>()
            .allow::<Col>()
            .allow::<Transform>()
            .allow::<GlobalTransform>()
            .allow::<Op>()
            .allow::<crate::components::Num>()
            .allow::<Arr>()
            .allow::<Selected>()
            .allow::<Visible>()
            .allow::<Save>()
            .allow::<Order>()
            .allow::<OpChanged>()
            .allow::<BlackHole>()
            .allow::<WhiteHole>()
            .allow::<Parent>()
            .allow::<Children>()
            .allow::<Text>()
            .allow::<ViewVisibility>()
            .allow::<InheritedVisibility>()
            .allow::<Visibility>()
            .allow::<Vertices>()
            .extract_entities(query.iter(&world))
            //.extract_resources()
            .build();
        let type_registry = world.resource::<AppTypeRegistry>();
        let serialized_scene = scene.serialize_ron(type_registry).unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        IoTaskPool::get()
            .spawn(async move {
                File::create(format!("scene.scn.ron"))
                    .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                    .expect("Error while writing scene to file");
            })
            .detach();
    }
}

fn load_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<Input<KeyCode>>,
    mut scene_load_event: EventWriter<SceneLoaded>,
) {
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if ctrl && keyboard_input.just_pressed(KeyCode::O) {
        commands.spawn(DynamicSceneBundle {
            scene: asset_server.load("scene.scn.ron"),
            ..default()
        });
        scene_load_event.send_default();
    }
}

fn post_load(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &Radius, &Transform, &Col, &Vertices)>,
    text_query: Query<(Entity, &Text), With<Save>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut op_change_event: EventWriter<OpChange>,
    op_query: Query<&Op>,
    mut order_change: EventWriter<OrderChange>,
    white_hole_query: Query<With<WhiteHole>>,
) {
    let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if ctrl && keyboard_input.just_pressed(KeyCode::P) {
        for (e, r, t, c, v) in query.iter() {
            commands.entity(e).insert(
                ColorMesh2dBundle {
                    mesh: meshes.add(BevyCircle { radius: r.0, vertices: v.0 }.into()).into(),
                    material: materials.add(ColorMaterial::from(c.0)),
                    transform: *t,
                    ..default()
                },
            );
            if let Ok(op) = op_query.get(e) {
                commands.entity(e).insert((
                    Network(Net32::new(0,1)),
                    NetIns(Vec::new()),
                ));
                op_change_event.send(OpChange(e, op.0.clone()));
            } else if white_hole_query.contains(e) {
                let arrow = commands.spawn( ColorMesh2dBundle {
                    // doesn't matter, it's gonna get replaced
                    mesh: meshes.add(BevyCircle::new(0.).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::hsla(0., 1., 1., 0.7))),
                    transform: Transform::from_translation(Vec3::Z),
                    ..default()
                }).id();
                commands.entity(e).insert(ConnectionArrow(arrow));
            }
        }
        for (e, t) in text_query.iter() {
            commands.entity(e).insert(
                Text2dBundle {
                    text: t.clone(),
                    transform: Transform::from_translation(Vec3{z:0.000001, ..default()}),
                ..default()
                }
            );
        }
        order_change.send_default();
    }
}

fn draw_selection_circle(
    cursor: Res<CursorInfo>,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    id: Res<SelectionCircle>,
    mut trans_query: Query<&mut Transform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle>,
) {
    if mouse_button_input.pressed(MouseButton::Left) &&
    ! mouse_button_input.just_pressed(MouseButton::Left) &&
    !keyboard_input.pressed(KeyCode::Space) {
        trans_query.get_mut(id.0).unwrap().translation = cursor.i.extend(1.);
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = BevyCircle { radius: cursor.i.distance(cursor.f), vertices: 12 }.into();
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        trans_query.get_mut(id.0).unwrap().translation = Vec3::Z;
        let Mesh2dHandle(mesh_id) = mesh_ids.get(id.0).unwrap();
        let mesh = meshes.get_mut(mesh_id).unwrap();
        *mesh = BevyCircle { radius: 0., vertices: 12 }.into();
    }
}


