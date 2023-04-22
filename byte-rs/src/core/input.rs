use std::{io::Write, path::Path, sync::Arc};

use crate::core::repr::*;

/// Tab-width considered when computing column and indentation.
pub const TAB_WIDTH: usize = 4;

/// This is used for the lexer to determined what is a whitespace character.
pub fn is_space(char: char) -> bool {
	matches!(char, ' ' | '\t')
}

/// Input text stream for the compiler. Can be cloned and shared with low
/// overhead.
#[derive(Clone)]
pub struct Input {
	internal: Arc<InputData>,
}

impl Input {
	/// Open a file as input.
	pub fn open_file<P: AsRef<Path>>(path: P) -> std::io::Result<Input> {
		let path = path.as_ref();
		let data = std::fs::read(path)?;
		let name = path.to_string_lossy().to_string();
		let data = InputData { name, data };

		Ok(Input {
			internal: data.into(),
		})
	}

	/// Open a plain string as input. The string is copied.
	pub fn open_str<S: AsRef<str>>(name: &str, text: S) -> Input {
		let text = text.as_ref().to_string();
		let data = InputData {
			name: name.to_string(),
			data: text.as_bytes().into(),
		};
		Input {
			internal: data.into(),
		}
	}
}

fmt_from_repr!(Input);

impl HasRepr for Input {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		let debug = output.is_debug();
		let full = if debug {
			output.format() == ReprFormat::Full
		} else {
			false
		};
		if full {
			write!(output, "Input({} -- {} bytes)", self.name(), self.len())
		} else if debug {
			write!(output, "Input({})", self.name())
		} else {
			write!(output, "{}", self.name())
		}
	}
}

struct InputData {
	name: String,
	data: Vec<u8>,
}

impl Input {
	pub fn name(&self) -> &str {
		&self.internal.name
	}

	pub fn len(&self) -> usize {
		self.internal.data.len()
	}

	pub fn start(&self) -> Cursor {
		Cursor {
			src: self.clone(),
			row: 0,
			col: 0,
			offset: 0,
			indent: 0,
		}
	}

	pub fn bytes(&self, span: &Span) -> &[u8] {
		&self.internal.data[span.sta.offset..span.end.offset]
	}

	pub fn text(&self, span: &Span) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.bytes(span)) }
	}
}

impl PartialEq for Input {
	fn eq(&self, other: &Self) -> bool {
		let a = &*self.internal;
		let b = &*other.internal;
		std::ptr::eq(a, b)
	}
}

impl Eq for Input {}

/// Span indexes a range of text from an [`Input`].
#[derive(Clone, Eq, PartialEq)]
pub struct Span {
	pub sta: Cursor,
	pub end: Cursor,
}

impl Span {
	pub fn from_range(a: Option<Span>, b: Option<Span>) -> Option<Span> {
		if a.is_none() && b.is_none() {
			None
		} else if let Some(a) = a {
			if let Some(b) = b {
				let (a, b) = if a.sta.offset() > b.sta.offset() {
					(b, a)
				} else {
					(a, b)
				};
				let sta = a.sta.clone();
				let end = b.end.clone();
				Some(Span { sta, end })
			} else {
				let sta = a.sta.clone();
				Some(Span {
					sta: sta.clone(),
					end: sta,
				})
			}
		} else {
			Span::from_range(b, None)
		}
	}

	pub fn src(&self) -> &Input {
		self.sta.src()
	}

	pub fn text(&self) -> &str {
		self.sta.src.text(self)
	}
}

fmt_from_repr!(Span);

impl HasRepr for Span {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		let debug = output.is_debug();
		if debug {
			write!(output, "Span(")?;
		}

		if output.format() > ReprFormat::Compact {
			write!(output, "{} at ", self.src().name())?;
		}
		self.sta.output_repr(&mut output.compact().display())?;
		if self.end != self.sta && output.format() > ReprFormat::Minimal {
			let _ = write!(output, "…");
			self.end.output_repr(&mut output.minimal().display())?;
		}
		if debug {
			write!(output, ")")?;
		}
		Ok(())
	}
}

/// Cursor indexes a position in an [`Input`] and provides methods for reading
/// characters from that position forward.
///
/// This type is lightweight and can be easily copied to save and backtrack to
/// an input position.
#[derive(Clone)]
pub struct Cursor {
	src: Input,
	row: usize,
	col: usize,
	offset: usize,
	indent: usize,
}

impl Cursor {
	/// Source input.
	pub fn src(&self) -> &Input {
		&self.src
	}

	pub fn row(&self) -> usize {
		self.row
	}

	pub fn col(&self) -> usize {
		self.col
	}

	/// Byte offset from the start of the input.
	pub fn offset(&self) -> usize {
		self.offset
	}

	/// Indentation level.
	///
	/// This is the width of the leading whitespace in the current line up to
	/// the current position.
	///
	/// For the purposes of indentation, TAB will expand the width to the next
	/// multiple of the [`TAB_WIDTH`] and other whitespace count as one.
	pub fn indent(&self) -> usize {
		self.indent
	}

	pub fn at_end(&self) -> bool {
		self.offset == self.src.len()
	}

	pub fn read(&mut self) -> Option<char> {
		let text = unsafe { std::str::from_utf8_unchecked(&self.src.internal.data[self.offset..]) };
		if let Some(next) = text.chars().next() {
			// keep indentation until we find the first non-space character
			let is_line_indent = self.indent == self.col;

			// update offset to next char
			self.offset += next.len_utf8();

			// translate CR and CR+LF to a single LF
			let next = if next == '\r' {
				let mut next = self.clone();
				if next.read() == Some('\n') {
					*self = next;
				}
				'\n'
			} else {
				next
			};

			// update position
			if next == '\n' {
				self.row += 1;
				self.col = 0;
			} else if next == '\t' {
				self.col += TAB_WIDTH - (self.col % TAB_WIDTH)
			} else {
				self.col += 1;
			}

			// update indentation
			if next == '\n' || (is_space(next) && is_line_indent) {
				self.indent = self.col;
			}
			Some(next)
		} else {
			None
		}
	}

	//------------------------------------------------------------------------//
	// Utility functions
	//------------------------------------------------------------------------//

	pub fn read_if(&mut self, expected: char) -> bool {
		let mut cursor = self.clone();
		if cursor.read() == Some(expected) {
			*self = cursor;
			true
		} else {
			false
		}
	}

	pub fn pos(&self) -> Span {
		Span {
			sta: self.clone(),
			end: self.clone(),
		}
	}
}

impl PartialEq for Cursor {
	fn eq(&self, other: &Self) -> bool {
		self.src == other.src && self.offset == other.offset
	}
}

impl Eq for Cursor {}

fmt_from_repr!(Cursor);

impl HasRepr for Cursor {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		let line = self.row() + 1;
		let column = self.col() + 1;
		if !output.is_debug() && output.format() > ReprFormat::Compact {
			write!(output, "line {line}, column {column}")
		} else {
			if output.format() > ReprFormat::Minimal {
				write!(output, "L")?;
			}
			write!(output, "{line}:{column:02}")
		}
	}
}
