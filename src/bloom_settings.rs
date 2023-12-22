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
    control: Query<(&BloomControl, &Children)>,
    mut bloom: Query<&mut BloomSettings, With<Camera>>,
    black_hole_query: Query<&BlackHole>,
    white_hole_query: Query<&WhiteHole>,
    connection_indices: Res<ConnectionIndices>,
    entity_indices: ResMut<EntityIndices>,
    num_query: Query<&Num>,
) {
    let mut bloom_settings = bloom.single_mut();
    for (control, children) in control.iter() {
        for child in children {
            if let Ok(white_hole) = white_hole_query.get(*child) {
                if white_hole.connection_type == 9 {
                    let black_hole = black_hole_query.get(connection_indices.0[white_hole.black_hole]).unwrap();
                    if black_hole.connection_type == 9 {
                        if let Ok(input) = num_query.get(entity_indices.0[black_hole.parent]) {
                            match control {
                                BloomControl::Intensity => bloom_settings.intensity = input.0,
                                BloomControl::LFBoost => bloom_settings.low_frequency_boost = input.0,
                                BloomControl::LFBCurvature => bloom_settings.low_frequency_boost_curvature = input.0,
                                BloomControl::HPFreq => bloom_settings.high_pass_frequency = input.0,
                                BloomControl::CompositeMode => bloom_settings.composite_mode = if input.0 > 0.5 {
                                    BloomCompositeMode::Additive } else { BloomCompositeMode::EnergyConserving },
                                BloomControl::PrefilterThreshold => bloom_settings.prefilter_settings.threshold = input.0,
                                BloomControl::PrefilterThresholdSoftness => bloom_settings.prefilter_settings.
                                    threshold_softness = input.0,
                            }
                        }
                    }
                }
            break;
            }
        }
    }
}

    //let dt = time.delta_seconds();
    //if keycode.pressed(KeyCode::A) {
    //    bloom_settings.intensity -= dt / 10.0;
    //}
    //if keycode.pressed(KeyCode::Q) {
    //    bloom_settings.intensity += dt / 10.0;
    //}
    //bloom_settings.intensity = bloom_settings.intensity.clamp(0.0, 1.0);

    //if keycode.pressed(KeyCode::S) {
    //    bloom_settings.low_frequency_boost -= dt / 10.0;
    //}
    //if keycode.pressed(KeyCode::W) {
    //    bloom_settings.low_frequency_boost += dt / 10.0;
    //}
    //bloom_settings.low_frequency_boost = bloom_settings.low_frequency_boost.clamp(0.0, 1.0);

    //if keycode.pressed(KeyCode::D) {
    //    bloom_settings.low_frequency_boost_curvature -= dt / 10.0;
    //}
    //if keycode.pressed(KeyCode::E) {
    //    bloom_settings.low_frequency_boost_curvature += dt / 10.0;
    //}
    //bloom_settings.low_frequency_boost_curvature =
    //    bloom_settings.low_frequency_boost_curvature.clamp(0.0, 1.0);

    //if keycode.pressed(KeyCode::F) {
    //    bloom_settings.high_pass_frequency -= dt / 10.0;
    //}
    //if keycode.pressed(KeyCode::R) {
    //    bloom_settings.high_pass_frequency += dt / 10.0;
    //}
    //bloom_settings.high_pass_frequency = bloom_settings.high_pass_frequency.clamp(0.0, 1.0);

    //if keycode.pressed(KeyCode::G) {
    //    bloom_settings.composite_mode = BloomCompositeMode::Additive;
    //}
    //if keycode.pressed(KeyCode::T) {
    //    bloom_settings.composite_mode = BloomCompositeMode::EnergyConserving;
    //}

    //if keycode.pressed(KeyCode::H) {
    //    bloom_settings.prefilter_settings.threshold -= dt;
    //}
    //if keycode.pressed(KeyCode::Y) {
    //    bloom_settings.prefilter_settings.threshold += dt;
    //}
    //bloom_settings.prefilter_settings.threshold =
    //    bloom_settings.prefilter_settings.threshold.max(0.0);

    //if keycode.pressed(KeyCode::J) {
    //    bloom_settings.prefilter_settings.threshold_softness -= dt / 10.0;
    //}
    //if keycode.pressed(KeyCode::U) {
    //    bloom_settings.prefilter_settings.threshold_softness += dt / 10.0;
    //}
    //bloom_settings.prefilter_settings.threshold_softness = bloom_settings
    //    .prefilter_settings
    //    .threshold_softness
    //    .clamp(0.0, 1.0);

