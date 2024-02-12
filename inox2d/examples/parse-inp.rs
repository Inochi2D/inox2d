use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use clap::Parser;
use inox2d::formats::inp::parse_inp;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[arg(help = "Path to the .inp file. .inx files don't work!")]
	inp_path: PathBuf,
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

	let model = parse_inp(data.as_slice()).unwrap();

	println!("== Puppet Meta ==\n{}", &model.puppet.meta);
	println!("== Nodes ==\n{}", &model.puppet.nodes);
	if model.vendors.is_empty() {
		println!("(No Vendor Data)\n");
	} else {
		println!("== Vendor Data ==");
		for vendor in &model.vendors {
			println!("{vendor}");
		}
	}
}
