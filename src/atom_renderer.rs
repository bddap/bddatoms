use std::sync::Arc;

use glam::Mat4;
use wgpu::IndexFormat;

use crate::{
    glue,
    gpubuf::GpuBuf,
    render_pipeline::{self, AtomCpu, UniformCpu, VertexCpu},
};

pub struct AtomRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    index_buf: GpuBuf<u32>,
    vertex_buf: GpuBuf<VertexCpu>,
    instance_buf: GpuBuf<AtomCpu>,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: GpuBuf<UniformCpu>,
}

impl AtomRenderer {
    pub fn create(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        swapchain_format: wgpu::TextureFormat,
    ) -> Self {
        let recording = shame::record_render_pipeline(render_pipeline::pipeline);
        let (render_pipeline, bind_group_layouts) =
            glue::make_render_pipeline(&recording, &device, Some(swapchain_format));

        let vertex_buf = GpuBuf::initialize(
            Arc::clone(&device),
            Arc::clone(&queue),
            &[
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [1.0, 1.0, 0.0],
                [-1.0, 1.0, 0.0],
            ],
            wgpu::BufferUsages::VERTEX,
        );
        let index_buf = GpuBuf::initialize(
            Arc::clone(&device),
            Arc::clone(&queue),
            &[0, 1, 2, 3, 0],
            wgpu::BufferUsages::INDEX,
        );

        let instance_buf = GpuBuf::initialize(
            Arc::clone(&device),
            Arc::clone(&queue),
            &[],
            wgpu::BufferUsages::VERTEX,
        );

        let transform = Mat4::from_cols_array_2d(&[
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        let uniform_buf = GpuBuf::initialize(
            Arc::clone(&device),
            Arc::clone(&queue),
            &[transform],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );

        assert_eq!(bind_group_layouts.len(), 1);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layouts[0],
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buf.as_entire_buffer_binding()),
            }],
            label: None,
        });

        Self {
            vertex_buf,
            index_buf,
            instance_buf,
            render_pipeline,
            bind_group,
            uniform_buf,
            queue,
            device,
        }
    }

    pub fn set_atoms(&mut self, atoms: &[AtomCpu]) {
        self.instance_buf = GpuBuf::initialize(
            Arc::clone(&self.device),
            Arc::clone(&self.queue),
            &atoms,
            wgpu::BufferUsages::VERTEX,
        );
    }

    pub fn set_transform(&mut self, transform: Mat4) {
        self.uniform_buf.copy_from_slice(&[transform]);
    }

    /// write render commands to the command buffer
    // TODO: consider passing a typed buffer into this function
    pub fn render<'a: 'b, 'b>(&'a self, pass: &mut wgpu::RenderPass<'b>) {
        let instance_slice = if let Some(i) = self.instance_buf.slice() {
            i
        } else {
            return;
        };

        pass.set_pipeline(&self.render_pipeline);

        pass.set_index_buffer(self.index_buf.slice().unwrap(), IndexFormat::Uint32);
        pass.set_vertex_buffer(0, self.vertex_buf.slice().unwrap());
        pass.set_vertex_buffer(1, instance_slice);
        pass.set_bind_group(0, &self.bind_group, &[]);

        pass.draw_indexed(
            0..(self.index_buf.len() as u32),
            0,
            0..(self.instance_buf.len() as u32),
        );
    }

    pub fn need_features() -> wgpu::Features {
        render_pipeline::features_used()
    }
}
