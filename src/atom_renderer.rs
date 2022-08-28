use std::sync::Arc;

use wgpu::IndexFormat;

use crate::{
    glue,
    gpubuf::GpuBuf,
    render_pipeline::{self, AtomCpu, VertexCpu},
};

pub struct AtomRenderer {
    index_buf: GpuBuf<u32>,
    vertex_buf: GpuBuf<VertexCpu>,
    instance_buf: GpuBuf<AtomCpu>,
    render_pipeline: wgpu::RenderPipeline,
}

impl AtomRenderer {
    pub fn create(device: Arc<wgpu::Device>, swapchain_format: wgpu::TextureFormat) -> Self {
        let recording = shame::record_render_pipeline(render_pipeline::pipeline);
        let render_pipeline =
            glue::make_render_pipeline(&recording, &device, Some(swapchain_format));

        let mut vertex_buf = GpuBuf::create(Arc::clone(&device), 4, wgpu::BufferUsages::VERTEX);
        vertex_buf.set_from_slice(&[
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
        ]);
        let mut index_buf = GpuBuf::create(Arc::clone(&device), 5, wgpu::BufferUsages::INDEX);
        index_buf.set_from_slice(&[0, 1, 2, 3, 0]);

        let instance_buf = GpuBuf::create(device, 1, wgpu::BufferUsages::VERTEX);

        Self {
            vertex_buf,
            index_buf,
            instance_buf,
            render_pipeline,
        }
    }

    pub fn set_atoms(&mut self, atoms: &[AtomCpu]) {
        self.instance_buf.set_from_slice(atoms);
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

        pass.draw_indexed(
            0..(self.index_buf.len() as u32),
            0,
            0..(self.instance_buf.len() as u32),
        );
    }

    pub fn need_features() -> wgpu::Features {
        wgpu::Features::DEPTH_CLIP_CONTROL
    }
}
