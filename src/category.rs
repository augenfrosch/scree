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

#[cfg(test)]
mod test {
	use super::{Category, ParseEnumError};

	macro_rules! display_fromstr_equivalence_check {
		($str:literal, $enum:expr) => {
			assert_eq!($enum.to_string(), $str);
			assert_eq!($str.parse(), Ok($enum));
		};
	}

	#[test]
	fn test_derived_display_and_fromstr() {
		display_fromstr_equivalence_check!("common", Category::Common);
		display_fromstr_equivalence_check!("bgcommon", Category::Bgcommon);
		display_fromstr_equivalence_check!("bg", Category::Bg);
		display_fromstr_equivalence_check!("cut", Category::Cut);
		display_fromstr_equivalence_check!("chara", Category::Chara);
		display_fromstr_equivalence_check!("ui", Category::Ui);
		display_fromstr_equivalence_check!("vfx", Category::Vfx);
		display_fromstr_equivalence_check!("ui_script", Category::UiScript);
		display_fromstr_equivalence_check!("exd", Category::Exd);
		display_fromstr_equivalence_check!("game_script", Category::GameScript);
		display_fromstr_equivalence_check!("music", Category::Music);
		display_fromstr_equivalence_check!("_sqpack_test", Category::SqpackTest);
		display_fromstr_equivalence_check!("_debug", Category::Debug);
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
