use std::{
	error::Error,
	num::{NonZero, ParseIntError},
	str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::Display)]
#[repr(u8)]
pub enum RepositoryType {
	#[strum(to_string = "ffxiv")]
	BaseGame = 0,
	#[strum(to_string = "ex{number}")]
	Expansion { number: NonZero<u8> },
}

impl From<u8> for RepositoryType {
	fn from(number: u8) -> Self {
		match NonZero::new(number) {
			Some(number) => Self::Expansion { number },
			None => Self::BaseGame,
		}
	}
}

impl From<RepositoryType> for u8 {
	fn from(repository_type: RepositoryType) -> Self {
		match repository_type {
			RepositoryType::BaseGame => 0,
			RepositoryType::Expansion { number } => number.get(),
		}
	}
}

#[derive(Debug, PartialEq, Eq, strum::Display)]
pub enum ParseRepositoryTypeError {
	#[strum(to_string = "Failed to parse repository type: {0}")]
	FailedToParseExpansionNumber(ParseIntError),
	#[strum(to_string = "Failed to parse repository type, incorrect format")]
	IncorrectFormat,
}

impl Error for ParseRepositoryTypeError {}

impl FromStr for RepositoryType {
	type Err = ParseRepositoryTypeError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"ffxiv" => Ok(Self::BaseGame),
			_ if let Some(("", number)) = s.split_once("ex") => Ok(Self::Expansion {
				number: number
					.parse()
					.map_err(ParseRepositoryTypeError::FailedToParseExpansionNumber)?,
			}),
			_ => Err(ParseRepositoryTypeError::IncorrectFormat),
		}
	}
}

#[cfg(test)]
mod test {
	use std::num::{IntErrorKind, NonZero};

	use super::{ParseRepositoryTypeError, RepositoryType};
	use crate::{assert_display_fromstr_equivalent, assert_parse_error_matches_expr};

	#[test]
	fn test_derived_display_and_fromstr() {
		assert_display_fromstr_equivalent!("ffxiv", RepositoryType::BaseGame);
		for i in 1..=u8::MAX {
			let i = NonZero::new(i).unwrap();

			assert_display_fromstr_equivalent!(
				format!("ex{i}"),
				RepositoryType::Expansion { number: i }
			);
		}
	}

	macro_rules! assert_parse_int_error_kind_eq {
		($str:literal, $err:expr) => {
			assert_parse_error_matches_expr!(
				$str->RepositoryType,
				ParseRepositoryTypeError::FailedToParseExpansionNumber(parse_int_error) => {
					assert_eq!(*parse_int_error.kind(), $err);
				},
			);
		};
	}

	#[test]
	fn test_parse_error() {
		assert_eq!(
			"AAAA".parse::<RepositoryType>(),
			Err(ParseRepositoryTypeError::IncorrectFormat)
		);
		assert_parse_int_error_kind_eq!("ex_1", IntErrorKind::InvalidDigit);
		assert_parse_int_error_kind_eq!("ex1111", IntErrorKind::PosOverflow);
		assert_parse_int_error_kind_eq!("ex", IntErrorKind::Empty);
		assert_parse_int_error_kind_eq!("ex0", IntErrorKind::Zero);
	}
}
