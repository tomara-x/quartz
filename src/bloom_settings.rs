use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
    },
    prelude::*,
};

use crate::{connections::*, circles::*, detachable_components::*};

pub struct BloomSettingsPlugin;

impl Plugin for BloomSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_bloom_settings);
    }
}


fn update_bloom_settings(
    mut bloom: Query<&mut BloomSettings, With<Camera>>,
    keycode: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut bloom_settings = bloom.single_mut();
    let dt = time.delta_seconds();

    if keycode.pressed(KeyCode::A) {
        bloom_settings.intensity -= dt / 10.0;
    }
    if keycode.pressed(KeyCode::Q) {
        bloom_settings.intensity += dt / 10.0;
    }
    bloom_settings.intensity = bloom_settings.intensity.clamp(0.0, 1.0);

    if keycode.pressed(KeyCode::S) {
        bloom_settings.low_frequency_boost -= dt / 10.0;
    }
    if keycode.pressed(KeyCode::W) {
        bloom_settings.low_frequency_boost += dt / 10.0;
    }
    bloom_settings.low_frequency_boost = bloom_settings.low_frequency_boost.clamp(0.0, 1.0);

    if keycode.pressed(KeyCode::D) {
        bloom_settings.low_frequency_boost_curvature -= dt / 10.0;
    }
    if keycode.pressed(KeyCode::E) {
        bloom_settings.low_frequency_boost_curvature += dt / 10.0;
    }
    bloom_settings.low_frequency_boost_curvature =
        bloom_settings.low_frequency_boost_curvature.clamp(0.0, 1.0);

    if keycode.pressed(KeyCode::F) {
        bloom_settings.high_pass_frequency -= dt / 10.0;
    }
    if keycode.pressed(KeyCode::R) {
        bloom_settings.high_pass_frequency += dt / 10.0;
    }
    bloom_settings.high_pass_frequency = bloom_settings.high_pass_frequency.clamp(0.0, 1.0);

    if keycode.pressed(KeyCode::G) {
        bloom_settings.composite_mode = BloomCompositeMode::Additive;
    }
    if keycode.pressed(KeyCode::T) {
        bloom_settings.composite_mode = BloomCompositeMode::EnergyConserving;
    }

    if keycode.pressed(KeyCode::H) {
        bloom_settings.prefilter_settings.threshold -= dt;
    }
    if keycode.pressed(KeyCode::Y) {
        bloom_settings.prefilter_settings.threshold += dt;
    }
    bloom_settings.prefilter_settings.threshold =
        bloom_settings.prefilter_settings.threshold.max(0.0);

    if keycode.pressed(KeyCode::J) {
        bloom_settings.prefilter_settings.threshold_softness -= dt / 10.0;
    }
    if keycode.pressed(KeyCode::U) {
        bloom_settings.prefilter_settings.threshold_softness += dt / 10.0;
    }
    bloom_settings.prefilter_settings.threshold_softness = bloom_settings
        .prefilter_settings
        .threshold_softness
        .clamp(0.0, 1.0);
}
