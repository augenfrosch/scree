use std::error::Error;

use scree::SqPackResources;

fn main() -> Result<(), Box<dyn Error>> {
	let args = std::env::args_os().collect::<Vec<_>>();
	if args.len() != 2 {
		return Err("Missing arguments, expects: `barely_interactive_path_existance_checker <GAME_INSTALL_DIR>`".into());
	}

	let install_path = &args[1];

	let resources = SqPackResources::load(install_path)?;

	println!(
		"Loaded resources. Enter paths to check (paths ending in '/' will check for the existance of the folder)"
	);
	for line in std::io::stdin().lines() {
		let line = line?;
		if line.ends_with('/') {
			println!(
				"{line}: {exists:?}",
				exists = resources.folder_exists(line.as_str())
			);
		} else {
			println!(
				"{line}: {exists:?}",
				exists = resources.file_exists(line.as_str())
			);
		}
	}

	Ok(())
}
