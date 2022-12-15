use std::{
    env,
    error::Error,
    ffi::CString,
    fs::File,
    io::{BufReader, Read},
    num::NonZeroU32,
};

use glow::HasContext;

use glutin::{
    context::PossiblyCurrentContext,
    display::Display,
    display::GetGlDisplay,
    prelude::{GlConfig, GlDisplay, NotCurrentGlContextSurfaceAccessor},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};

use inox2d::{parsers::inp::parse_inp, renderers::opengl::OpenglRenderer};
use raw_window_handle::HasRawWindowHandle;

use tracing::{debug, error, info, warn};

use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*};

use winit::{
    event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(help = "Path to the .inp file. .inx files don't work!")]
    inp_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(LevelFilter::DEBUG)
        .init();

    info!("Parsing puppet");
    let data = {
        let file = File::open(&cli.inp_path).unwrap();
        let mut file = BufReader::new(file);
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        data
    };
    let model = parse_inp(&data).unwrap().1;
    let puppet = model.puppet;
    info!(
        "Successfully parsed puppet {:?}",
        puppet.meta.name.unwrap_or_default()
    );

    info!("Setting up windowing and OpenGL");
    let App {
        gl,
        gl_ctx,
        gl_surface,
        gl_display,
        events,
        window,
    } = launch_opengl_window()?;

    let renderer = OpenglRenderer::new(gl, puppet.nodes, model.textures);
    let zsorted_nodes = renderer.nodes.zsorted();

    // Event loop
    events.run(move |event, _, control_flow| {
        // They need to be present
        let _gl_display = &gl_display;
        let _window = &window;

        control_flow.set_wait();

        match event {
            Event::NewEvents(StartCause::Poll) | Event::RedrawRequested(_) => {
                debug!("Redrawing");

                renderer.clear();
                renderer.render_nodes(&zsorted_nodes);

                gl_surface.swap_buffers(&gl_ctx).unwrap();
                // window.request_redraw();
            }
            _ => handle_event(event, control_flow, &renderer.gl, &gl_surface, &gl_ctx),
        }
    })
}

struct App {
    pub gl: glow::Context,
    pub gl_ctx: PossiblyCurrentContext,
    pub gl_surface: Surface<WindowSurface>,
    pub gl_display: Display,
    pub window: Window,
    pub events: EventLoop<()>,
}

fn launch_opengl_window() -> Result<App, Box<dyn Error>> {
    if cfg!(target_os = "linux") {
        // disables vsync sometimes on x11
        if env::var("vblank_mode").is_err() {
            env::set_var("vblank_mode", "0");
        }
    }

    let events = winit::event_loop::EventLoop::new();

    let window_builder = winit::window::WindowBuilder::new()
        .with_transparent(true)
        .with_inner_size(winit::dpi::PhysicalSize::new(2048, 2048))
        .with_title("Render Inochi2D Puppet");

    let (window, gl_config) = glutin_winit::DisplayBuilder::new()
        .with_window_builder(Some(window_builder))
        .build(&events, <_>::default(), |configs| {
            configs
                .filter(|c| c.srgb_capable())
                .max_by_key(|c| c.num_samples())
                .unwrap()
        })?;

    let window = window.unwrap(); // set in display builder
    let raw_window_handle = window.raw_window_handle();
    let gl_display = gl_config.display();

    let context_attributes = glutin::context::ContextAttributesBuilder::new()
        .with_profile(glutin::context::GlProfile::Compatibility)
        .with_context_api(glutin::context::ContextApi::Gles(Some(
            glutin::context::Version::new(2, 0),
        )))
        .build(Some(raw_window_handle));

    let dimensions = window.inner_size();

    let (gl_surface, gl_ctx) = {
        let attrs = SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(dimensions.width).unwrap(),
            NonZeroU32::new(dimensions.height).unwrap(),
        );

        let surface = unsafe { gl_display.create_window_surface(&gl_config, &attrs)? };
        let context = unsafe { gl_display.create_context(&gl_config, &context_attributes)? }
            .make_current(&surface)?;
        (surface, context)
    };

    // Load the OpenGL function pointers
    let gl = unsafe {
        glow::Context::from_loader_function(|symbol| {
            gl_display.get_proc_address(&CString::new(symbol).unwrap()) as *const _
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
        gl,
        gl_ctx,
        gl_surface,
        gl_display,
        window,
        events,
    })
}

fn handle_event(
    event: Event<()>,
    control_flow: &mut ControlFlow,
    _gl: &glow::Context,
    gl_surface: &Surface<WindowSurface>,
    gl_ctx: &PossiblyCurrentContext,
) {
    match event {
        Event::LoopDestroyed => (),
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::Resized(physical_size) => {
                // Handle window resizing
                gl_surface.resize(
                    gl_ctx,
                    NonZeroU32::new(physical_size.width).unwrap(),
                    NonZeroU32::new(physical_size.height).unwrap(),
                );
            }
            WindowEvent::CloseRequested => control_flow.set_exit(),
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                info!("There is an Escape D:");
                control_flow.set_exit();
            }
            _ => (),
        },
        _ => (),
    }
}
