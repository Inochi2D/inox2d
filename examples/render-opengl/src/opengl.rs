use std::env;
use std::error::Error;
use std::ffi::CString;
use std::num::NonZeroU32;

use glow::HasContext;
use glutin::config::Config;
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext, Version};
use glutin::display::{Display, GetGlDisplay};
use glutin::prelude::{GlConfig, GlDisplay};
use glutin::surface::{Surface, SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::ApiPreference;
use raw_window_handle::HasRawWindowHandle;
use tracing::{debug, error, info, warn};
use winit::event_loop::EventLoop;
use winit::window::Window;

pub struct App {
	pub gl: glow::Context,
	pub gl_ctx: PossiblyCurrentContext,
	pub gl_surface: Surface<WindowSurface>,
	pub gl_display: Display,
	pub window: Window,
	pub events: EventLoop<()>,
}

pub fn launch_opengl_window() -> Result<App, Box<dyn Error>> {
	if cfg!(target_os = "linux") {
		// disables vsync sometimes on x11
		if env::var("vblank_mode").is_err() {
			env::set_var("vblank_mode", "0");
		}
	}

	let events = winit::event_loop::EventLoop::new().unwrap();

	let window_builder = winit::window::WindowBuilder::new()
		.with_transparent(true)
		.with_resizable(true)
		.with_inner_size(winit::dpi::PhysicalSize::new(600, 800))
		.with_title("Render Inochi2D Puppet (OpenGL)");

	let (window, gl_config) = glutin_winit::DisplayBuilder::new()
		.with_preference(ApiPreference::FallbackEgl)
		.with_window_builder(Some(window_builder))
		.build(&events, <_>::default(), |configs| {
			configs
				.filter(|c| match c {
					Config::Egl(c) => c.srgb_capable(),
					_ => false,
				})
				.max_by_key(|c| match c {
					Config::Egl(c) => c.num_samples(),
					_ => 0,
				})
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
		let context = unsafe { gl_display.create_context(&gl_config, &context_attributes)? }.make_current(&surface)?;
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
