use std::{
	error::Error,
	fmt::Display,
	fs, io,
	path::{Path, PathBuf},
};

use physis_re_exports::sqpack::{FolderEntryInfo, Hash, IndexEntry, SqPackIndex};

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
	_info: IndexInfo,
	index: SqPackIndex,
	_index2: SqPackIndex,
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
	#[strum(to_string = "Parsed indexes' entries are not sorted by their hash")]
	IndexEntriesNotSorted,
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
			if !index.entries.is_sorted_by_key(|entry| match entry.hash {
				physis_re_exports::sqpack::Hash::SplitPath { name, path } => (path, name),
				physis_re_exports::sqpack::Hash::FullPath(_) => unreachable!(
					"Malformed index, indexes of type `.index` must not have full-path hashes"
				),
			}) || !index2.entries.is_sorted_by_key(|entry| match entry.hash {
				physis_re_exports::sqpack::Hash::SplitPath { .. } => unreachable!(
					"Malformed index, indexes of type `.index2` must not have split-path hashes"
				),
				physis_re_exports::sqpack::Hash::FullPath(hash) => hash,
			}) {
				return Err(LoadRepositoryError::IndexEntriesNotSorted);
			}

			let category = index_info.category;
			let index = Index {
				_info: index_info,
				index,
				_index2: index2,
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
	_game_directory: PathBuf,
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

		for (_repository_type, path) in paths {
			repositories.push(Repository::load(path)?);
		}

		Ok(Self {
			_game_directory: game_directory,
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

		let Hash::SplitPath {
			name: filename_crc,
			path: directory_crc,
		} = SqPackIndex::calculate_hash(
			physis_re_exports::sqpack::IndexType::Index1,
			asset_path.as_ref(),
		)
		else {
			unreachable!(
				"`SqPackIndex::calculate_hash` should always return a split-path hash for `IndexType::Index1`"
			)
		};

		self.get_repository(repository_type)?
			.indexes
			.get(category)?
			.iter()
			.find_map(|index| {
				index
					.index
					.entries
					.binary_search_by(|entry| match entry.hash {
						Hash::SplitPath { name, path } => {
							(path, name).cmp(&(directory_crc, filename_crc))
						},
						Hash::FullPath(_) => unreachable!(
							"Full-path hash in `.index` file (This should have been caught while loading)"
						),
					})
					.ok()
					.map(|idx| IndexEntry::from(&index.index.entries[idx]))

				// TODO: Look into this. The assertion seems to not hold (which would explain why ironworks & Physis iterate over all of them, I think)
				// But the number of entries in the `.index2`s seems to be consistently much lower than the corresponding `.index`
				// (which is close to what ResLogger2 has in the `CurrentPathList`).
				// #[cfg(debug_assertions)]
				// if found.is_some() {
				// 	debug_assert_eq!(found, index.index2.find_entry(asset_path.as_ref()))
				// }
				// #[cfg(debug_assertions)]
				// if let Some(found) = &found
				// 	&& let Some(found2) = &index.index2.find_entry(asset_path.as_ref())
				// 	&& found != found2
				// {
				// 	eprintln!(
				// 		r#"Assertion for path="{path}" failed: {found:?} != {found2:?}"#,
				// 		path = asset_path.as_ref()
				// 	);
				// }
			})
	}

	pub fn folder_exists<'a>(&self, path: impl Into<AssetPath<'a>>) -> Option<FolderEntryInfo> {
		let asset_path = path.into();
		let (category, repository_type) = asset_path.category_repository_type();
		let category = category.ok()?;

		// This is currently partially copy-paste from `index.rs`; TODO: look into somehow consolidating this
		let path = if asset_path.ends_with('/') {
			&asset_path[..asset_path.len() - 1]
		} else {
			&asset_path
		};
		let hash = SqPackIndex::calculate_partial_hash(path);

		self.get_repository(repository_type)?
			.indexes
			.get(category)?
			.iter()
			.find_map(|index| {
				index
					.index
					.folder_entries
					.binary_search_by_key(&hash, |folder_entry| folder_entry.hash)
					.ok()
					.map(|idx| FolderEntryInfo::new(&index.index, &index.index.folder_entries[idx]))

				// See above TODO; `index2.folder_entries` seems to be empty
				// which does seem to make sense since it looks like the entries point to a continuous range of file entries for the folder
				// and the entries are sorted by their hash (which would make them non-contiguous for `.index2` files).
				// #[cfg(debug_assertions)]
				// if found.is_some() {
				// 	debug_assert!(index.index2.find_folder_entry(asset_path.as_ref()).is_some())
				// }
			})
	}
}
