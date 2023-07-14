use super::*;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StringValue(Arc<String>);

impl StringValue {
	pub fn new<T: Into<String>>(str: T) -> Self {
		let str = str.into();
		Self(Arc::new(str))
	}

	pub fn new_from_arc(str: Arc<String>) -> Self {
		Self(str)
	}

	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}

	pub fn len(&self) -> usize {
		self.as_str().len()
	}
}

impl Display for StringValue {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl Debug for StringValue {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self.0)
	}
}

impl<T: Into<String>> From<T> for StringValue {
	fn from(value: T) -> Self {
		StringValue(value.into().into())
	}
}
