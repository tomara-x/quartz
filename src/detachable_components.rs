use bevy::{
    prelude::*};

use crate::{circles::*};

pub struct DetachableComponentsPlugin;

impl Plugin for DetachableComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Num>();
        app.register_type::<Arr>();
        app.register_type::<ColorOffset>();
        app.register_type::<PosOffset>();
        app.register_type::<RadiusOffset>();
        app.add_systems(Update, attach_detach_data.run_if(in_state(Mode::Edit)));
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Num(f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Arr(Vec<f32>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct ColorOffset(Color);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct PosOffset(Vec3);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct RadiusOffset(f32);

fn attach_detach_data(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<Entity, With<Selected>>,
    mut commands: Commands,
) {
    let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if keyboard_input.just_pressed(KeyCode::F1) {
        if shift {
            for e in query.iter() {
                commands.entity(e).remove::<Num>();
            }
        } else {
            for e in query.iter() {
                commands.entity(e).insert(Num(0.0));
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::F2) {
        if shift {
            for e in query.iter() {
                commands.entity(e).remove::<Arr>();
            }
        } else {
            for e in query.iter() {
                commands.entity(e).insert(Arr(vec![0.,1.,2.,4.]));
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::F3) {
        if shift {
            for e in query.iter() {
                commands.entity(e).remove::<ColorOffset>();
            }
        } else {
            for e in query.iter() {
                commands.entity(e).insert(ColorOffset(Color::hsl(0.0,1.0,0.5)));
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::F4) {
        if shift {
            for e in query.iter() {
                commands.entity(e).remove::<PosOffset>();
            }
        } else {
            for e in query.iter() {
                commands.entity(e).insert(PosOffset(Vec3::ONE));
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::F5) {
        if shift {
            for e in query.iter() {
                commands.entity(e).remove::<RadiusOffset>();
            }
        } else {
            for e in query.iter() {
                commands.entity(e).insert(RadiusOffset(1.));
            }
        }
    }
}

