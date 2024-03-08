use bevy::{
    prelude::*,
    render::{
        render_resource::PrimitiveTopology,
        mesh::Indices,
        render_asset::RenderAssetUsages,
    },
};

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
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_indices(indices);
        mesh
    }
}

