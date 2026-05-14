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
pub enum Platform {
	Win32 = 0x0,
	Ps3 = 0x1,
	Ps4 = 0x2,
	Ps5 = 0x3,
	#[strum(serialize = "lys")]
	Xbox = 0x4,
}

impl From<Platform> for physis::Platform {
	fn from(platform: Platform) -> Self {
		match platform {
			Platform::Win32 => physis::Platform::Win32,
			Platform::Ps3 => physis::Platform::PS3,
			Platform::Ps4 => physis::Platform::PS4,
			Platform::Ps5 => physis::Platform::PS5,
			Platform::Xbox => physis::Platform::Xbox,
		}
	}
}

#[cfg(test)]
mod test {
	use super::{ParseEnumError, Platform};

	macro_rules! display_fromstr_equivalence_check {
		($str:literal, $enum:expr) => {
			assert_eq!($enum.to_string(), $str);
			assert_eq!($str.parse(), Ok($enum));
		};
	}

	#[test]
	fn test_derived_display_and_fromstr() {
		display_fromstr_equivalence_check!("win32", Platform::Win32);
		display_fromstr_equivalence_check!("ps3", Platform::Ps3);
		display_fromstr_equivalence_check!("ps4", Platform::Ps4);
		display_fromstr_equivalence_check!("ps5", Platform::Ps5);
		display_fromstr_equivalence_check!("lys", Platform::Xbox);
	}

	#[test]
	fn test_parse_error() {
		assert_eq!(
			"AAAA".parse::<Platform>(),
			Err(ParseEnumError("AAAA".to_string()))
		);
		assert_eq!(
			"WIN32".parse::<Platform>(),
			Err(ParseEnumError("WIN32".to_string()))
		);
		assert_eq!(
			"xbox".parse::<Platform>(),
			Err(ParseEnumError("xbox".to_string()))
		);
	}
}
