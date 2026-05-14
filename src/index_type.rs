use strum::{EnumString, FromRepr};

use crate::ParseEnumError;

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

impl From<IndexType> for physis::sqpack::IndexType {
	fn from(index_type: IndexType) -> Self {
		match index_type {
			IndexType::Index => physis::sqpack::IndexType::Index1,
			IndexType::Index2 => physis::sqpack::IndexType::Index2,
		}
	}
}

#[cfg(test)]
mod test {
	use super::{IndexType, ParseEnumError};

	macro_rules! display_fromstr_equivalence_check {
		($str:literal, $enum:expr) => {
			assert_eq!($enum.to_string(), $str);
			assert_eq!($str.parse(), Ok($enum));
		};
	}

	#[test]
	fn test_derived_display_and_fromstr() {
		display_fromstr_equivalence_check!("index", IndexType::Index);
		display_fromstr_equivalence_check!("index2", IndexType::Index2);
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
}
