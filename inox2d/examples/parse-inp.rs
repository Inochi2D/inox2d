use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::PathBuf;

use clap::Parser;
use inox2d::formats::inp::{dump_inp, parse_inp};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[arg(help = "Path to the .inp or .inx file.")]
	inp_path: PathBuf,

	#[arg(long, help = "The directory where to dump the inp file's internals. (No dumping if unspecified.)")]
	dump_dir: Option<PathBuf>,
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

	if let Some(dump_dir) = cli.dump_dir {
		fs::create_dir_all(&dump_dir).unwrap();
		dump_inp(data.as_slice(), &dump_dir).unwrap();
	}

	use std::time::Instant;
	let now = Instant::now();

	let model = match parse_inp(data.as_slice()) {
		Ok(m) => m,
		Err(e) => {
			println!("{e}");
			return;
		}
	};

	let elapsed = now.elapsed();
	println!("parse_inp() took: {:.2?}", elapsed);

	println!("== Puppet Meta ==\n{}", &model.puppet.meta);
	// TODO: Implement full node print after ECS
	// println!("== Nodes ==\n{}", &model.puppet.nodes);
	if model.vendors.is_empty() {
		println!("(No Vendor Data)\n");
	} else {
		println!("== Vendor Data ==");
		for vendor in &model.vendors {
			println!("{vendor}");
		}
	}

	println!("== Puppet Textures ({}) ==", model.textures.len());
	for texture in &model.textures {
		println!("{:?} ({} B)", texture.format, texture.data.len());
	}
}
