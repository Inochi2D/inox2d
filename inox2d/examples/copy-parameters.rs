use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use clap::Parser;
use inox2d::formats::inp::{parse_inp_parts, serialize_parts};
use inox2d::formats::vendors::{SessionBinding, SESSION_BINDINGS_KEY};
use inox2d::puppet::Puppet;
use std::collections::HashSet;

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
			eprintln!("Error when reading input puppet: {e}");
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
			eprintln!("Error when reading input puppet: {e}");
			return;
		}
	};

	// Bindings are treated as "vendor data" for some reason
	let mut in_bindings = None;
	for (index, item) in in_vendors.iter().enumerate() {
		if item.name == SESSION_BINDINGS_KEY {
			in_bindings = Some(index);
		}
	}

	let in_bindings = in_vendors
		.get(in_bindings.expect("Input puppet has bindings data"))
		.expect("Bindings data didn't change from prior lookup");

	let mut out_bindings = None;
	for (index, item) in out_vendors.iter().enumerate() {
		if item.name == SESSION_BINDINGS_KEY {
			out_bindings = Some(index);
		}
	}

	if out_bindings.is_some() {
		eprintln!("Output puppet already has bindings data! Refusing to continue.");
		return;
	}

	// Validate that all params mentioned in in_bindings exists in out_puppet.
	let out_puppet_data = Puppet::new_from_json(&out_puppet).expect("valid puppet JSON");
	let mut good_bindings = HashSet::new();
	for (_param_name, param) in out_puppet_data.params {
		good_bindings.insert(param.uuid);
	}

	let in_bindings_data = SessionBinding::new_from_json_list(&in_bindings.payload).expect("valid bindings data");
	for binding_block in in_bindings_data {
		if !good_bindings.contains(&binding_block.param) {
			eprintln!(
				"Binding name {} refers to nonexistent param {:?}.",
				binding_block.name, binding_block.param
			);
			eprintln!("These puppets do not have compatible bindings and cannot have bindings copied.");
			return;
		}
	}

	out_vendors.push(in_bindings.clone());

	let file = File::create(cli.out_path).unwrap();
	serialize_parts(file, out_puppet, &out_textures, &out_vendors).unwrap();
}
