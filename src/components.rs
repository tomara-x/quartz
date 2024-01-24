use bevy::{
    render::{
        render_resource::PrimitiveTopology,
        mesh::Indices,
    },
    ecs::{
        system::Command,
        entity::{EntityMapper, MapEntities},
        reflect::{ReflectComponent, ReflectMapEntities},
    },
    prelude::*};
use fundsp::hacker32::*;
// -------------------- components --------------------
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Op(pub String);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Num(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Arr(pub Vec<f32>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Radius(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Vertices(pub usize);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Col(pub Color);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Selected;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Visible;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Order(pub usize);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct OpChanged(pub bool);

#[derive(Component)]
pub struct Network(pub Net32);

#[derive(Component)]
pub struct NetIns(pub Vec<Shared<f32>>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Save;

#[derive(Component)]
pub struct Highlight;

#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct WhiteHole {
    pub bh: Entity,
    pub bh_parent: Entity,
    pub link_types: (i32, i32), //(black, white)
    pub new_lt: bool,
    pub open: bool,
}
impl FromWorld for WhiteHole {
    fn from_world(_world: &mut World) -> Self {
        WhiteHole {
            bh: Entity::PLACEHOLDER,
            bh_parent: Entity::PLACEHOLDER,
            link_types: (0, 0),
            new_lt: true,
            open: true,
        }
    }
}
impl MapEntities for WhiteHole {
    fn map_entities(&mut self, entity_mapper: &mut EntityMapper) {
        self.bh = entity_mapper.get_or_reserve(self.bh);
        self.bh_parent = entity_mapper.get_or_reserve(self.bh_parent);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct BlackHole {
    pub wh: Entity,
}
impl FromWorld for BlackHole {
    fn from_world(_world: &mut World) -> Self {
        BlackHole {
            wh: Entity::PLACEHOLDER,
        }
    }
}
impl MapEntities for BlackHole {
    fn map_entities(&mut self, entity_mapper: &mut EntityMapper) {
        self.wh = entity_mapper.get_or_reserve(self.wh);
    }
}

#[derive(Component)]
pub struct CommandText;

// -------------------- states --------------------
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum Mode {
    Draw,
    Connect,
    #[default]
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
pub struct Slot(pub Slot32, pub Slot32);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct DragModes {
    pub t: bool,
    pub r: bool,
    pub n: bool,
    pub h: bool,
    pub s: bool,
    pub l: bool,
    pub a: bool,
    pub o: bool,
}
impl DragModes {
    pub fn falsify(&mut self) {
        self.t = false;
        self.r = false;
        self.n = false;
        self.h = false;
        self.s = false;
        self.l = false;
        self.a = false;
        self.o = false;
    }
}

#[derive(Resource)]
pub struct SelectionCircle(pub Entity);

#[derive(Resource)]
pub struct ConnectingLine(pub Entity);

// -------------------- events --------------------
#[derive(Event, Default)]
pub struct OrderChange;

#[derive(Event)]
pub struct ColorChange(pub Entity, pub Color);

#[derive(Event)]
pub struct RadiusChange(pub Entity, pub f32);

#[derive(Event)]
pub struct VerticesChange(pub Entity, pub usize);

#[derive(Event)]
pub struct OpChange(pub Entity, pub String);

#[derive(Event, Default)]
pub struct SceneLoaded;

// -------------------- commands --------------------
pub struct DespawnCircle(pub Entity);
impl Command for DespawnCircle {
    fn apply(self, world: &mut World) {
        despawn_circle(self.0, world);
    }
}
fn despawn_circle(entity: Entity, world: &mut World) {
    if world.get_entity(entity).is_none() { return; }
    if let Some(mirror) = get_mirror_hole(entity, world) {
        world.entity_mut(entity).despawn_recursive();
        world.entity_mut(mirror).despawn_recursive();
    } else {
        if let Some(children) = world.entity(entity).get::<Children>() {
            let children = children.to_vec();
            for child in children {
                if let Some(mirror) = get_mirror_hole(child, world) {
                    world.entity_mut(child).despawn_recursive();
                    world.entity_mut(mirror).despawn_recursive();
                }
            }
        }
        world.entity_mut(entity).despawn_recursive();
    }
}
fn get_mirror_hole(entity: Entity, world: &World) -> Option<Entity> {
    let e = world.entity(entity);
    if let Some(wh) = e.get::<WhiteHole>() {
        return Some(wh.bh);
    } else if let Some(bh) = e.get::<BlackHole>() {
        return Some(bh.wh);
    } else {
        return None;
    }
}

// -------------------- helper-functions --------------------
pub fn str_to_lt(s: &str) -> i32 {
    if let Ok(n) = s.parse::<i32>() {
        n
    } else {
        match s {
            "n" => -1,
            "r" => -2,
            "x" => -3,
            "y" => -4,
            "z" => -5,
            "h" => -6,
            "s" => -7,
            "l" => -8,
            "a" => -9,
            "ord" => -10,
            "v" => -11,
            "o" => -12,
            _ => 0,
        }
    }
}
pub fn lt_to_string(n: i32) -> String {
    match n {
        -1 => "n".to_string(),
        -2 => "r".to_string(),
        -3 => "x".to_string(),
        -4 => "y".to_string(),
        -5 => "z".to_string(),
        -6 => "h".to_string(),
        -7 => "s".to_string(),
        -8 => "l".to_string(),
        -9 => "a".to_string(),
        -10 => "ord".to_string(),
        -11 => "v".to_string(),
        -12 => "o".to_string(),
        _ => n.to_string(),
    }
}

// -------------------- tri mesh --------------------
pub struct Tri {
    pub i: Vec2,
    pub f: Vec2,
    pub ip: f32,
    pub fp: f32,
    pub b: f32,
}

impl From<Tri> for Mesh {
    fn from(tri: Tri) -> Self {
        let i = tri.f * tri.ip + tri.i * (1. - tri.ip);
        let f = tri.i * tri.fp + tri.f * (1. - tri.fp);
        let perp = (i - f).perp().normalize_or_zero() * tri.b;
        let vertices = vec!(
            [f.x, f.y, 0.0],
            [i.x + perp.x, i.y + perp.y, 0.0],
            [i.x - perp.x, i.y - perp.y, 0.0]
        );
        let indices = Indices::U32(vec![0, 1, 2]);
        Mesh::new(PrimitiveTopology::TriangleList)
            .with_indices(Some(indices))
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    }
}
