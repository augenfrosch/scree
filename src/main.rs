use scree::{LoadSqPackResourcesError, SqPackResources};

fn main() -> Result<(), LoadSqPackResourcesError> {
	let res = SqPackResources::load(std::env::var("INPUT").unwrap())?;
	// dbg!(res);

	for (_i, line) in std::io::stdin().lines().enumerate() {
		let line = line.unwrap();
		// if i % 10000 == 0 {
		// 	println!("@ i = {i}: {line}");
		// }

		if line.ends_with('/') {
			// res.folder_exists(line.as_str().trim_end_matches('/')).unwrap();
			println!(
				"{line}: {exists:?}",
				exists = res.folder_exists(line.as_str().trim_end_matches('/'))
			);
		} else {
			// res.file_exists(line.as_str()).unwrap();
			println!(
				"{line}: {exists:?}",
				exists = res.file_exists(line.as_str())
			);
		}
	}
	// std::thread::sleep(std::time::Duration::new(120, 0));
	// assert!(!res.repositories.iter().any(|repository| repository.indexes.inner.iter().filter(Option::is_some).))
	Ok(())
}
