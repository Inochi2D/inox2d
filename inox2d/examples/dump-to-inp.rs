use std::fs::File;
use std::path::PathBuf;

use clap::Parser;
use inox2d::formats::inp::dump_to_inp;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[arg(help = "The dump directory.")]
	dump_dir: PathBuf,

	#[arg(short, long, help = "Output file (.inp or .inx)")]
	output: PathBuf,
}

fn main() {
	let cli = Cli::parse();

	let mut output = File::create(&cli.output).unwrap();
	dump_to_inp(&cli.dump_dir, &mut output).unwrap();
}
