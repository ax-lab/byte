use std::{ops::RangeBounds, path::Path, sync::Arc};

mod cursor;
mod span;

pub use cursor::*;
pub use span::*;

/// Tab-width considered to calculate the next tab stop after a TAB character.
///
/// This is used when computing column and indentation values.
pub const TAB_WIDTH: usize = 4;

/// Returns true for a whitespace character, as considered by the language.
pub fn is_space(char: char) -> bool {
	matches!(char, ' ' | '\t')
}

/// Input file or text for the compiler.
#[derive(Clone)]
pub struct Input(Arc<InputData>);

impl Input {
	pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
		let path = path.as_ref();
		let data = std::fs::read(path)?;
		let name = path.to_string_lossy().to_string();
		let data = InputData { name, data };
		Ok(Input(data.into()))
	}

	pub fn new<S1: Into<String>, S2: Into<String>>(name: S1, data: S2) -> Self {
		let data: String = data.into();
		Input(Arc::new(InputData {
			name: name.into(),
			data: data.into_bytes(),
		}))
	}

	pub fn name(&self) -> &str {
		&self.0.name
	}

	pub fn len(&self) -> usize {
		self.0.data.len()
	}

	pub fn start(&self) -> Cursor {
		todo!()
	}

	pub fn bytes<R: RangeBounds<usize>>(&self, span: R) -> &[u8] {
		let sta = match span.start_bound() {
			std::ops::Bound::Included(n) => *n,
			std::ops::Bound::Excluded(n) => *n + 1,
			std::ops::Bound::Unbounded => 0,
		};
		let end = match span.end_bound() {
			std::ops::Bound::Included(n) => *n + 1,
			std::ops::Bound::Excluded(n) => *n,
			std::ops::Bound::Unbounded => self.len(),
		};
		&self.0.data[sta..end]
	}

	pub fn text<R: RangeBounds<usize>>(&self, span: R) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.bytes(span)) }
	}
}

#[derive(Default)]
struct InputData {
	name: String,
	data: Vec<u8>,
}

//====================================================================================================================//
// Traits and helper code
//====================================================================================================================//

// Default empty value

impl Default for Input {
	fn default() -> Self {
		Input::new("(empty)", "")
	}
}

// Conversion from strings

impl<T: Into<String>> From<T> for Input {
	fn from(data: T) -> Self {
		Input::new("string", data)
	}
}

// Identity equality

impl PartialEq for Input {
	fn eq(&self, other: &Self) -> bool {
		std::ptr::eq(self.0.as_ref(), other.0.as_ref())
	}
}

impl Eq for Input {}

// Debug

impl std::fmt::Debug for Input {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<input `{}` with {} bytes>", self.name(), self.len())
	}
}

impl std::fmt::Display for Input {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name())
	}
}
