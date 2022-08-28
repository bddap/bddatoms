use bddatoms::render::Render;
use bddatoms::render_pipeline::AtomCpu;
use std::sync::Arc;
use winit::event::{KeyboardInput, VirtualKeyCode};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

async fn run(event_loop: EventLoop<()>, window: Window) {
    let window = Arc::new(window);
    let mut render = Render::create(Arc::clone(&window)).await;

    render.atom_renderer_mut().set_atoms(&[
        AtomCpu {
            pos: [0.0; 3],
            color: [0.4; 3],
            radius: 0.1,
        },
        AtomCpu {
            pos: [0.3, 0.0, 0.5],
            color: [0.15, 0.1, 0.05],
            radius: 0.3,
        },
        AtomCpu {
            pos: [-0.4, -0.3, 0.4],
            color: [0.3, 0.3, 0.1],
            radius: 0.2,
        },
        AtomCpu {
            pos: [-0.37, -0.21, 0.41],
            color: [0.15, 0.1, 0.05],
            radius: 0.17,
        },
        AtomCpu {
            pos: [-0.34, -0.27, 0.41 + 0.17],
            color: [0.1, 0.1, 0.1],
            radius: 0.03,
        },
        AtomCpu {
            pos: [-0.46, -0.27, 0.41 + 0.17],
            color: [0.1, 0.1, 0.1],
            radius: 0.03,
        },
        AtomCpu {
            pos: [0.0, 0.0, -100.0],
            color: [0.1, 0.1, 0.2],
            radius: 10.0,
        },
    ]);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            render.resize(size);
        }
        Event::RedrawRequested(_) => {
            render.frame();
        }
        Event::RedrawEventsCleared => {
            window.request_redraw();
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        }
        | Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Q),
                            ..
                        },
                    is_synthetic: false,
                    ..
                },
            ..
        } => *control_flow = ControlFlow::Exit,
        _ => {}
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize {
            width: 512,
            height: 512,
        })
        .build(&event_loop)
        .unwrap();

    env_logger::init();
    // Temporarily avoid srgb formats for the swapchain on the web
    pollster::block_on(run(event_loop, window));
}
