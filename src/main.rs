use std::io;

use scree::SqPackResources;

fn main() -> io::Result<()> {
	let res = SqPackResources::new(std::env::var("INPUT").unwrap())?;
	dbg!(res);
	// std::thread::sleep(std::time::Duration::new(120, 0));
	// assert!(!res.repositories.iter().any(|repository| repository.indexes.inner.iter().filter(Option::is_some).))
	Ok(())
}
