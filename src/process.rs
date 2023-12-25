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
        .insert_resource(BloomCircleId(Entity::from_raw(0)))
        .add_systems(Startup, spawn_bloom_circle)
        .add_systems(Update, update_bloom_settings)
        ;
    }
}

#[derive(Resource)]
struct BloomCircleId(Entity);

fn spawn_bloom_circle(
    mut commands: Commands,
    mut resource: ResMut<BloomCircleId>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let id = commands.spawn((
        ColorMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(100.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::hsl(300., 1.0, 0.5))),
            transform: Transform::from_translation(Vec3{x:0., y:0., z:-11.}),
            ..default()
        },
        Radius(100.),
        Visible,
        Order(0),
        Num(0.),
        Arr(Vec::new()),
        Offset {trans:Vec3::ZERO, color:Color::BLACK, radius:0.},
    )).id();
    let text = commands.spawn(Text2dBundle {
        text: Text::from_sections([
            TextSection::new(
                "order: 0\n",
                TextStyle::default()
            ),
            TextSection::new(
                "0\n",
                TextStyle::default()
            ),
        ]),
        transform: Transform::from_translation(Vec3{z:0.000001, ..default()}),
        ..default()
    }).id();
    commands.entity(id).add_child(text);
    resource.0 = id;
}


fn update_bloom_settings(
    children_query: Query<&Children>,
    mut bloom: Query<&mut BloomSettings, With<Camera>>,
    black_hole_query: Query<&BlackHole>,
    white_hole_query: Query<&WhiteHole>,
    id: Res<BloomCircleId>,
    num_query: Query<&Num>,
) {
    let mut bloom_settings = bloom.single_mut();
    // why doesn't iter_descendants need error checking?
    for child in children_query.iter_descendants(id.0) {
        if let Ok(white_hole) = white_hole_query.get(child) {
            let black_hole = black_hole_query.get(white_hole.bh).unwrap();
            let input = num_query.get(black_hole.parent).unwrap().0 / 100.;
            match (black_hole.link_type, white_hole.link_type) {
                (4, 9) => bloom_settings.intensity = input,
                (4, 10) => bloom_settings.low_frequency_boost = input,
                (4, 11) => bloom_settings.low_frequency_boost_curvature = input,
                (4, 12) => bloom_settings.high_pass_frequency = input,
                (4, 13) => bloom_settings.composite_mode = if input > 0.5 {
                BloomCompositeMode::Additive } else { BloomCompositeMode::EnergyConserving },
                (4, 14) => bloom_settings.prefilter_settings.threshold = input,
                (4, 15) => bloom_settings.prefilter_settings.threshold_softness = input,
                _ => {},
            }
        }
    }
}

// updating color/position/radius from inputs and applying offset go here
// maybe in separate systems tho, cause it applies to all entities with inputs

//fn update_position(
//    mut query: Query<&mut Transform, With<Selected>>,
//) {
