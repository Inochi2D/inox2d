use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
    num::NonZeroU32,
};

use glutin::surface::GlSurface;
use inox2d::{
    formats::inp::parse_inp,
    renderers::{
        opengl::opengl_app,
        App,
    },
};

use raw_window_handle::HasRawWindowHandle;

use tracing::{debug, error, info, warn};

use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*};

use winit::{
    event::{ElementState, Event, KeyboardInput, StartCause},
    window::WindowBuilder,
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
        let file = File::open(cli.inp_path).unwrap();
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
    let events = winit::event_loop::EventLoop::new();

    let window = WindowBuilder::new().build(&events).unwrap();
    let app = opengl_app(&window, puppet.nodes, model.textures).unwrap();

    let zsorted_nodes = app.renderer.nodes.zsorted();

    // Event loop
    events.run(move |event, _, control_flow| {
        // They need to be present
        let _gl_display = &app.display;
        let _window = &window;

        control_flow.set_wait();

        match event {
            Event::NewEvents(StartCause::Poll) | Event::RedrawRequested(_) => {
                debug!("Redrawing");

                app.renderer.clear();
                app.renderer.render_nodes(&zsorted_nodes);

                app.surface.swap_buffers(&app.gl_ctx).unwrap();
                // window.request_redraw();
            }
            _ => app.update(event),
        }
    })
}
