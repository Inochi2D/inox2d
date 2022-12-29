use futures::executor::block_on;
use wgpu::{Device, Queue, Surface};

use crate::{
    model::ModelTexture, nodes::node_tree::ExtInoxNodeTree,
    renderers::opengl::DefaultCustomRenderer,
};

use super::{ext_wgpu_renderer, CustomRenderer, ExtWgpuRenderer};
pub struct App<T, R>
where
    R: CustomRenderer<NodeData = T>,
{
    pub surface: Surface,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub renderer: ExtWgpuRenderer<T, R>,
}

impl<T, R: CustomRenderer<NodeData = T>> App<T, R> {
    pub fn update(&self, event: winit::event::Event<()>) {
        todo!()
    }

    pub fn launch(
        window: &winit::window::Window,
        nodes: ExtInoxNodeTree<T>,
        textures: Vec<crate::model::ModelTexture>,
        custom_renderer: R,
    ) -> Result<Self, wgpu::Error>
    where
        Self: Sized,
    {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let surface = unsafe { instance.create_surface(&window) };

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();
        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        ))
        .unwrap();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        Ok(Self {
            surface,

            size,
            renderer: ext_wgpu_renderer(device, queue, config, nodes, textures, custom_renderer),
        })
    }
}
