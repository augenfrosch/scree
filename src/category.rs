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
pub enum Category {
	Common = 0x00,
	Bgcommon = 0x01,
	Bg = 0x02,
	Cut = 0x03,
	Chara = 0x04,
	Shader = 0x05,
	Ui = 0x06,
	Sound = 0x07,
	Vfx = 0x08,
	UiScript = 0x09,
	Exd = 0x0a,
	GameScript = 0x0b,
	Music = 0x0c,
	// 0x0d..=0x11 are currently unused or at least not present in the released version
	#[strum(serialize = "_sqpack_test")]
	SqpackTest = 0x12,
	#[strum(serialize = "_debug")]
	Debug = 0x13,
}

impl From<Category> for u8 {
	fn from(category: Category) -> Self {
		category as u8
	}
}

#[cfg(test)]
mod test {
	use super::{Category, ParseEnumError};
	use crate::{assert_display_fromstr_equivalent, assert_fromrepr_reprfrom_equivalent};

	#[test]
	fn test_fromrepr_reprfrom() {
		assert_fromrepr_reprfrom_equivalent!((0x00, Category::Common): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x01, Category::Bgcommon): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x02, Category::Bg): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x03, Category::Cut): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x04, Category::Chara): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x05, Category::Shader): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x06, Category::Ui): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x07, Category::Sound): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x08, Category::Vfx): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x09, Category::UiScript): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x0a, Category::Exd): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x0b, Category::GameScript): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x0c, Category::Music): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x12, Category::SqpackTest): (u8, Category));
		assert_fromrepr_reprfrom_equivalent!((0x13, Category::Debug): (u8, Category));
	}

	#[test]
	fn test_fromrepr_none() {
		assert_eq!(Category::from_repr(0x0d), None);
		assert_eq!(Category::from_repr(0x0e), None);
		assert_eq!(Category::from_repr(0x0f), None);
		assert_eq!(Category::from_repr(0x10), None);
		assert_eq!(Category::from_repr(0x11), None);

		assert_eq!(Category::from_repr(0x14), None);
		assert_eq!(Category::from_repr(0xff), None);
	}

	#[test]
	fn test_derived_display_and_fromstr() {
		assert_display_fromstr_equivalent!("common", Category::Common);
		assert_display_fromstr_equivalent!("bgcommon", Category::Bgcommon);
		assert_display_fromstr_equivalent!("bg", Category::Bg);
		assert_display_fromstr_equivalent!("cut", Category::Cut);
		assert_display_fromstr_equivalent!("chara", Category::Chara);
		assert_display_fromstr_equivalent!("ui", Category::Ui);
		assert_display_fromstr_equivalent!("vfx", Category::Vfx);
		assert_display_fromstr_equivalent!("ui_script", Category::UiScript);
		assert_display_fromstr_equivalent!("exd", Category::Exd);
		assert_display_fromstr_equivalent!("game_script", Category::GameScript);
		assert_display_fromstr_equivalent!("music", Category::Music);
		assert_display_fromstr_equivalent!("_sqpack_test", Category::SqpackTest);
		assert_display_fromstr_equivalent!("_debug", Category::Debug);
	}

	#[test]
	fn test_parse_error() {
		assert_eq!(
			"AAAA".parse::<Category>(),
			Err(ParseEnumError("AAAA".to_string()))
		);
		assert_eq!(
			"debug".parse::<Category>(),
			Err(ParseEnumError("debug".to_string()))
		);
	}
}
