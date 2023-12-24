use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
    },
    prelude::*,
};

use crate::{connections::*, circles::*};

pub struct ProcessPlugin;

impl Plugin for ProcessPlugin {
    fn build(&self, app: &mut App) { app
        .add_systems(Update, visual)
        ;
    }
}


#[derive(Component)]
enum Vop {
    No,
    BloomControl,
}

fn assign_op() {}

// updating color/position/radius from inputs and applying offset go here
// maybe in separate systems tho, cause it applies to all entities with inputs

fn visual(
    children_query: Query<&Children>,
    query: Query<(Entity, &Vop)>,
    mut bloom: Query<&mut BloomSettings, With<Camera>>,
    black_hole_query: Query<&BlackHole>,
    white_hole_query: Query<&WhiteHole>,
    rad_query: Query<&Radius>,
) {
    let mut bloom_settings = bloom.single_mut();
    for (id, op) in query.iter() {
        match op {
            Vop::No => {},
            Vop::BloomControl => for child in children_query.iter_descendants(id) {
                if let Ok(white_hole) = white_hole_query.get(child) {
                    let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                    if black_hole.link_type == 3 {
                        if let Ok(input) = rad_query.get(black_hole.parent) {
                            let input = input.0 / 100.;
                            match white_hole.link_type {
                                9 => bloom_settings.intensity = input,
                                10 => bloom_settings.low_frequency_boost = input,
                                11 => bloom_settings.low_frequency_boost_curvature = input,
                                12 => bloom_settings.high_pass_frequency = input,
                                13 => bloom_settings.composite_mode = if input > 0.5 {
                                    BloomCompositeMode::Additive
                                } else { BloomCompositeMode::EnergyConserving },
                                14 => bloom_settings.prefilter_settings.threshold = input,
                                15 => bloom_settings.prefilter_settings.threshold_softness = input,
                                _ => {},
                            }
                        }
                    }
                }
            },
        }
    }
}

