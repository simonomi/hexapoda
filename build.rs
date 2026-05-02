use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate_to, Shell};
use clap_complete_nushell::Nushell;
use std::env;
use std::io::Error;

include!("src/arguments.rs");

fn main() -> Result<(), Error> {
	let completions_folder = match env::var_os("HEXAPODA_COMPLETIONS") {
		None => return Ok(()),
		Some(folder) => folder,
	};
	
	let manpage_folder = match env::var_os("HEXAPODA_MANPAGE") {
		None => return Ok(()),
		Some(folder) => folder,
	};
	
	let mut command = Arguments::command();
	for &shell in Shell::value_variants() {
		generate_to(shell, &mut command, "hexapoda", &completions_folder)?;
	}
	generate_to(Nushell, &mut command, "hexapoda", &completions_folder)?;
	
	clap_mangen::generate_to(command, &manpage_folder)?;
	
	println!("cargo:warning=completions generated in {completions_folder:?}");
	println!("cargo:warning=manpage generated in {manpage_folder:?}");
	println!("cargo:rerun-if-changed=src/arguments.rs");
	
	Ok(())
}
