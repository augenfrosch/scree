use std::{
	error::Error,
	fmt::Display,
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

#[derive(Debug, PartialEq, Eq)]
pub enum ParseRepositoryTypeError {
	FailedToParseExpansionNumber(ParseIntError),
	IncorrectFormat,
}

impl Display for ParseRepositoryTypeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ParseRepositoryTypeError::FailedToParseExpansionNumber(parse_int_error) => {
				write!(f, "Failed to parse repository type: {parse_int_error}")
			},
			ParseRepositoryTypeError::IncorrectFormat => {
				write!(f, "Failed to parse repository type, incorrect format")
			},
		}
	}
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

	macro_rules! display_fromstr_equivalence_check {
		($str:expr, $enum:expr) => {
			assert_eq!($enum.to_string(), $str);
			assert_eq!($str.parse(), Ok($enum));
		};
	}

	#[test]
	fn test_derived_display_and_fromstr() {
		display_fromstr_equivalence_check!("ffxiv", RepositoryType::BaseGame);
		for i in 1..=u8::MAX {
			let i = NonZero::new(i).unwrap();

			display_fromstr_equivalence_check!(
				format!("ex{i}"),
				RepositoryType::Expansion { number: i }
			);
		}
	}

	macro_rules! assert_parse_int_error_kind_eq {
		($str:literal, $err:expr) => {
			match $str.parse::<RepositoryType>() {
				Err(ParseRepositoryTypeError::FailedToParseExpansionNumber(parse_int_error)) => {
					assert_eq!(*parse_int_error.kind(), $err)
				},
				res => assert!(res.is_err()),
			}
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
