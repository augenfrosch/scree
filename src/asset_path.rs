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
		let (category, rest) = self.inner.split_once('/').unwrap_or(("", self.inner));
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
