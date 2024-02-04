use bevy::{
    render::{
        render_resource::PrimitiveTopology,
        mesh::Indices,
    },
    ecs::{
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
pub struct NetChanged(pub bool);

#[derive(Component)]
pub struct Network(pub Net32);

#[derive(Component)]
pub struct NetIns(pub Vec<Shared<f32>>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Save;

#[derive(Component)]
pub struct Highlight(pub Entity);

#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct Targets(pub Vec<Entity>);
impl FromWorld for Targets {
    fn from_world(_world: &mut World) -> Self {
        Targets(Vec::new())
    }
}
impl MapEntities for Targets {
    fn map_entities(&mut self, entity_mapper: &mut EntityMapper) {
        for entity in &mut self.0 {
            *entity = entity_mapper.get_or_reserve(*entity);
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct WhiteHole {
    pub bh: Entity,
    pub bh_parent: Entity,
    pub link_types: (i32, i32), //(black, white)
    pub open: bool,
}
impl FromWorld for WhiteHole {
    fn from_world(_world: &mut World) -> Self {
        WhiteHole {
            bh: Entity::PLACEHOLDER,
            bh_parent: Entity::PLACEHOLDER,
            link_types: (0, 0),
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

#[derive(Component)]
pub struct ConnectionArrow(pub Entity);

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

#[derive(Component)]
pub struct InfoText(pub Entity);

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
pub struct Slot(pub Slot32);

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
    pub v: bool,
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
        self.v = false;
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
pub struct SaveCommand(pub String);

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
// an isosceles triangle defined by 2 points
pub struct Tri {
    // the midpoint of the base
    pub i: Vec2,
    // the apex
    pub f: Vec2,
    // padding/margin area (in pixles)
    // it offsets where the triangle is drawn on either side
    pub ip: f32,
    pub fp: f32,
    // half the width of the base
    pub b: f32,
}

impl From<Tri> for Mesh {
    fn from(tri: Tri) -> Self {
        let diff = (tri.f - tri.i).normalize_or_zero();
        let i = diff * tri.ip;
        let f = diff * -tri.fp;
        let perp = diff.perp() * tri.b;
        let vertices = vec!(
            [tri.f.x + f.x, tri.f.y + f.y, 0.0],
            [tri.i.x + i.x + perp.x, tri.i.y + i.y + perp.y, 0.0],
            [tri.i.x + i.x - perp.x, tri.i.y + i.y - perp.y, 0.0]
        );
        let indices = Indices::U32(vec![0, 1, 2]);
        Mesh::new(PrimitiveTopology::TriangleList)
            .with_indices(Some(indices))
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    }
}
