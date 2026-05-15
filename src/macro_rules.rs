#[cfg(test)]
#[macro_use]
mod test {
	#[macro_export]
	macro_rules! assert_display_fromstr_equivalent {
		($str:expr, $enum:expr) => {
			assert_eq!($enum.to_string(), $str);
			assert_eq!($str.parse(), Ok($enum));
		};
	}

	#[macro_export]
	macro_rules! assert_fromrepr_reprfrom_equivalent {
		(($repr:literal, $variant:expr): ($repr_ty:ident, $enum_ty:ident)) => {
			assert_eq!($enum_ty::from_repr($repr), Some($variant));
			assert_eq!($repr_ty::from($variant), $repr);
		};
	}

	// #[macro_export]
	// macro_rules! assert_parse_error_matches_condition {
	// 	($value:tt->$parse_ty:ty, $pat:pat => $condition:expr$(,)?) => {
	// 		assert!(($value).parse::<$parse_ty>().is_err_and(|err| match err {
	// 			$pat => $condition,
	// 			_ => false,
	// 		}));
	// 	};
	// }

	#[macro_export]
	macro_rules! assert_parse_error_matches_expr {
		($value:tt->$parse_ty:ty, $pat:pat => $eq:expr$(,)?) => {
			let result = ($value).parse::<$parse_ty>();
			match result {
				Err(err) => match err {
					$pat => $eq,
					#[allow(unused)]
					_ => assert!(matches!(err, $pat)),
				},
				Ok(_) => assert!(result.is_err()),
			}
		};
	}
}
