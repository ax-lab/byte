use std::ops::Range;

use super::*;

//====================================================================================================================//
// Input
//====================================================================================================================//

/// Default tab width across the compiler.
///
/// This affects reported column numbers and the computed indentation values.
pub const DEFAULT_TAB_WIDTH: usize = 4;

/// Generic input source.
#[derive(Clone)]
pub struct Input {
	data: Arc<(String, Vec<u8>)>,
	tab_width: usize,
	line: usize,
	column: usize,
	indent: usize,
}

impl Input {
	pub fn new<T: Into<String>>(name: T, data: Vec<u8>) -> Self {
		let name = name.into();
		let data = Arc::new((name, data));
		Self {
			data,
			tab_width: DEFAULT_TAB_WIDTH,
			line: 0,
			column: 0,
			indent: 0,
		}
	}

	pub fn with_pos(self, line: usize, column: usize, indent: usize) -> Self {
		let mut input = self;
		input.line = line - 1;
		input.column = column - 1;
		input.indent = indent;
		input
	}

	pub fn start(&self) -> Cursor {
		Cursor {
			source: self.clone(),
			line: self.line,
			column: self.column,
			indent: self.indent,
			offset: 0,
		}
	}

	pub fn len(&self) -> usize {
		self.data().len()
	}

	pub fn name(&self) -> &str {
		&self.data.0
	}

	pub fn data(&self) -> &[u8] {
		&self.data.1
	}

	pub fn text(&self) -> &str {
		std::str::from_utf8(self.data()).unwrap()
	}

	pub fn tab_width(&self) -> usize {
		self.tab_width
	}
}

impl Debug for Input {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let name = self.name();
		let len = self.len();
		if len <= 80 {
			write!(f, "<Input {name}: {:?}>", self.text())
		} else {
			write!(f, "<Input {name} ({} bytes)>", len)
		}
	}
}

impl PartialEq for Input {
	fn eq(&self, other: &Self) -> bool {
		if Arc::as_ptr(&self.data) == Arc::as_ptr(&other.data) {
			self.line == other.line
				&& self.column == other.column
				&& self.indent == other.indent
				&& self.tab_width == other.tab_width
		} else {
			false
		}
	}
}

impl Eq for Input {}

//====================================================================================================================//
// Cursor
//====================================================================================================================//

/// Indexes a position in an [`Input`] and provides methods for reading it.
#[derive(Clone)]
pub struct Cursor {
	source: Input,
	line: usize,
	column: usize,
	indent: usize,
	offset: usize,
}

impl Cursor {
	pub fn source(&self) -> &Input {
		&self.source
	}

	pub fn with_pos(&self, line: usize, column: usize, indent: usize) -> Self {
		let mut cursor = self.clone();
		cursor.line = line - 1;
		cursor.column = column - 1;
		cursor.indent = indent;
		cursor
	}

	pub fn at_end(&self) -> bool {
		self.data().len() == 0
	}

	pub fn data(&self) -> &[u8] {
		let data = self.source().data();
		&data[self.offset..]
	}

	pub fn line(&self) -> usize {
		self.line + 1
	}

	pub fn is_indent(&self) -> bool {
		self.column == self.indent
	}

	pub fn column(&self) -> usize {
		self.column + 1
	}

	pub fn indent(&self) -> usize {
		self.indent
	}

	pub fn offset(&self) -> usize {
		self.offset
	}

	pub fn read(&mut self) -> Option<char> {
		if let Some((char, size)) = self.next_char() {
			self.advance(size);
			Some(char)
		} else {
			None
		}
	}

	pub fn try_read(&mut self, expected: char) -> bool {
		if let Some((next, size)) = self.next_char() {
			if next == expected {
				self.advance(size);
				true
			} else {
				false
			}
		} else {
			false
		}
	}

	pub fn next_char(&self) -> Option<(char, usize)> {
		let data = self.source().data();
		let offset = self.offset;
		if offset < data.len() {
			let size = char_size(&data[offset..]);
			let char = &data[offset..offset + size];
			let char = std::str::from_utf8(char)
				.ok()
				.and_then(|x| x.chars().next())
				.unwrap_or('\u{FFFD}');
			Some((char, size))
		} else {
			None
		}
	}

