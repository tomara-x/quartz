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
        .add_systems(Update, update_color)
        .add_systems(Update, update_num)
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
    mut white_hole_query: Query<&mut WhiteHole>,
    id: Res<BloomCircleId>,
    num_query: Query<&Num>,
) {
    let mut bloom_settings = bloom.single_mut();
    // why doesn't iter_descendants need error checking?
    for child in children_query.iter_descendants(id.0) {
        if let Ok(mut white_hole) = white_hole_query.get_mut(child) {
            if !white_hole.changed { continue; }
            white_hole.changed = false;
            let black_hole = black_hole_query.get(white_hole.bh).unwrap();
            let input = num_query.get(black_hole.parent).unwrap().0 / 100.;
            match (black_hole.link_type, white_hole.link_type) {
                (-4, 1) => bloom_settings.intensity = input,
                (-4, 2) => bloom_settings.low_frequency_boost = input,
                (-4, 3) => bloom_settings.low_frequency_boost_curvature = input,
                (-4, 4) => bloom_settings.high_pass_frequency = input,
                (-4, 5) => bloom_settings.composite_mode = if input > 0.5 {
                BloomCompositeMode::Additive } else { BloomCompositeMode::EnergyConserving },
                (-4, 6) => bloom_settings.prefilter_settings.threshold = input,
                (-4, 7) => bloom_settings.prefilter_settings.threshold_softness = input,
                _ => {},
            }
        }
    }
}

fn update_num(
    query: Query<(Entity, &Children)>,
    black_hole_query: Query<&BlackHole>,
    mut white_hole_query: Query<&mut WhiteHole>,
    mut num_query: Query<&mut Num>,
) {
    for (e, children) in query.iter() {
        for child in children.iter() {
            if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                if !white_hole.changed { continue; }
                white_hole.changed = false;
                let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                let input = num_query.get(black_hole.parent).unwrap().0;
                if black_hole.link_type == -4 && white_hole.link_type == -4 {
                    num_query.get_mut(e).unwrap().0 = input;
                    // now we have to let anything connected to this circle know about this change
                    for child in children.iter() {
                        if let Ok(black_hole) = black_hole_query.get(*child) {
                            if black_hole.link_type == -4 {
                                white_hole_query.get_mut(black_hole.wh).unwrap().changed = true;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn update_color(
    query: Query<(Entity, &Children)>,
    black_hole_query: Query<&BlackHole>,
    mut white_hole_query: Query<&mut WhiteHole>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    material_ids: Query<&Handle<ColorMaterial>>,
) {
    for (e, children) in query.iter() {
        for child in children.iter() {
            if let Ok(mut white_hole) = white_hole_query.get_mut(*child) {
                if !white_hole.changed { continue; }
                white_hole.changed = false;
                let black_hole = black_hole_query.get(white_hole.bh).unwrap();
                if black_hole.link_type == -2 && white_hole.link_type == -2 {
                    let id = material_ids.get(black_hole.parent).unwrap();
                    let mat = mats.get(id).unwrap();
                    let input = mat.color;
                    mats.get_mut(material_ids.get(e).unwrap()).unwrap().color = input;
                    for child in children.iter() {
                        if let Ok(black_hole) = black_hole_query.get(*child) {
                            if black_hole.link_type == -2 {
                                white_hole_query.get_mut(black_hole.wh).unwrap().changed = true;
                            }
                        }
                    }
                }
            }
        }
    }
}
