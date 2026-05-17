use std::{error::Error, fs};

use scree::SqPackResources;

fn main() -> Result<(), Box<dyn Error>> {
	let args = std::env::args_os().collect::<Vec<_>>();
	if args.len() != 3 {
		return Err("Missing arguments, expects: `verify_current_pathlist <GAME_INSTALL_DIR> <CURRENT_PATH_LIST_FILE>`".into());
	}

	let install_path = &args[1];
	let path_list_file = &args[2];

	let resources = SqPackResources::load(install_path)?;
	let paths = fs::read_to_string(path_list_file)?;

	for line in paths.lines() {
		assert!(resources.file_exists(line).is_some());
	}

	Ok(())
}