	pub fn data_from<'a>(&self, start: &'a Cursor) -> &'a [u8] {
		assert!(self.offset() > start.offset());
		let offset = self.offset() - start.offset();
		let data = start.data();
		&data[..offset]
	}

	pub fn advance(&mut self, length: usize) {
		let tab_width = self.source().tab_width();
		let data = self.source().data();

		let mut skip = &data[self.offset..(self.offset + length)];
		let mut line = self.line;
		let mut column = self.column;
		let mut indent = self.indent;
		while skip.len() > 0 {
			let size = if let Some((char, size)) = check_space(skip) {
				let is_indent = column == indent;
				if char == '\t' {
					column += tab_width - (self.column % tab_width);
				} else {
					column += 1;
				}
				if is_indent {
					indent = column;
				}
				size
			} else if let Some(size) = check_line_break(skip) {
				column = 0;
				indent = 0;
				line += 1;
				size
			} else {
				column += 1;
				char_size(skip)
			};
			assert!(size > 0);
			skip = &skip[size..];
		}
		self.line = line;
		self.column = column;
		self.indent = indent;
		self.offset += length;
	}

	pub fn advance_span(&mut self, length: usize) -> Span {
		let start = self.clone();
		self.advance(length);
		self.span_from(&start)
	}

	pub fn span_from(&self, start: &Cursor) -> Span {
		let pos = Location {
			line: start.line(),
			column: start.column(),
		};
		let end = Location {
			line: self.line(),
			column: self.column(),
		};
		let offset = start.offset()..self.offset();

		let source = self.source().clone();
		Span {
			offset,
			pos,
			end,
			source,
		}
	}

	pub fn span(&self) -> Span {
		self.span_from(self)
	}
}

//====================================================================================================================//
// Span and location
//====================================================================================================================//

/// Provides information about the source code span for an arbitrary value.
///
/// This is used to trace and report source and line information across the
/// compiler (e.g. for error messages).
///
/// The location includes information such as the input name, file path, line
/// and column numbers, source text, offset, length.
///
/// Not all values have an associated location, and not all information may be
/// available for any given [`Span`].
#[derive(Clone)]
pub struct Span {
	source: Input,
	offset: Range<usize>,
	pos: Location,
	end: Location,
}

impl Span {
	/// Source for the span.
	pub fn source(&self) -> &Input {
		&self.source
	}

	/// Byte offset and range for the span source location.
	pub fn offset(&self) -> Range<usize> {
		self.offset.clone()
	}

	/// Line and column info for the span start position.
	pub fn pos(&self) -> Location {
		self.pos.clone()
	}

	/// Length in bytes of this span.
	pub fn len(&self) -> usize {
		self.offset.end - self.offset.start
	}

	/// Line and column info for the span exclusive end position.
	pub fn end(&self) -> Location {
		self.end.clone()
	}

	/// Returns the full source text corresponding to the span.
	pub fn text(&self) -> &str {
		let data = self.source.data();
		let data = &data[self.offset.start..self.offset.end];
		std::str::from_utf8(data).unwrap()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Format helpers and methods
	//----------------------------------------------------------------------------------------------------------------//

	/// Format the span as a simple `line:col` format with the given separator.
	pub fn format_pos(&self, separator: &str, output: &mut dyn Write) -> std::fmt::Result {
		let pos = self.pos();
		let line = pos.line();
		let column = pos.column();
		write!(output, "{separator}{line}:{column}")
	}

	/// Format the span as a `start:col…end:col` format with the given separator.
	pub fn format_range(&self, separator: &str, output: &mut dyn Write) -> std::fmt::Result {
		self.format_pos(separator, output)?;
		if self.end != self.pos {
			let line = self.end.line();
			let column = self.end.column();
			write!(output, "…{line}:{column}")?;
		}
		Ok(())
	}

	/// Format similar to `format_pos` but including the source name.
	pub fn format_name_pos(&self, separator: &str, output: &mut dyn Write) -> std::fmt::Result {
		let name = self.source().name();
		write!(output, "{separator}{name}")?;
		self.format_pos(":", output)?;
		Ok(())
	}

	/// Format including the full source and line information available.
	///
	/// This is similar to `format_range` but includes the source name.
	pub fn format_full(&self, separator: &str, output: &mut dyn Write) -> std::fmt::Result {
		let name = self.source().name();
		write!(output, "{separator}{name}")?;
		self.format_range(":", output)?;
		Ok(())
	}

	/// Similar to `format_full`, but in a more verbose format that's ideal for
	/// messages.
	pub fn format_verbose(&self, separator: &str, output: &mut dyn std::fmt::Write) -> std::fmt::Result {
		let name = self.source().name();
		write!(output, "{separator}{name}")?;

		let pos = self.pos();
		let line = pos.line();
		let column = pos.column();
		write!(output, ", line {line}:{column}")?;

		if self.pos != self.end {
			let end = self.end();
			let line = end.line();
			let column = end.column();
			write!(output, " to {line}:{column}")?;
		}
		Ok(())
	}
}

/// Line and column information for a [`Span`].
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Location {
	line: usize,
	column: usize,
}

impl Location {
	/// The line number for this location, starting from one.
	pub fn line(&self) -> usize {
		assert!(self.line > 0);
		self.line
	}

	/// The column number for this location, starting from one.
	pub fn column(&self) -> usize {
		self.column
	}
}
