use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use clap::Parser;
use inox2d::formats::inp::{parse_inp_parts, serialize_parts};

const BINDINGS_KEY: &str = "com.inochi2d.inochi-session.bindings";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[arg(help = "The .inp to copy parameter mappings from")]
	in_path: PathBuf,

	#[arg(help = "The .inp to copy parameter mappings to")]
	out_path: PathBuf,
}

fn main() {
	let cli = Cli::parse();

	let indata = {
		let file = File::open(cli.in_path).unwrap();
		let mut file = BufReader::new(file);
		let mut data = Vec::new();
		file.read_to_end(&mut data).unwrap();
		data
	};

	let (_in_puppet, _in_textures, in_vendors) = match parse_inp_parts(indata.as_slice()) {
		Ok(m) => m,
		Err(e) => {
			println!("Error when reading input puppet: {e}");
			return;
		}
	};

	let outdata = {
		let file = File::open(cli.out_path.clone()).unwrap();
		let mut file = BufReader::new(file);
		let mut data = Vec::new();
		file.read_to_end(&mut data).unwrap();
		data
	};

	let (out_puppet, out_textures, mut out_vendors) = match parse_inp_parts(outdata.as_slice()) {
		Ok(m) => m,
		Err(e) => {
			println!("Error when reading input puppet: {e}");
			return;
		}
	};

	//Bindings are treated as "vendor data" for some reason
	let mut in_bindings = None;
	for (index, item) in in_vendors.iter().enumerate() {
		if item.name == BINDINGS_KEY {
			in_bindings = Some(index);
		}
	}

	let in_bindings = in_vendors
		.get(in_bindings.expect("Input puppet has bindings data"))
		.expect("Bindings data didn't change from prior lookup");

	let mut out_bindings = None;
	for (index, item) in out_vendors.iter().enumerate() {
		if item.name == BINDINGS_KEY {
			out_bindings = Some(index);
		}
	}

	if out_bindings.is_some() {
		println!("Output puppet already has bindings data! Refusing to continue.");
		return;
	}

	// TODO: Validate that the bindings still make sense when copied in this
	// way. We're sort of relying on the user to only copy bindings between
	// two versions of the same puppet.

	out_vendors.push(in_bindings.clone());

	let file = File::create(cli.out_path).unwrap();
	serialize_parts(file, out_puppet, &out_textures, &out_vendors).unwrap();
}
