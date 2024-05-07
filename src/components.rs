use bevy::{
    prelude::*,
    sprite::Mesh2dHandle,
    ecs::{
        entity::MapEntities,
        reflect::{ReflectComponent, ReflectMapEntities},
    },
};

use fundsp::{
    net::Net32,
    shared::Shared,
    slot::Slot32,
};

use copypasta::ClipboardContext;

// -------------------- components --------------------
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Op(pub String);

#[derive(Component, Reflect, Default, PartialEq)]
#[reflect(Component)]
pub struct Number(pub f32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Arr(pub Vec<f32>);

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
pub struct Order(pub usize);

#[derive(Component)]
pub struct OpChanged(pub bool);

#[derive(Component)]
pub struct Network(pub Net32);

#[derive(Component)]
pub struct NetIns(pub Vec<Shared<f32>>);

#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct Holes(pub Vec<Entity>);
impl FromWorld for Holes {
    fn from_world(_world: &mut World) -> Self {
        Holes(Vec::new())
    }
}
impl MapEntities for Holes {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        for entity in &mut self.0 {
            *entity = entity_mapper.map_entity(*entity);
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct GainedWH(pub bool);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct LostWH(pub bool);

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
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        for entity in &mut self.0 {
            *entity = entity_mapper.map_entity(*entity);
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct WhiteHole {
    pub bh: Entity,
    pub bh_parent: Entity,
    pub link_types: (i8, i8), //(black, white)
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
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.bh = entity_mapper.map_entity(self.bh);
        self.bh_parent = entity_mapper.map_entity(self.bh_parent);
    }
}

#[derive(Component)]
pub struct ConnectionArrow(pub Entity);

#[derive(Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct BlackHole {
    pub wh: Entity,
    pub wh_parent: Entity,
}
impl FromWorld for BlackHole {
    fn from_world(_world: &mut World) -> Self {
        BlackHole {
            wh: Entity::PLACEHOLDER,
            wh_parent: Entity::PLACEHOLDER,
        }
    }
}
impl MapEntities for BlackHole {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.wh = entity_mapper.map_entity(self.wh);
        self.wh_parent = entity_mapper.map_entity(self.wh_parent);
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

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct LoopQueue(pub Vec<Entity>);

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

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct DefaultDrawColor(pub Color);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct DefaultDrawVerts(pub usize);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct HighlightColor(pub Color);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ConnectionColor(pub Color);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct CommandColor(pub Color);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ConnectionWidth(pub f32);

#[derive(Resource)]
pub struct DefaultLT(pub (i8, i8));

#[derive(Resource)]
pub struct SystemClipboard(pub ClipboardContext);

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Version(pub String);

#[derive(Resource, Default)]
pub struct PolygonHandles(pub Vec<Option<Mesh2dHandle>>);

#[derive(Resource)]
pub struct ArrowHandle(pub Mesh2dHandle);

#[derive(Resource, Default)]
pub struct DacCircles(pub Vec<Entity>);

#[derive(Resource)]
pub struct ConnectionMat(pub Handle<ColorMaterial>);

// -------------------- events --------------------
#[derive(Event, Default)]
pub struct OrderChange;

#[derive(Event)]
pub struct SaveCommand(pub String);

#[derive(Event, Default)]
pub struct CopyCommand;

#[derive(Event, Default)]
pub struct PasteCommand;

#[derive(Event, Default)]
pub struct DeleteCommand;

#[derive(Event, Default)]
pub struct DacChange;

#[derive(Event)]
pub struct ConnectCommand(pub Entity);
