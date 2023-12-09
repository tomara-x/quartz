use bevy::{prelude::*};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorInfo {i: Vec2::ZERO, f: Vec2::ZERO});
        app.add_systems(Update, update_cursor_info);
    }
}

#[derive(Resource)]
pub struct CursorInfo {
    pub i: Vec2,
    pub f: Vec2,
}

fn update_cursor_info(
    mouse_button_input: Res<Input<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut cursor: ResMut<CursorInfo>,
) {
    let (cam, cam_transform) = camera_query.single();
    if mouse_button_input.pressed(MouseButton::Left) {
        let Some(cursor_pos) = windows.single().cursor_position() else { return; };
        let Some(point) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };
        cursor.f = point;
    }
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let Some(cursor_pos) = windows.single().cursor_position() else { return; };
        let Some(point) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };
        cursor.i = point;
    }
}

