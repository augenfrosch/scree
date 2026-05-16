use std::ops::Deref;

use crate::{Category, ParseEnumError, RepositoryType};

pub struct AssetPath<'a> {
	inner: &'a str,
}

impl AssetPath<'_> {
	pub fn category(&self) -> Result<Category, ParseEnumError> {
		self.category_repository_type().0
	}

	pub fn repository_type(&self) -> RepositoryType {
		self.category_repository_type().1
	}

	pub fn category_repository_type(&self) -> (Result<Category, ParseEnumError>, RepositoryType) {
		let (category, rest) = self.inner.split_once('/').unwrap_or((self.inner, ""));
		let (repository_type, _rest) = rest.split_once('/').unwrap_or(("", rest));
		(
			category.parse(),
			repository_type.parse().unwrap_or(RepositoryType::BaseGame),
		)
	}
}

impl<'a, T: Into<&'a str>> From<T> for AssetPath<'a> {
	fn from(value: T) -> Self {
		Self {
			inner: value.into(),
		}
	}
}

impl AsRef<str> for AssetPath<'_> {
	fn as_ref(&self) -> &str {
		self.inner
	}
}

impl<'a> Deref for AssetPath<'a> {
	type Target = &'a str;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

#[cfg(test)]
mod test {
	use std::num::NonZero;

	use super::{AssetPath, Category, ParseEnumError, RepositoryType};

	#[test]
	fn test_category_repository_type() {
		assert_eq!(
			AssetPath::from("exd/").category_repository_type(),
			(Ok(Category::Exd), RepositoryType::BaseGame)
		);
		assert_eq!(
			AssetPath::from("exd").category_repository_type(),
			(Ok(Category::Exd), RepositoryType::BaseGame)
		);
		assert_eq!(
			AssetPath::from("cut/ex5/sound/").category_repository_type(),
			(
				Ok(Category::Cut),
				RepositoryType::Expansion {
					number: NonZero::new(5).unwrap()
				}
			)
		);
	}

	#[test]
	fn test_category_repository_type_error() {
		assert_eq!(
			AssetPath::from("AAAA/BBBB").category_repository_type(),
			(
				Err(ParseEnumError("AAAA".to_string())),
				RepositoryType::BaseGame
			)
		);
		assert_eq!(
			AssetPath::from(r"bg\ex1").category_repository_type(),
			(
				Err(ParseEnumError(r"bg\ex1".to_string())),
				RepositoryType::BaseGame
			)
		);
		assert_eq!(
			AssetPath::from("cutex5/sound/").category_repository_type(),
			(
				Err(ParseEnumError("cutex5".to_string())),
				RepositoryType::BaseGame
			)
		);
	}
}
