use std::{
	ops::{Range, RangeBounds},
	sync::Arc,
};

#[derive(Clone)]
pub enum Str {
	Empty,
	Static(&'static str),
	Shared(Arc<String>),
}

impl Str {
	pub fn as_str(&self) -> &str {
		match self {
			Str::Empty => "",
			Str::Static(str) => str,
			Str::Shared(str) => &str,
		}
	}

	pub fn len(&self) -> usize {
		match self {
			Str::Empty => 0,
			Str::Static(str) => str.len(),
			Str::Shared(str) => str.len(),
		}
	}

	pub fn compute_range<R: RangeBounds<usize>>(range: R, len: usize) -> Range<usize> {
		let sta = match range.start_bound() {
			std::ops::Bound::Included(n) => *n,
			std::ops::Bound::Excluded(n) => *n + 1,
			std::ops::Bound::Unbounded => 0,
		};
		let end = match range.end_bound() {
			std::ops::Bound::Included(n) => *n + 1,
			std::ops::Bound::Excluded(n) => *n,
			std::ops::Bound::Unbounded => len,
		};
		sta..end
	}

	pub fn range<R: RangeBounds<usize>>(&self, range: R) -> &str {
		let str = self.as_str();
		&str[Self::compute_range(range, self.len())]
	}
}

impl Default for Str {
	fn default() -> Self {
		Str::Empty
	}
}

impl std::fmt::Display for Str {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl std::fmt::Debug for Str {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.as_str())
	}
}

impl From<&'static str> for Str {
	fn from(value: &'static str) -> Self {
		Self::Static(value)
	}
}

impl From<String> for Str {
	fn from(value: String) -> Self {
		Self::Shared(Arc::new(value))
	}
}

impl PartialEq for Str {
	fn eq(&self, other: &Self) -> bool {
		self.as_str() == other.as_str()
	}
}

impl PartialEq<&str> for Str {
	fn eq(&self, other: &&str) -> bool {
		self.as_str() == *other
	}
}

impl PartialEq<Str> for &str {
	fn eq(&self, other: &Str) -> bool {
		other.as_str() == *self
	}
}

impl PartialEq<String> for Str {
	fn eq(&self, other: &String) -> bool {
		self.as_str() == other
	}
}

impl PartialEq<Str> for String {
	fn eq(&self, other: &Str) -> bool {
		other.as_str() == self
	}
}

impl Eq for Str {}

impl AsRef<str> for Str {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn str_empty() {
		assert_eq!(Str::Empty, Str::Empty);
		assert_eq!(Str::Empty, Str::Static(""));
		assert_eq!(Str::Empty, Str::from(""));
		assert_eq!(Str::Empty, Str::from(String::new()));
		assert_eq!(Str::Empty, "");
		assert_eq!(Str::Empty, String::new());
		assert_eq!("", Str::Empty);
		assert_eq!(String::new(), Str::Empty);
	}

	#[test]
	fn str_eq() {
		let a1 = Str::Static("123");
		let a2 = Str::from("123");
		let a3 = Str::from(String::from("123"));

		assert_eq!(a1, "123");
		assert_eq!(a1, String::from("123"));
		assert_eq!(a1, a1);
		assert_eq!(a1, a2);
		assert_eq!(a1, a3);

		assert_eq!(a2, "123");
		assert_eq!(a2, String::from("123"));
		assert_eq!(a2, a1);
		assert_eq!(a2, a2);
		assert_eq!(a2, a3);

		assert_eq!(a3, "123");
		assert_eq!(a3, String::from("123"));
		assert_eq!(a3, a1);
		assert_eq!(a3, a2);
		assert_eq!(a3, a3);

		let s = "123";
		assert_eq!(s, a1);
		assert_eq!(s, a2);
		assert_eq!(s, a3);

		let s = String::from("123");
		assert_eq!(s, a1);
		assert_eq!(s, a2);
		assert_eq!(s, a3);

		assert_eq!(a1, a1.clone());
		assert_eq!(a2, a2.clone());
		assert_eq!(a3, a3.clone());
	}
}
