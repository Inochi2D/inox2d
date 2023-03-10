use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use inox2d::formats::inp::parse_inp;
use inox2d::{model::Model, renderers::wgpu::Renderer};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use clap::Parser;

pub async fn run(puppet: Model) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(2048, 2048))
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
        width: 2048,
        height: 2048,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: Vec::new(),
    };
    surface.configure(&device, &config);

    let mut renderer = Renderer::new(&device, &queue, wgpu::TextureFormat::Bgra8Unorm, &puppet);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
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
            _ => {}
        },
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            // Reconfigure the surface with the new size
            config.width = size.width;
            config.height = size.height;
            surface.configure(&device, &config);
            // On macos the window needs to be redrawn manually after resizing
            window.request_redraw();
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            let output = surface.get_current_texture().unwrap();
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            renderer.render(&queue, &device, &puppet, &view);
            output.present();
        }
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

    let data = {
        let file = File::open(cli.inp_path).unwrap();
        let mut file = BufReader::new(file);
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        data
    };
    let model = parse_inp(data.as_slice()).unwrap();

    pollster::block_on(run(model));
}
