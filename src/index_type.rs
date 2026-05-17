use std::{error::Error, fmt::Display, num::ParseIntError, path::Path, str::FromStr};

use physis_re_exports::sqpack::IndexType as PhysisIndexType;
use strum::{EnumString, FromRepr};

use crate::{Category, ParseEnumError, Platform, RepositoryType};

#[derive(
	Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, FromRepr, strum::Display, EnumString,
)]
#[strum(serialize_all = "snake_case")]
#[strum(
	parse_err_fn = ParseEnumError::new,
	parse_err_ty = ParseEnumError,
)]
#[repr(u8)]
pub enum IndexType {
	Index = 0x0,
	Index2 = 0x2,
}

impl From<IndexType> for PhysisIndexType {
	fn from(index_type: IndexType) -> Self {
		match index_type {
			IndexType::Index => PhysisIndexType::Index1,
			IndexType::Index2 => PhysisIndexType::Index2,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexInfo {
	pub category: Category,
	pub repository_type: RepositoryType,
	pub chunk: u8,
	pub platform: Platform,
}

impl Display for IndexInfo {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let Self {
			category,
			repository_type,
			chunk,
			platform,
		} = *self;
		// ex4's `020409.<...>.index(2)` has the highest chunk number, so it is not apparent what base chunk is in (same for `repository_type`)
		write!(
			f,
			"{category:02x}{repository_type:02x}{chunk:02x}.{platform}",
			category = u8::from(category),
			repository_type = u8::from(repository_type)
		)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, strum::Display)]
pub enum ParseIndexInfoError {
	#[strum(to_string = "Invalid format")]
	FormatInvalid,
	#[strum(to_string = "Invalid category")]
	CategoryInvalid,
	#[strum(to_string = "Invalid part: {0}")]
	IntegerPartParsingError(ParseIntError),
	#[strum(to_string = "Invalid part: {0}")]
	PartParsingError(ParseEnumError),
}

impl From<ParseEnumError> for ParseIndexInfoError {
	fn from(err: ParseEnumError) -> Self {
		Self::PartParsingError(err)
	}
}

impl From<ParseIntError> for ParseIndexInfoError {
	fn from(err: ParseIntError) -> Self {
		Self::IntegerPartParsingError(err)
	}
}

impl Error for ParseIndexInfoError {}

impl FromStr for IndexInfo {
	type Err = ParseIndexInfoError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let Some((index_name, platform)) = s.split_once('.') else {
			return Err(ParseIndexInfoError::FormatInvalid);
		};
		if index_name.len() == 6
			&& let (Some(category), Some(repository_type), Some(chunk)) =
				(s.get(0..2), s.get(2..4), s.get(4..6))
		{
			let category = Category::from_repr(u8::from_str_radix(category, 16)?)
				.ok_or(ParseIndexInfoError::CategoryInvalid)?;
			let repository_type = RepositoryType::from(u8::from_str_radix(repository_type, 16)?);
			let chunk = u8::from_str_radix(chunk, 16)?;
			let platform = platform.parse()?;
			Ok(Self {
				category,
				repository_type,
				chunk,
				platform,
			})
		} else {
			Err(ParseIndexInfoError::FormatInvalid)
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, strum::Display)]
pub enum IndexClassificationError {
	#[strum(to_string = "Invalid path")]
	InvalidPath,
	// TODO: look into error messages; I very much doubt they are consistent and according to best practice
	#[strum(to_string = "Invalid part: {0}")]
	FailedToParseEnum(ParseEnumError),
	#[strum(to_string = "Invalid part: {0}")]
	FailedToParseIndexInfo(ParseIndexInfoError),
}

impl From<ParseEnumError> for IndexClassificationError {
	fn from(err: ParseEnumError) -> Self {
		Self::FailedToParseEnum(err)
	}
}

impl From<ParseIndexInfoError> for IndexClassificationError {
	fn from(err: ParseIndexInfoError) -> Self {
		Self::FailedToParseIndexInfo(err)
	}
}

impl Error for IndexClassificationError {}

/// Classifies the path to an index file by the info contained in the filename.
///
/// # Errors
/// Returns an [`IndexClassificationError`] if the path is invalid or parsing of its components fails.
pub fn classify_index_path(
	path: impl AsRef<Path>,
) -> Result<(IndexInfo, IndexType), IndexClassificationError> {
	let path = path.as_ref();
	if let (Some(index_info), Some(index_type)) = (
		path.file_stem().and_then(|file_stem| file_stem.to_str()),
		path.extension().and_then(|extension| extension.to_str()),
	) {
		let index_info = index_info.parse()?;
		let index_type = index_type.parse()?;

		Ok((index_info, index_type))
	} else {
		Err(IndexClassificationError::InvalidPath)
	}
}

#[cfg(test)]
mod test {
	use std::num::{IntErrorKind, NonZero};

	use super::{
		Category, IndexInfo, IndexType, ParseEnumError, ParseIndexInfoError, Platform,
		RepositoryType,
	};
	use crate::{assert_display_fromstr_equivalent, assert_parse_error_matches_expr};

	#[test]
	fn test_derived_display_and_fromstr() {
		assert_display_fromstr_equivalent!("index", IndexType::Index);
		assert_display_fromstr_equivalent!("index2", IndexType::Index2);
	}

	#[test]
	fn test_parse_error() {
		assert_eq!(
			"AAAA".parse::<IndexType>(),
			Err(ParseEnumError("AAAA".to_string()))
		);
		assert_eq!(
			"INDEX1".parse::<IndexType>(),
			Err(ParseEnumError("INDEX1".to_string()))
		);
		assert_eq!(
			"index3".parse::<IndexType>(),
			Err(ParseEnumError("index3".to_string()))
		);
	}

	#[test]
	fn test_parse_index_info() {
		assert_display_fromstr_equivalent!(
			"000000.win32",
			IndexInfo {
				category: Category::Common,
				repository_type: RepositoryType::BaseGame,
				chunk: 0,
				platform: Platform::Win32
			}
		);
		assert_display_fromstr_equivalent!(
			"020105.win32",
			IndexInfo {
				category: Category::Bg,
				repository_type: RepositoryType::Expansion {
					number: NonZero::new(1).unwrap()
				},
				chunk: 5,
				platform: Platform::Win32
			}
		);
		assert_display_fromstr_equivalent!(
			"0c0200.win32",
			IndexInfo {
				category: Category::Music,
				repository_type: RepositoryType::Expansion {
					number: NonZero::new(2).unwrap()
				},
				chunk: 0,
				platform: Platform::Win32
			}
		);
		assert_display_fromstr_equivalent!(
			"020409.win32",
			IndexInfo {
				category: Category::Bg,
				repository_type: RepositoryType::Expansion {
					number: NonZero::new(4).unwrap()
				},
				chunk: 9,
				platform: Platform::Win32
			}
		);
	}

	#[test]
	fn test_parse_index_info_error() {
		assert_eq!(
			"AAAA".parse::<IndexInfo>(),
			Err(ParseIndexInfoError::FormatInvalid)
		);
		assert_eq!(
			"000.win32".parse::<IndexInfo>(),
			Err(ParseIndexInfoError::FormatInvalid)
		);
		assert_eq!(
			"0f0000.win32".parse::<IndexInfo>(),
			Err(ParseIndexInfoError::CategoryInvalid)
		);
		assert_parse_error_matches_expr!(
			"02040z.win32"->IndexInfo,
			ParseIndexInfoError::IntegerPartParsingError(parse_int_error) => {
				assert_eq!(*parse_int_error.kind(), IntErrorKind::InvalidDigit);
			},
		);
		assert_parse_error_matches_expr!(
			"020409.win3"->IndexInfo,
			ParseIndexInfoError::PartParsingError(ParseEnumError(part)) => {
				assert_eq!(part, "win3");
			},
		);
	}
}
