use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use crate::atom_renderer::AtomRenderer;

pub struct Render {
    atom_renderer: AtomRenderer,
    device: Arc<wgpu::Device>,

    surface: wgpu::Surface,
    _window: Arc<Window>, // window must outlive surface for safety

    depth_texture: wgpu::Texture,
    queue: wgpu::Queue,
    swapchain_format: wgpu::TextureFormat,
}

impl Render {
    pub async fn create(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let surface: wgpu::Surface = unsafe { instance.create_surface(window.as_ref()) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: AtomRenderer::need_features(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    limits: {
                        let mut limits = wgpu::Limits::downlevel_webgl2_defaults()
                            .using_resolution(adapter.limits());
                        limits.max_push_constant_size = 4;
                        limits
                    },
                },
                None,
            )
            .await
            .expect("Failed to create device");
        let device = Arc::new(device);

        let swapchain_format = surface.get_preferred_format(&adapter).unwrap();

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: swapchain_format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
            },
        );

        let atom_renderer = AtomRenderer::create(Arc::clone(&device), swapchain_format);

        Self {
            depth_texture: depth_buffer_with_size(size.width, size.height, &device),
            atom_renderer,
            device,
            _window: window,
            surface,
            queue,
            swapchain_format,
        }
    }

    pub fn frame(&self) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture_view = self
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass: wgpu::RenderPass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(f32::NEG_INFINITY),
                            store: false,
                        }),
                        stencil_ops: None,
                    }),
                });

            self.atom_renderer.render(&mut pass);
        }

        self.queue.submit(Some(encoder.finish()));

        frame.present();
    }

    pub fn resize(&mut self, PhysicalSize { width, height }: winit::dpi::PhysicalSize<u32>) {
        debug_assert!(width != 0 && height != 0);
        if width != 0 && height != 0 {
            self.surface.configure(
                &self.device,
                &wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: self.swapchain_format,
                    width,
                    height,
                    present_mode: wgpu::PresentMode::Mailbox,
                },
            );
            self.depth_texture = depth_buffer_with_size(width, height, &self.device);
        }
    }

    pub fn atom_renderer_mut(&mut self) -> &mut AtomRenderer {
        &mut self.atom_renderer
    }
}

fn depth_buffer_with_size(w: u32, h: u32, device: &wgpu::Device) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
    })
}
