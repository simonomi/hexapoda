use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate_to, Shell};
use std::env;
use std::io::Error;

include!("src/arguments.rs");

fn main() -> Result<(), Error> {
	let output_folder = match env::var_os("OUT_DIR") {
		None => return Ok(()),
		Some(output_folder) => output_folder,
	};
	
	let mut command = Arguments::command();
	for &shell in Shell::value_variants() {
		generate_to(shell, &mut command, "hexapoda", &output_folder)?;
	}
	
	println!("cargo:warning=completions generated in {output_folder:?}");
	println!("cargo:rerun-if-changed=src/arguments.rs");
	
	Ok(())
}
