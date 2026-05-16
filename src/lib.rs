use std::{
	error::Error,
	fmt::Display,
	fs, io,
	path::{Path, PathBuf},
};

use physis_re_exports::sqpack::{IndexEntry, SqPackIndex};

mod asset_path;
pub use asset_path::AssetPath;
mod category;
pub use category::Category;
mod index_type;
pub use index_type::{IndexClassificationError, IndexInfo, IndexType, classify_index_path};
pub(crate) mod macro_rules;
mod platform;
pub use platform::Platform;
mod repository_type;
pub use repository_type::{ParseRepositoryTypeError, RepositoryType};

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

#[derive(Debug)]
pub struct Index {
	info: IndexInfo,
	index: SqPackIndex,
	index2: SqPackIndex,
}

#[derive(Debug)]
pub struct IndexesEntry {
	category: Category,
	index_vec: Vec<Index>,
}

impl IndexesEntry {
	pub fn new(category: Category) -> Self {
		Self {
			category,
			index_vec: Vec::new(),
		}
	}
}

#[derive(Debug)]
pub struct Indexes {
	inner: Vec<IndexesEntry>,
}

impl Indexes {
	pub fn new() -> Self {
		Self { inner: Vec::new() }
	}

	pub fn get(&self, category: Category) -> Option<&[Index]> {
		self.inner
			.binary_search_by_key(&category, |entry| entry.category)
			.map(|index| self.inner[index].index_vec.as_slice())
			.ok()
	}

	pub fn get_mut_or_create_new(&mut self, category: Category) -> &mut Vec<Index> {
		match self
			.inner
			.binary_search_by_key(&category, |entry| entry.category)
		{
			Ok(index) => &mut self.inner[index].index_vec,
			Err(index) => {
				self.inner.insert(index, IndexesEntry::new(category));
				&mut self.inner[index].index_vec
			},
		}
	}
}

#[derive(Debug)]
pub struct Repository {
	r#type: RepositoryType,
	indexes: Indexes,
}

#[derive(Debug, strum::Display)]
pub enum LoadRepositoryError {
	#[strum(to_string = "Failed to parse repository type: {0}")]
	ParseRepositoryTypeError(ParseRepositoryTypeError),
	#[strum(to_string = "IO error encountered: {0}")]
	IoError(io::Error),
	#[strum(to_string = r#"Mismatched number of files per index type"#)]
	MismatchedNumberOfIndexFilesPerType,
	#[strum(to_string = "Failed to classify index: {0}")]
	IndexClassificationError(IndexClassificationError),
	#[strum(to_string = "Failed to parse index file")]
	ParseIndexFileError,
}

impl From<ParseRepositoryTypeError> for LoadRepositoryError {
	fn from(err: ParseRepositoryTypeError) -> Self {
		Self::ParseRepositoryTypeError(err)
	}
}

impl From<io::Error> for LoadRepositoryError {
	fn from(err: io::Error) -> Self {
		Self::IoError(err)
	}
}

impl From<IndexClassificationError> for LoadRepositoryError {
	fn from(err: IndexClassificationError) -> Self {
		Self::IndexClassificationError(err)
	}
}

impl Error for LoadRepositoryError {}

impl Repository {
	pub fn load(path: impl AsRef<Path>) -> Result<Self, LoadRepositoryError> {
		let path = path.as_ref();
		let repository_type = path
			.file_name()
			.and_then(|s| s.to_str())
			.map(|s| s.parse())
			.unwrap_or(Err(ParseRepositoryTypeError::IncorrectFormat))?;

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
		let (paths, remainder) = paths.as_chunks::<2>();
		if remainder.len() != 0 {
			return Err(LoadRepositoryError::MismatchedNumberOfIndexFilesPerType);
		}
		let mut indexes = Indexes::new();
		for [index_path, index2_path] in paths {
			let (index_info, index_type) = classify_index_path(index_path)?;
			let (index2_info, index2_type) = classify_index_path(index2_path)?;
			if !(index_type == IndexType::Index
				&& index2_type == IndexType::Index2
				&& index_info == index2_info)
			{
				return Err(LoadRepositoryError::MismatchedNumberOfIndexFilesPerType);
			}

			let platform = index_info.platform.into();
			let index = SqPackIndex::from_existing(platform, &index_path)
				.ok_or(LoadRepositoryError::ParseIndexFileError)?;
			let index2 = SqPackIndex::from_existing(platform, &index2_path)
				.ok_or(LoadRepositoryError::ParseIndexFileError)?;

			let category = index_info.category;
			let index = Index {
				info: index_info,
				index,
				index2,
			};
			indexes.get_mut_or_create_new(category).push(index);
		}

		Ok(Self {
			r#type: repository_type,
			indexes,
		})
	}
}

#[derive(Debug)]
pub struct SqPackResources {
	/// The `game` directory / the directory containing the `sqpack` folder
	game_directory: PathBuf,
	/// The "repositories" SqPack files exist for, contained in subfolders in the `sqpack` folder
	repositories: Vec<Repository>,
}

#[derive(Debug, strum::Display)]
pub enum LoadSqPackResourcesError {
	#[strum(to_string = "IO error encountered: {0}")]
	IoError(io::Error),
	#[strum(to_string = "Failed to load repository: {0}")]
	LoadRepositoryError(LoadRepositoryError),
}

impl From<io::Error> for LoadSqPackResourcesError {
	fn from(err: io::Error) -> Self {
		Self::IoError(err)
	}
}

impl From<LoadRepositoryError> for LoadSqPackResourcesError {
	fn from(err: LoadRepositoryError) -> Self {
		Self::LoadRepositoryError(err)
	}
}

impl Error for LoadSqPackResourcesError {}

#[derive(Debug, Clone, Copy)]
pub struct FolderEntry {
	pub files_offset: u32,
	pub file_count: usize,
}

impl SqPackResources {
	pub fn load(install_path: impl Into<PathBuf>) -> Result<Self, LoadSqPackResourcesError> {
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
			repositories.push(Repository::load(path)?);
		}

		Ok(Self {
			game_directory,
			repositories,
		})
	}

	fn get_repository(&self, repository_type: RepositoryType) -> Option<&Repository> {
		self.repositories
			.binary_search_by_key(&repository_type, |repository| repository.r#type)
			.ok()
			.map(|index| &self.repositories[index])
	}

	pub fn file_exists<'a>(&self, path: impl Into<AssetPath<'a>>) -> Option<IndexEntry> {
		let asset_path = path.into();
		let (category, repository_type) = asset_path.category_repository_type();
		let category = category.ok()?;
		self.get_repository(repository_type)?
			.indexes
			.get(category)?
			.iter()
			.find_map(|index| {
				let found = index.index.find_entry(asset_path.as_ref());
				#[cfg(debug_assertions)]
				if found.is_some() {
					debug_assert!(index.index2.find_entry(asset_path.as_ref()).is_some())
				}
				found
			})
	}

	// pub fn folder_exists<'a>(&self, path: impl Into<AssetPath<'a>>) -> Option<FolderEntry> {
	// 	let asset_path = path.into();
	// 	let (category, repository_type) = asset_path.category_repository_type();
	// 	let category = category.ok()?;
	// 	self.get_repository(repository_type)?.indexes.get(category)?.iter().find_map(|index| {
	// 		let found = index.index.folder_entries.iter().find_map(|folder_entry| folder_entry.)
	// 		#[cfg(debug_assertions)]
	// 		if found.is_some() {
	// 			debug_assert!(index.index2.find_entry(asset_path.as_ref()).is_some())
	// 		}
	// 		found
	// 	})
	// }
}
