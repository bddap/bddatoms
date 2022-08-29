use shame::prelude::*;

pub type VertexCpu = [f32; 3];

type VertexGpu = float3;

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

pub type UniformCpu = glam::Mat4;
type UniformGpu = float4x4;

pub fn features_used() -> wgpu::Features {
    wgpu::Features::PUSH_CONSTANTS | wgpu::Features::DEPTH_CLIP_CONTROL
}

pub fn pipeline(mut f: RenderFeatures) {
    let index: TriangleStrip<u32> = f.io.index_buffer();

    let vertex: VertexGpu = f.io.vertex_buffer();
    let atom: AtomGpu = f.io.instance_buffer();
    let transform: UniformGpu = f.io.group().uniform_block();

    let pos = transform * (atom.pos, 1.0);

    let clip_position = pos + (vertex.xy() * atom.radius, 0.0, 0.0);
    let poly = f.raster.rasterize(clip_position, Cull::Off, index);

    let uv = poly.lerp(vertex.xy());

    let distance_to_center_squared = uv.dot(uv);
    distance_to_center_squared
        .gt(&1.0)
        .then(|| Any::discard_fragment());

    let dr = (1.0 - distance_to_center_squared).sqrt();
    let hit_normal = (uv, dr).rec().normalize();

    // the direction to the light source
    // not the direction light is traveling
    let light_direction = (1.0, -2.0, 3.0, 0.0);
    let light_direction = transform * light_direction;

    let light_direction = transform * light_direction;
    let lighta_intensity = 0.2 + light_direction.xyz().normalize().dot(hit_normal).max(0.0) * 2.0;
    let lighta_color = (0.2, 0.1, 0.3);
    let lighta = lighta_intensity * lighta_color;

    let light_direction = transform * light_direction;
    let lightb_intensity = 0.2 + light_direction.xyz().normalize().dot(hit_normal).max(0.0) * 2.0;
    let lightb_color = (0.2, 0.3, 0.1);
    let lightb = lightb_intensity * lightb_color;

    let radius = poly.lerp(atom.radius);
    let base_distance = poly.lerp(clip_position.rec().z());

    let depth = dr * radius + base_distance;

    f.io.depth::<Depth32>()
        .test_write(DepthTest::Greater, DepthWrite::Write(depth));

    let color = poly.lerp(atom.color);
    let color = color * lighta + color * lightb;
    f.io.color::<RGBA_Surface>().set((color, 0.0));
}
