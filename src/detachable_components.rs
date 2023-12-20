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
        app.add_systems(Update, attach_data.run_if(in_state(Mode::Edit)));
        app.add_systems(Update, detach_data.run_if(in_state(Mode::Edit)));
    }
}

#[derive(Component, Reflect)]
struct Num(f32);

#[derive(Component, Reflect)]
struct Arr(Vec<f32>);

#[derive(Component, Reflect)]
struct ColorOffset(Color);

#[derive(Component, Reflect)]
struct PosOffset(Vec3);

#[derive(Component, Reflect)]
struct RadiusOffset(f32);

fn attach_data(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<Entity, With<Selected>>,
    mut commands: Commands,
) {
    if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keyboard_input.just_pressed(KeyCode::N) {
            for e in query.iter() {
                commands.entity(e).insert(Num(0.0));
            }
        }
        if keyboard_input.just_pressed(KeyCode::A) {
            for e in query.iter() {
                commands.entity(e).insert(Arr(vec![0.,1.,2.,4.]));
            }
        }
        if keyboard_input.just_pressed(KeyCode::C) {
            for e in query.iter() {
                commands.entity(e).insert(ColorOffset(Color::hsl(0.0,1.0,0.5)));
            }
        }
        if keyboard_input.just_pressed(KeyCode::P) {
            for e in query.iter() {
                commands.entity(e).insert(PosOffset(Vec3::ONE));
            }
        }
        if keyboard_input.just_pressed(KeyCode::R) {
            for e in query.iter() {
                commands.entity(e).insert(RadiusOffset(1.));
            }
        }
    }
}

fn detach_data(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<Entity, With<Selected>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Key1) {
        for e in query.iter() {
            commands.entity(e).remove::<Num>();
        }
    }
    if keyboard_input.just_pressed(KeyCode::Key2) {
        for e in query.iter() {
            commands.entity(e).remove::<Arr>();
        }
    }
    if keyboard_input.just_pressed(KeyCode::Key3) {
        for e in query.iter() {
            commands.entity(e).remove::<ColorOffset>();
        }
    }
    if keyboard_input.just_pressed(KeyCode::Key4) {
        for e in query.iter() {
            commands.entity(e).remove::<PosOffset>();
        }
    }
    if keyboard_input.just_pressed(KeyCode::Key5) {
        for e in query.iter() {
            commands.entity(e).remove::<RadiusOffset>();
        }
    }
}


