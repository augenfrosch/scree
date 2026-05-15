use std::{
	error::Error,
	fmt::Display,
	fs, io,
	ops::{Index, IndexMut},
	path::{Path, PathBuf},
};

use physis::sqpack::SqPackIndex;

mod asset_path;
pub use asset_path::AssetPath;
mod category;
pub use category::Category;
mod index_type;
pub use index_type::IndexType;
pub(crate) mod macro_rules;
mod platform;
pub use platform::Platform;
mod repository_type;
pub use repository_type::RepositoryType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseEnumError(String);

impl ParseEnumError {
	pub fn new(s: &str) -> Self {
		ParseEnumError(s.to_string())
	}
}

impl Display for ParseEnumError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, r#"Failed to parse "{s}""#, s = self.0)
	}
}

impl Error for ParseEnumError {}

#[derive(Debug, Clone, Copy)]
pub struct IndexesKey {
	pub category: Category,
	pub index_type: IndexType,
}

impl From<IndexesKey> for usize {
	fn from(key: IndexesKey) -> Self {
		let offset = match key.index_type {
			IndexType::Index => 0,
			IndexType::Index2 => 1,
		};
		(key.category as usize) + INDEXES_MAP_SIZE * offset
	}
}

const INDEXES_MAP_SIZE: usize = (Category::Debug as usize + 1) * 2;

#[derive(Debug)]
pub struct IndexesMap {
	inner: [Vec<SqPackIndex>; INDEXES_MAP_SIZE * 2],
}

impl IndexesMap {
	pub fn new() -> Self {
		Self {
			inner: [const { Vec::new() }; INDEXES_MAP_SIZE * 2],
		}
	}
}

impl Index<IndexesKey> for IndexesMap {
	type Output = Vec<SqPackIndex>;

	fn index(&self, key: IndexesKey) -> &Self::Output {
		&self.inner[usize::from(key)]
	}
}

impl IndexMut<IndexesKey> for IndexesMap {
	fn index_mut(&mut self, key: IndexesKey) -> &mut Self::Output {
		&mut self.inner[usize::from(key)]
	}
}

#[derive(Debug)]
pub struct Repository {
	r#type: RepositoryType,
	indexes: IndexesMap,
}

impl Repository {
	pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Option<Self>> {
		let path = path.as_ref();
		let Ok(repository_type) = path
			.file_name()
			.unwrap_or_default()
			.to_string_lossy()
			.parse()
		else {
			return Ok(None);
		};

		let mut indexes = IndexesMap::new();
		let mut paths = fs::read_dir(path)?
			.filter_map(|res| res.ok())
			.map(|entry| entry.path())
			.filter(|path| {
				path.extension()
					.and_then(|s| s.to_str())
					.is_some_and(|extension| extension.starts_with("index"))
			})
			.collect::<Vec<_>>();
		paths.sort();
		dbg!(&paths);
		for path in paths {
			if let Some(filename) = path.file_name()
				&& let Some(filename) = filename.to_str()
				&& let Some((index_name, platform_index_type)) = filename.split_once('.')
				&& let Some((platform, index_type)) = platform_index_type.split_once('.')
				&& let Some(category) = index_name.get(..2)
				&& let Ok(Some(category)) =
					u8::from_str_radix(category, 16).map(Category::from_repr)
			{
				let index_type = index_type
					.parse::<IndexType>()
					.expect(&format!("Unknown index type: {platform}"));
				let platform = platform
					.parse::<Platform>()
					.expect(&format!("Unknown platform: {platform}"));

				let index = SqPackIndex::from_existing(platform.into(), &path).expect(&format!(
					"Failed to read index at {path}",
					path = path.display()
				));

				let key = IndexesKey {
					category,
					index_type,
				};
				indexes[key].push(index);
			}
		}

		Ok(Some(Self {
			r#type: repository_type,
			indexes,
		}))
	}
}

#[derive(Debug)]
pub struct SqPackResources {
	/// The `game` directory / the directory containing the `sqpack` folder
	game_directory: PathBuf,
	/// The "repositories" SqPack files exist for, contained in subfolders in the `sqpack` folder
	repositories: Vec<Repository>,
}

impl SqPackResources {
	pub fn new(install_path: impl Into<PathBuf>) -> io::Result<Self> {
		let mut game_directory = install_path.into();
		if !game_directory.join("sqpack").is_dir() {
			game_directory = game_directory.join("game");
		}

		let mut repositories = Vec::new();
		let mut paths = fs::read_dir(game_directory.join("sqpack"))?
			.into_iter()
			.filter_map(|res| res.ok())
			.map(|entry| entry.path())
			.filter_map(|path| {
				path.file_name()
					.and_then(|filename| filename.to_str())
					.and_then(|filename| filename.parse::<RepositoryType>().ok())
					.map(|repository_type| (repository_type, path))
			})
			.collect::<Vec<_>>();
		paths.sort_by_key(|(repository_type, _path)| *repository_type);
		dbg!(&paths);
		for (_repository_type, path) in paths {
			if let Some(repository) = Repository::new(path)? {
				repositories.push(repository);
			}
		}
		// repositories.sort_by_key(|repository| repository.r#type);

		Ok(Self {
			game_directory,
			repositories,
		})
	}
}
