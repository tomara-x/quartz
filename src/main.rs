use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanCamPlugin::default())
        .add_plugins(Shape2dPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, draw)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn the camera
    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}

fn draw(mut painter: ShapePainter) {
    // Draw a circle
    painter.circle(100.0);
}
