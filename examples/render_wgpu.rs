use glam::{uvec2, vec2, Vec2};
use scene::ExampleSceneController;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use inox2d::formats::inp::parse_inp;
use inox2d::{model::Model, render::wgpu::Renderer};
use std::{fs, env};
use std::path::PathBuf;

use clap::Parser;

#[path = "./common/scene.rs"]
mod scene;

pub async fn run(model: Model) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 800))
        .with_resizable(true)
        .with_transparent(true)
        .with_title("Render Inochi2D Puppet (WGPU)")
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER,
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await
        .unwrap();

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::PreMultiplied,
        view_formats: Vec::new(),
    };
    surface.configure(&device, &config);

    let mut renderer = Renderer::new(
        &device,
        &queue,
        wgpu::TextureFormat::Bgra8Unorm,
        &model,
        uvec2(window.inner_size().width, window.inner_size().height),
    );
    renderer.camera.scale = Vec2::splat(0.15);

    let mut scene_ctrl = ExampleSceneController::new(&renderer.camera, 0.5);
    let mut puppet = model.puppet;

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            scene_ctrl.update(&mut renderer.camera);

            puppet.begin_set_params();
            let t = scene_ctrl.current_elapsed();
            puppet.set_param("Head:: Yaw-Pitch", vec2(t.cos(), t.sin()));
            puppet.end_set_params();

            let output = surface.get_current_texture().unwrap();
            let view = (output.texture).create_view(&wgpu::TextureViewDescriptor::default());

            renderer.render(&queue, &device, &puppet, &view);
            output.present();
        }
        Event::WindowEvent { ref event, .. } => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);

                // Update the renderer's internal viewport
                renderer.resize(uvec2(size.width, size.height));

                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            _ => scene_ctrl.interact(&window, event, &renderer.camera),
        },
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        _ => {}
    });
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(help = "Path to the .inp file. .inx files don't work!")]
    inp_path: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let data = fs::read(cli.inp_path).unwrap();
    let model = parse_inp(data.as_slice()).unwrap();

    pollster::block_on(run(model));
}
