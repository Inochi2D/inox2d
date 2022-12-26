use glutin::{
    config::{ConfigSurfaceTypes, ConfigTemplateBuilder},
    context::{ContextApi, ContextAttributesBuilder},
    display::DisplayApiPreference::Egl,
};
use std::{env, ffi::CString, num::NonZeroU32};

use glow::HasContext;

use glutin::{
    context::PossiblyCurrentContext,
    display::Display,
    prelude::{GlConfig, GlDisplay, NotCurrentGlContextSurfaceAccessor},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};

use crate::{
    model::ModelTexture,
    nodes::node_tree::ExtInoxNodeTree,
    renderers::opengl::opengl_renderer_ext,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use tracing::{debug, error, info, warn};

use winit::event::{Event, WindowEvent};

use super::{CustomRenderer, ExtOpenglRenderer};

pub struct App<T, R>
where
    R: CustomRenderer<NodeData = T>,
{
    pub gl_ctx: PossiblyCurrentContext,
    pub surface: Surface<WindowSurface>,
    pub display: Display,
    pub renderer: ExtOpenglRenderer<T, R>,
}

impl<T, R> super::super::App for App<T, R>
where
    R: CustomRenderer<NodeData = T>,
{
    fn update(&self, event: winit::event::Event<()>) {
        match event {
            Event::LoopDestroyed => (),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    // Handle window resizing
                    self.surface.resize(
                        &self.gl_ctx,
                        NonZeroU32::new(physical_size.width).unwrap(),
                        NonZeroU32::new(physical_size.height).unwrap(),
                    );
                }

                _ => (),
            },
            _ => (),
        }
    }
    type Error = glutin::error::Error;
    type NodeData = T;
    type CustomRenderer = R;
    fn launch(
        window: &winit::window::Window,
        nodes: ExtInoxNodeTree<Self::NodeData>,
        textures: Vec<ModelTexture>,
        custom_renderer: R,
    ) -> Result<Self, Self::Error> {
        if cfg!(target_os = "linux") {
            // disables vsync sometimes on x11
            if env::var("vblank_mode").is_err() {
                env::set_var("vblank_mode", "0");
            }
        }

        let display = unsafe { Display::new(window.raw_display_handle(), Egl)? };
        let template = ConfigTemplateBuilder::default()
            .with_alpha_size(8)
            .with_surface_type(ConfigSurfaceTypes::WINDOW)
            .build();
        let config = unsafe { display.find_configs(template) }
            .unwrap()
            .reduce(|config, acc| {
                if config.num_samples() > acc.num_samples() {
                    config
                } else {
                    acc
                }
            })
            .expect("No available configs");

        println!("Picked a config with {} samples", config.num_samples());

        // Context creation.
        //
        // In particular, since we are doing offscreen rendering we have no raw window
        // handle to provide.
        let context_attributes = ContextAttributesBuilder::new().build(None);

        // Since glutin by default tries to create OpenGL core context, which may not be
        // present we should try gles.
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(None);

        let not_current = unsafe {
            display
                .create_context(&config, &context_attributes)
                .unwrap_or_else(|_| {
                    display
                        .create_context(&config, &fallback_context_attributes)
                        .expect("failed to create context")
                })
        };

        let raw_window_handle = window.raw_window_handle();

        let context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_profile(glutin::context::GlProfile::Compatibility)
            .with_context_api(glutin::context::ContextApi::Gles(Some(
                glutin::context::Version::new(2, 0),
            )))
            .build(Some(raw_window_handle));

        let dimensions = window.inner_size();

        let (surface, gl_ctx) = {
            let attrs = SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new().build(
                raw_window_handle,
                NonZeroU32::new(dimensions.width).unwrap(),
                NonZeroU32::new(dimensions.height).unwrap(),
            );

            let surface = unsafe { display.create_window_surface(&config, &attrs)? };
            let context = unsafe { display.create_context(&config, &context_attributes)? }
                .make_current(&surface)?;
            (surface, context)
        };

        // Load the OpenGL function pointers
        let gl = unsafe {
            glow::Context::from_loader_function(|symbol| {
                display.get_proc_address(&CString::new(symbol).unwrap()) as *const _
            })
        };

        unsafe {
            gl.debug_message_callback(|_src, ty, _id, sevr, msg| {
                let ty = match ty {
                    glow::DEBUG_TYPE_ERROR => "Error: ",
                    glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior: ",
                    glow::DEBUG_TYPE_MARKER => "Marker: ",
                    glow::DEBUG_TYPE_OTHER => "",
                    glow::DEBUG_TYPE_POP_GROUP => "Pop Group: ",
                    glow::DEBUG_TYPE_PORTABILITY => "Portability: ",
                    glow::DEBUG_TYPE_PUSH_GROUP => "Push Group: ",
                    glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined Behavior: ",
                    glow::DEBUG_TYPE_PERFORMANCE => "Performance: ",
                    ty => unreachable!("unknown debug type {ty}"),
                };
                match sevr {
                    glow::DEBUG_SEVERITY_NOTIFICATION => debug!(target: "opengl", "{ty}{msg}"),
                    glow::DEBUG_SEVERITY_LOW => info!(target: "opengl", "{ty}{msg}"),
                    glow::DEBUG_SEVERITY_MEDIUM => warn!(target: "opengl", "{ty}{msg}"),
                    glow::DEBUG_SEVERITY_HIGH => error!(target: "opengl", "{ty}{msg}"),
                    sevr => unreachable!("unknown debug severity {sevr}"),
                };
            });

            gl.enable(glow::DEBUG_OUTPUT);
        }

        unsafe { gl.viewport(0, 0, 2048, 2048) };

        Ok(App {
            gl_ctx,
            surface,
            display,
            renderer: opengl_renderer_ext(gl, nodes, textures, custom_renderer),
        })
    }
}
