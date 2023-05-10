use std::{ops::RangeBounds, path::Path, sync::Arc};

use super::*;

mod cursor;
mod location;
mod span;

pub use cursor::*;
pub use location::*;
pub use span::*;

/// Tab-width considered to calculate the next tab stop after a TAB character.
///
/// This is used when computing column and indentation values.
pub const TAB_WIDTH: usize = 4;

/// Returns true for a whitespace character, as considered by the language.
pub fn is_space(char: char) -> bool {
	matches!(char, ' ' | '\t')
}

/// Input source file or text.
#[derive(Clone)]
pub enum Input {
	File(Arc<InputFile>),
	Text(Str),
}

impl Input {
	/// Open a file.
	pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
		let path = path.as_ref();
		let data = std::fs::read(path)?;
		let name = path.to_string_lossy().to_string();
		let data = InputFile { name, data };
		Ok(Input::File(data.into()))
	}

	/// Create a new named input from a string.
	pub fn new<S1: Into<String>, S2: Into<String>>(name: S1, data: S2) -> Self {
		let name: String = name.into();
		let data: String = data.into();
		assert!(name.len() > 0);
		Input::File(
			InputFile {
				name: name.into(),
				data: data.into_bytes(),
			}
			.into(),
		)
	}

	/// Input name, if opened from a file or other named source.
	pub fn name(&self) -> Option<&str> {
		if let Input::File(file) = self {
			Some(file.name.as_str())
		} else {
			None
		}
	}

	/// Input length in bytes.
	pub fn len(&self) -> usize {
		match self {
			Input::File(file) => file.data.len(),
			Input::Text(text) => text.len(),
		}
	}

	/// Return a span for the entire input.
	pub fn span(&self) -> Span {
		Span::from_input(self)
	}

	/// Return a cursor to the beginning of the input.
	pub fn cursor(&self) -> Cursor {
		self.span().cursor()
	}

	/// Returns a range of text from the input.
	pub fn range<R: RangeBounds<usize>>(&self, range: R) -> &str {
		let text = self.text();
		let range = Str::compute_range(range, text.len());
		&text[range]
	}

	/// Returns the entire input text.
	pub fn text(&self) -> &str {
		match self {
			Input::File(file) => unsafe { std::str::from_utf8_unchecked(&file.data) },
			Input::Text(text) => text.as_str(),
		}
	}
}

impl<T: Into<Str>> From<T> for Input {
	fn from(value: T) -> Self {
		Input::Text(value.into())
	}
}

#[derive(Default)]
pub struct InputFile {
	name: String,
	data: Vec<u8>,
}

//====================================================================================================================//
// Traits and helper code
//====================================================================================================================//

// Identity equality

impl PartialEq for Input {
	fn eq(&self, other: &Self) -> bool {
		match self {
			Input::File(file) => {
				if let Input::File(other) = other {
					std::ptr::eq(file.as_ref(), other.as_ref())
				} else {
					false
				}
			}
			Input::Text(text) => {
				if let Input::Text(other) = other {
					text == other
				} else {
					false
				}
			}
		}
	}
}

impl Eq for Input {}

// Debug

impl std::fmt::Debug for Input {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<input ")?;
		if let Some(name) = self.name() {
			write!(f, "`{}` ", name)?;
		} else {
			write!(f, "text ")?;
		}
		write!(f, "with {} bytes>", self.len())
	}
}

impl std::fmt::Display for Input {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(name) = self.name() {
			write!(f, "{}", name)
		} else if self.len() == 0 {
			write!(f, "(empty)")
		} else {
			write!(f, "")
		}
	}
}
