//! Low-level window initialization handling

use std::error::Error;
use std::ffi::CString;
use std::num::NonZeroU32;

use glow::HasContext;
use raw_window_handle::HasRawWindowHandle;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::window::{Window, WindowBuilder};

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentContext, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::SwapInterval;

use glutin_winit::{self, DisplayBuilder, GlWindow};

pub trait App {
	fn resume_window(&mut self, gl: glow::Context);
	fn resize(&mut self, width: i32, height: i32);
	fn draw(&mut self);
	fn handle_window_event(&mut self, event: WindowEvent, window_target: &EventLoopWindowTarget<()>);
}

pub struct AppFrame {
	window: Option<Window>,
	event_loop: EventLoop<()>,
	gl_config: glutin::config::Config,
	gl_display: glutin::display::Display,
	not_current_gl_context: Option<NotCurrentContext>,
	window_builder: WindowBuilder,
}

impl AppFrame {
	pub fn init(window_builder: WindowBuilder) -> Result<Self, Box<dyn Error>> {
		let event_loop = EventLoop::new()?;
		let mut template = ConfigTemplateBuilder::new();

		// The template will match only the configurations supporting rendering
		// to windows.
		//
		// XXX We force transparency only on macOS, given that EGL on X11 doesn't
		// have it, but we still want to show window. The macOS situation is like
		// that, because we can query only one config at a time on it, but all
		// normal platforms will return multiple configs, so we can find the config
		// with transparency ourselves inside the `reduce`.
		if window_builder.transparent() {
			template = template.with_alpha_size(8).with_transparency(cfg!(cgl_backend));
		} else {
			template = template.with_transparency(false);
		}

		// Only Windows requires the window to be present before creating the display.
		// Other platforms don't really need one.
		//
		// XXX if you don't care about running on Android or so you can safely remove
		// this condition and always pass the window builder.
		let maydow_builder = {
			let wgl_backend = cfg!(all(feature = "wgl", windows, not(wasm_platform)));
			wgl_backend.then_some(window_builder.clone())
		};

		let display_builder = DisplayBuilder::new().with_window_builder(maydow_builder);

		let (window, gl_config) = display_builder.build(&event_loop, template, |configs| {
			// Find the config with the maximum number of samples, so our triangle will
			// be smooth.
			configs
				.reduce(|accum, config| {
					let config_transparent = config.supports_transparency().unwrap_or(false);
					let accum_transparent = accum.supports_transparency().unwrap_or(false);

					let transparency_check = {
						match window_builder.transparent() {
							true => config_transparent && !accum_transparent,
							false => !config_transparent && accum_transparent,
						}
					};

					if transparency_check || config.num_samples() > accum.num_samples() {
						config
					} else {
						accum
					}
				})
				.unwrap()
		})?;

		let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());

		// XXX The display could be obtained from any object created by it, so we can
		// query it from the config.
		let gl_display = gl_config.display();

		let not_current_gl_context = {
			// The context creation part. It can be created before surface and that's how
			// it's expected in multithreaded + multiwindow operation mode, since you
			// can send NotCurrentContext, but not Surface.
			let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

			// Since glutin by default tries to create OpenGL core context, which may not be
			// present we should try gles.
			let fallback_context_attributes = ContextAttributesBuilder::new()
				.with_context_api(ContextApi::Gles(None))
				.build(raw_window_handle);

			// There are also some old devices that support neither modern OpenGL nor GLES.
			// To support these we can try and create a 2.1 context.
			let legacy_context_attributes = ContextAttributesBuilder::new()
				.with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
				.build(raw_window_handle);

			Some(unsafe {
				gl_display
					.create_context(&gl_config, &context_attributes)
					.unwrap_or_else(|_| {
						gl_display
							.create_context(&gl_config, &fallback_context_attributes)
							.unwrap_or_else(|_| {
								gl_display
									.create_context(&gl_config, &legacy_context_attributes)
									.expect("failed to create context")
							})
					})
			})
		};

		Ok(Self {
			window,
			event_loop,
			gl_config,
			gl_display,
			not_current_gl_context,
			window_builder,
		})
	}

	pub fn run<A: App + 'static>(mut self, mut app: A) -> Result<(), Box<dyn Error>> {
		let mut state = None;

		self.event_loop.run(move |event, window_target| {
			match event {
				Event::Resumed => {
					#[cfg(android_platform)]
					println!("Android window available");

					let window = self.window.take().unwrap_or_else(|| {
						let window_builder = self.window_builder.clone();
						glutin_winit::finalize_window(window_target, window_builder, &self.gl_config).unwrap()
					});

					let attrs = window.build_surface_attributes(Default::default());
					let gl_surface = unsafe {
						self.gl_config
							.display()
							.create_window_surface(&self.gl_config, &attrs)
							.unwrap()
					};

					// Make it current.
					let gl_context = (self.not_current_gl_context)
						.take()
						.unwrap()
						.make_current(&gl_surface)
						.unwrap();

					// Load the OpenGL function pointers (this needs a current context in WGL)
					let mut gl = unsafe {
						glow::Context::from_loader_function(|symbol| {
							self.gl_display.get_proc_address(&CString::new(symbol).unwrap()) as *const _
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
									glow::DEBUG_SEVERITY_NOTIFICATION => tracing::debug!(target: "opengl", "{ty}{msg}"),
									glow::DEBUG_SEVERITY_LOW => tracing::info!(target: "opengl", "{ty}{msg}"),
									glow::DEBUG_SEVERITY_MEDIUM => tracing::warn!(target: "opengl", "{ty}{msg}"),
									glow::DEBUG_SEVERITY_HIGH => tracing::error!(target: "opengl", "{ty}{msg}"),
									sevr => unreachable!("unknown debug severity {sevr}"),
								};
							});

							gl.enable(glow::DEBUG_OUTPUT);
						}
					}

					app.resume_window(gl);

					// Try setting vsync.
					if let Err(res) =
						gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
					{
						eprintln!("Error setting vsync: {res:?}");
					}

					assert!(state.replace((gl_context, gl_surface, window)).is_none());
				}
				Event::Suspended => {
					// This event is only raised on Android, where the backing NativeWindow for a GL
					// Surface can appear and disappear at any moment.
					#[cfg(android_platform)]
					println!("Android window removed");

					// Destroy the GL Surface and un-current the GL Context before ndk-glue releases
					// the window back to the system.
					let (gl_context, ..) = state.take().unwrap();
					assert!(self
						.not_current_gl_context
						.replace(gl_context.make_not_current().unwrap())
						.is_none());
				}
				Event::WindowEvent { event, .. } => match event {
					WindowEvent::Resized(size) => {
						if size.width != 0 && size.height != 0 {
							// Some platforms like EGL require resizing GL surface to update the size
							// Notable platforms here are Wayland and macOS, other don't require it
							// and the function is no-op, but it's wise to resize it for portability
							// reasons.
							if let Some((gl_context, gl_surface, _)) = &state {
								gl_surface.resize(
									gl_context,
									NonZeroU32::new(size.width).unwrap(),
									NonZeroU32::new(size.height).unwrap(),
								);
								app.resize(size.width as i32, size.height as i32);
							}
						}
					}
					_ => {
						app.handle_window_event(event, window_target);
					}
				},
				Event::AboutToWait => {
					if let Some((gl_context, gl_surface, window)) = &state {
						app.draw();
						window.request_redraw();

						gl_surface.swap_buffers(gl_context).unwrap();
					}
				}
				_ => (),
			}
		})?;

		Ok(())
	}
}
