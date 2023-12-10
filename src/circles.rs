use bevy::{
    render::view::VisibleEntities,
    sprite::Mesh2dHandle,
    prelude::*};

use crate::cursor::*;

pub struct CirclesPlugin;

impl Plugin for CirclesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Depth {value: -10.});
        app.insert_resource(CursorInfo {i: Vec2::ZERO, f: Vec2::ZERO});
        app.add_systems(Update, spawn_circles);
        app.add_systems(Update, update_color);
        app.add_systems(Update, update_radius);
        app.add_systems(Update, draw_pointer_circle);
        app.add_systems(Update,
            (mark_visible, update_selection, highlight_selected, move_selected).chain());
        app.add_systems(Update, delete_selected);
    }
}

#[derive(Resource)]
struct Depth { value: f32 }

#[derive(Component)]
pub struct Radius { pub value: f32 }

#[derive(Component)]
pub struct Pos { pub value: Vec3 }

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Visible;

fn spawn_circles(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut depth: ResMut<Depth>,
    cursor: Res<CursorInfo>,
) {
    if mouse_button_input.just_released(MouseButton::Left) && keyboard_input.pressed(KeyCode::Z) {
        commands.spawn((ColorMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(cursor.f.distance(cursor.i)).into()).into(),
        material: materials.add(ColorMaterial::from(Color::hsl(0., 1.0, 0.5))),
        transform: Transform::from_translation(cursor.i.extend(depth.value)),
        ..default()
        },
        Radius { value: cursor.f.distance(cursor.i)}, //opt?
        // to keep track of initial position while moving (use local param?)
        Pos { value: cursor.i.extend(depth.value)},
        ));
        depth.value += 0.00001;
    }
}

fn update_color(
    mut mats: ResMut<Assets<ColorMaterial>>,
    material_ids: Query<&Handle<ColorMaterial>, With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
) {
    if keyboard_input.pressed(KeyCode::C) {
        for id in material_ids.iter() {
            let mat = mats.get_mut(id).unwrap();
            mat.color = Color::hsl(cursor.i.distance(cursor.f)%360., 1.0, 0.5);
        }
    }
}

fn update_radius(
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_ids: Query<&Mesh2dHandle, With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
    cursor: Res<CursorInfo>,
) {
    if keyboard_input.pressed(KeyCode::V) {
        for Mesh2dHandle(id) in mesh_ids.iter() {
            let mesh = meshes.get_mut(id).unwrap();
            *mesh = shape::Circle::new(cursor.f.distance(cursor.i)).into();
        }
    }
}


//need to make this conditional
fn draw_pointer_circle(
    cursor: Res<CursorInfo>,
    mut gizmos: Gizmos,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        gizmos.circle_2d(cursor.i, cursor.f.distance(cursor.i), Color::GREEN).segments(64);
    }
}

fn highlight_selected(
    mut gizmos: Gizmos,
    query: Query<(&Radius, &Pos), With<Selected>>,
) {
    for (r, p) in query.iter() {
        gizmos.circle_2d(p.value.xy(), r.value, Color::BLUE).segments(64);
    }
}

// loop over the visible entities and give them a Visible component
// so we can query just the visible entities
fn mark_visible(
    mouse_button_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    query: Query<Entity, With<Visible>>,
    visible: Query<&VisibleEntities>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        for e in query.iter() {
            commands.entity(e).remove::<Visible>();
        }
        let vis = visible.single();
        for e in &vis.entities {
            commands.entity(*e).insert(Visible);
        }
    }
}

fn update_selection(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<(Entity, &Radius, &Pos), Or<(With<Visible>, With<Selected>)>>,
    selected: Query<Entity, With<Selected>>,
    cursor: Res<CursorInfo>,
    keyboard_input: Res<Input<KeyCode>>,
    mut clicked_on_circle: Local<bool>,
) {
    let mut none_selected = selected.is_empty();
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    if mouse_button_input.just_pressed(MouseButton::Left) {
        for (e, r, p) in query.iter() {
            if cursor.i.distance(p.value.xy()) < r.value {
                *clicked_on_circle = true;
                if none_selected {
                    commands.entity(e).insert(Selected);
                }
                break;
            }
        }
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        if *clicked_on_circle {
            // some entities are selected and we just clicked on one
            if !none_selected && cursor.i.distance(cursor.f) < 0.01 {
                for entity in selected.iter() {
                    commands.entity(entity).remove::<Selected>();
                }
                for (e, r, p) in query.iter() {
                    if cursor.i.distance(p.value.xy()) < r.value {
                        commands.entity(e).insert(Selected);
                        break;
                    }
                }
            }
        }
        else {
            // deselect everything
            for entity in selected.iter() {
                commands.entity(entity).remove::<Selected>();
            }
            // select those in the dragged area
            for (e, r, p) in query.iter() {
                if cursor.i.distance(cursor.f) + r.value > cursor.i.distance(p.value.xy()) {
                    commands.entity(e).insert(Selected);
                }
            }
        }
    }
}

// move the selected entities by changing the translation of entity directly
// when mouse is released we store the translation in temporary position component
fn move_selected(
    mouse_button_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
    mut query: Query<(&mut Transform, &mut Pos), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) && keyboard_input.pressed(KeyCode::X) {
        for (mut t, p) in query.iter_mut() {
            t.translation = (p.value.xy() + cursor.f - cursor.i).extend(p.value.z);
            //t.translation.x = p.value.x + cursor.f.x - cursor.i.x;
            //t.translation.y = p.value.y + cursor.f.y - cursor.i.y;
        }
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        for (t, mut p) in query.iter_mut() {
            p.value = t.translation;
        }
    }
}

fn delete_selected(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<Entity, With<Selected>>,
    mut commands: Commands,
) {
    if keyboard_input.pressed(KeyCode::Delete) {
        for id in query.iter() {
            commands.entity(id).despawn();
        }
    }
}

