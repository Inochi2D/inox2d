use std::{
    env,
    error::Error,
    ffi::CString,
    fs::File,
    io::{BufReader, Read},
    num::NonZeroU32,
    time::Instant,
};

use glam::{uvec2, vec2, Vec2};
use glow::HasContext;

use glutin::{
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version},
    display::Display,
    display::GetGlDisplay,
    prelude::{GlConfig, GlDisplay, NotCurrentGlContextSurfaceAccessor},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};

use glutin_winit::ApiPrefence;
use inox2d::{formats::inp::parse_inp, renderers::opengl::OpenglRenderer};
use raw_window_handle::HasRawWindowHandle;

use tracing::{debug, error, info, warn};

use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*};

use winit::{
    event::{ElementState, Event, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent},
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
        .with(LevelFilter::INFO)
        .init();

    info!("Parsing puppet");
    let data = {
        let file = File::open(cli.inp_path).unwrap();
        let mut file = BufReader::new(file);
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        data
    };
    let model = parse_inp(data.as_slice()).unwrap();
    let puppet = model.puppet;
    info!(
        "Successfully parsed puppet: {}",
        puppet
            .meta
            .name
            .as_deref()
            .unwrap_or("<no puppet name specified in file>")
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

    info!("Initializing Inox2D renderer");
    let window_size = window.inner_size();
    let viewport = uvec2(window_size.width, window_size.height);
    let mut renderer = OpenglRenderer::new(gl, viewport, &puppet)?;
    renderer.upload_model_textures(&model.textures)?;
    renderer.camera.scale = Vec2::splat(0.15);
    info!("Inox2D renderer initialized");

    // variables and state for camera position and mouse interactions
    let mut camera_pos = Vec2::ZERO;
    let mut mouse_pos = Vec2::ZERO;
    let mut mouse_pos_held = mouse_pos;
    let mut mouse_state = ElementState::Released;

    // variables and state for smooth scrolling
    let scroll_speed: f32 = 3.0;
    let mut hard_scale = renderer.camera.scale;

    // variables and state for FPS-independent interactions
    let start = Instant::now();
    let mut prev_elapsed: f32 = 0.0;
    let mut current_elapsed: f32 = 0.0;

    let mut puppet = puppet;

    // Event loop
    events.run(move |event, _, control_flow| {
        // They need to be present
        let _gl_display = &gl_display;
        let _window = &window;

        control_flow.set_wait();

        match event {
            Event::RedrawRequested(_) => {
                debug!("Redrawing");

                let time_delta = current_elapsed - prev_elapsed;
                renderer.camera.scale = renderer.camera.scale
                    + time_delta.powf(0.6) * (hard_scale - renderer.camera.scale);

                renderer.clear();

                puppet.begin_set_params();
                let t = current_elapsed;
                puppet.set_param("Head:: Yaw-Pitch", Vec2::new(t.cos(), t.sin()));
                puppet.end_set_params();

                renderer.render(&puppet);

                gl_surface.swap_buffers(&gl_ctx).unwrap();
                window.request_redraw();

                prev_elapsed = current_elapsed;
                current_elapsed = start.elapsed().as_secs_f32();
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    // Handle window resizing
                    renderer.resize(physical_size.width, physical_size.height);
                    gl_surface.resize(
                        &gl_ctx,
                        NonZeroU32::new(physical_size.width).unwrap(),
                        NonZeroU32::new(physical_size.height).unwrap(),
                    );
                    window.request_redraw();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    mouse_pos = vec2(position.x as f32, position.y as f32);
                    if mouse_state == ElementState::Pressed {
                        renderer.camera.position =
                            camera_pos + (mouse_pos - mouse_pos_held) / renderer.camera.scale;

                        window.request_redraw();
                    }
                }
                WindowEvent::MouseInput { state, .. } => {
                    mouse_state = *state;
                    if mouse_state == ElementState::Pressed {
                        mouse_pos_held = mouse_pos;
                        camera_pos = renderer.camera.position;
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    // Handle mouse wheel (zoom)
                    let my = match delta {
                        MouseScrollDelta::LineDelta(_, y) => *y * 12.,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };

                    let time_delta = current_elapsed - prev_elapsed;
                    hard_scale *= 10_f32.powf(scroll_speed * time_delta * my);

                    window.request_redraw();
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }

        handle_close(event, control_flow);
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
        .with_resizable(true)
        .with_inner_size(winit::dpi::PhysicalSize::new(600, 800))
        .with_title("Render Inochi2D Puppet");

    let (window, gl_config) = glutin_winit::DisplayBuilder::new()
        .with_preference(ApiPrefence::FallbackEgl)
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

    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 1))))
        .with_profile(glutin::context::GlProfile::Core)
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

    // Check for "GL_KHR_debug" support (not present on Apple *OS).
    if gl.supported_extensions().contains("GL_KHR_debug") {
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
    }

    Ok(App {
        gl,
        gl_ctx,
        gl_surface,
        gl_display,
        window,
        events,
    })
}

fn handle_close(event: Event<()>, control_flow: &mut ControlFlow) {
    if let Event::WindowEvent { event, .. } = event {
        match event {
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
        }
    }
}
