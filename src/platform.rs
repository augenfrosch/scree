use physis_re_exports::common::Platform as PhysisPlatform;
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
	Win32 = 0,
	Ps3 = 1,
	Ps4 = 2,
	Ps5 = 3,
	#[strum(serialize = "lys")]
	Xbox = 4,
}

impl From<Platform> for u8 {
	fn from(platform: Platform) -> Self {
		platform as u8
	}
}

impl From<Platform> for PhysisPlatform {
	fn from(platform: Platform) -> Self {
		match platform {
			Platform::Win32 => PhysisPlatform::Win32,
			Platform::Ps3 => PhysisPlatform::PS3,
			Platform::Ps4 => PhysisPlatform::PS4,
			Platform::Ps5 => PhysisPlatform::PS5,
			Platform::Xbox => PhysisPlatform::Xbox,
		}
	}
}

#[cfg(test)]
mod test {
	use super::{ParseEnumError, Platform};
	use crate::{assert_display_fromstr_equivalent, assert_fromrepr_reprfrom_equivalent};

	#[test]
	fn test_fromrepr_reprfrom() {
		assert_fromrepr_reprfrom_equivalent!((0, Platform::Win32): (u8, Platform));
		assert_fromrepr_reprfrom_equivalent!((1, Platform::Ps3): (u8, Platform));
		assert_fromrepr_reprfrom_equivalent!((2, Platform::Ps4): (u8, Platform));
		assert_fromrepr_reprfrom_equivalent!((3, Platform::Ps5): (u8, Platform));
		assert_fromrepr_reprfrom_equivalent!((4, Platform::Xbox): (u8, Platform));
	}

	#[test]
	fn test_fromrepr_none() {
		assert_eq!(Platform::from_repr(5), None);
		assert_eq!(Platform::from_repr(5), None);
		assert_eq!(Platform::from_repr(5), None);

		assert_eq!(Platform::from_repr(122), None);

		assert_eq!(Platform::from_repr(u8::MAX), None);
	}

	#[test]
	fn test_derived_display_and_fromstr() {
		assert_display_fromstr_equivalent!("win32", Platform::Win32);
		assert_display_fromstr_equivalent!("ps3", Platform::Ps3);
		assert_display_fromstr_equivalent!("ps4", Platform::Ps4);
		assert_display_fromstr_equivalent!("ps5", Platform::Ps5);
		assert_display_fromstr_equivalent!("lys", Platform::Xbox);
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
