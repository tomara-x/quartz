use bevy::{prelude::*};
use fundsp::hacker32::*;
// -------------------- components --------------------
#[derive(Component)]
pub struct Op(pub i32);

#[derive(Component)]
pub struct Num(pub f32);

#[derive(Component)]
pub struct Arr(pub Vec<f32>);

#[derive(Component)]
pub struct Offset {
    pub trans: Vec3,
    pub color: Color,
    pub radius: f32,
}

#[derive(Component)]
pub struct Radius(pub f32);

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Visible;

#[derive(Component)]
pub struct Order(pub usize);

#[derive(Component)]
pub struct OpChanged(pub bool);

#[derive(Component)]
pub struct Network(pub Net32);

#[derive(Component)]
pub struct NetIns(pub Vec<Shared<f32>>);

// `new`?
// `output_changed` and `link_type_changed` ? too much typing tho
// leave changed as is, change `new` to `new_lt` ?
// hole enum? (what on earth was on your mind, weeks-ago-amy?)
// TODO(amy): clean this mess
#[allow(dead_code)]
#[derive(Component)]
pub struct WhiteHole {
    pub id: Entity,
    pub parent: Entity,
    pub bh: Entity,
    pub link_type: i32,
    pub changed: bool,
    pub new: bool,
}

#[derive(Component)]
pub struct BlackHole {
    pub id: Entity,
    pub parent: Entity,
    pub wh: Entity,
    pub link_type: i32,
    pub new: bool,
}

// -------------------- states --------------------
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum Mode {
    #[default]
    Draw,
    Connect,
    Edit,
}

// -------------------- resources --------------------
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Queue(pub Vec<Vec<Entity>>);

// initial, final, delta
#[derive(Resource, Default)]
pub struct CursorInfo {
    pub i: Vec2,
    pub f: Vec2,
    pub d: Vec2,
}

#[derive(Resource)]
pub struct Depth(pub f32);

#[derive(Resource)]
pub struct Slot(pub Slot32, pub Slot32);

// -------------------- events --------------------
#[derive(Event, Default)]
pub struct OrderChange;


