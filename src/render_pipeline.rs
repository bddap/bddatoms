use shame::prelude::*;

pub type VertexCpu = [f32; 3];

pub type VertexGpu = float3;

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct AtomCpu {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub radius: f32,
}

#[derive(shame::Fields)]
struct AtomGpu {
    pos: float3,
    color: float3,
    radius: float,
}

pub fn pipeline(mut f: RenderFeatures) {
    let index: TriangleStrip<u32> = f.io.index_buffer();

    let vertex: VertexGpu = f.io.vertex_buffer();
    let atom: AtomGpu = f.io.instance_buffer();

    let clip_position = ((vertex.xyz() * atom.radius) + atom.pos, 1.0);
    let poly = f.raster.rasterize(clip_position, Cull::CW, index);

    let uv = poly.lerp(vertex.xy());

    let distance_to_center_squared = uv.dot(uv);
    distance_to_center_squared
        .gt(&1.0)
        .then(|| Any::discard_fragment());
    // if distance_to_center_squared > 1f {
    //     discard;
    // }

    let dr = (1.0 - distance_to_center_squared).sqrt();
    let hit_normal = (uv, dr).as_ten().normalize();

    // the direction to the light source
    // not the direction light is traveling
    let light_direction = (1.0, -2.0, 3.0).as_ten().normalize();

    let intensity = 0.2 + light_direction.dot(hit_normal).max(0.0) * 2.0;

    let depth = dr * poly.lerp(atom.radius) + poly.lerp(clip_position.as_ten().z());
    f.io.depth::<Depth32>()
        .test_write(DepthTest::Greater, DepthWrite::Write(depth));

    let color = poly.lerp(atom.color) * intensity;
    f.io.color::<RGBA_Surface>().set((color, 0.0));
}
