use super::*;

/// Span of text from an input [`Source`].
///
/// Can be used to refer to a range of source text and provides methods for
/// reading the text.
#[derive(Default, Clone, Eq, PartialEq, Hash)]
pub struct Span {
	source: Source,
	offset: usize,
	length: usize,
	indent: usize,
	column: usize,
	line: usize,
}

impl Source {
	pub fn span(&self) -> Span {
		Span {
			source: self.clone(),
			offset: 0,
			length: self.len(),
			indent: 0,
			column: 0,
			line: 0,
		}
	}
}

impl Span {
	/// Return a [`Span`] spanning the entire range of the given spans.
	pub fn merge(a: Span, b: Span) -> Span {
		if a.offset() == 0 {
			b
		} else if b.offset() == 0 {
			a
		} else if a.source != b.source {
			Span::default()
		} else {
			let (mut a, b) = if b.offset < a.offset { (b, a) } else { (a, b) };
			a.length = (b.offset + b.length) - a.offset;
			a
		}
	}

	/// Returns a [`Span`] combining the entire range of the given nodes.
	pub fn from_nodes<T: IntoIterator<Item = Node>>(nodes: T) -> Self {
		let mut nodes = nodes.into_iter();
		let sta = nodes.next().map(|x| x.span());
		let end = nodes.last().map(|x| x.span());
		let sta = sta.unwrap_or_default();
		let end = end.unwrap_or_default();
		Self::merge(sta, end)
	}

	/// Return a new span with the range from the current to the given span.
	pub fn to(self, other: Span) -> Span {
		Self::merge(self, other)
	}

	/// Return a span with zero-length from the current span position.
	pub fn pos(&self) -> Span {
		self.truncate(0)
	}

	/// Global offset for this span as given by the [`Source`] offset.
	pub fn source_range(&self) -> (usize, usize) {
		let sta = self.offset();
		let end = sta + self.length;
		(sta, end)
	}

	/// Global offset for this span's starting position. This is related to
	/// the [`Source`] offset.
	pub fn offset(&self) -> usize {
		self.source().offset() + self.offset
	}

	/// Tab-width for the source.
	pub fn tab_width(&self) -> usize {
		self.source.tab_width()
	}

	/// Indentation level of the span's starting position.
	///
	/// This is the column width for the leading space in the line for the
	/// current position.
	///
	/// When the span position is at the leading space, this will only consider
	/// the width up to the span's position. E.g., this will always be zero at
	/// the start of the line.
	pub fn indent(&self) -> usize {
		self.indent
	}

	/// Column width for the span's starting position.
	///
	/// This is zero at the start of the line and increments by one for each
	/// character, except for TAB, where it increments to the next multiple of
	/// the tab-width.
	pub fn column(&self) -> usize {
		self.column
	}

	/// [`Source`] for this span.
	pub fn source(&self) -> &Source {
		&self.source
	}

	/// Length of the span in bytes.
	pub fn len(&self) -> usize {
		self.length
	}

	/// Text for this span.
	pub fn text(&self) -> &str {
		let sta = self.offset;
		let end = sta + self.length;
		&self.source.text()[sta..end]
	}

	/// Text for this span as raw UTF-8 bytes.
	pub fn data(&self) -> &[u8] {
		self.text().as_bytes()
	}

	/// Return a copy of the current span truncated to the given byte length.
	pub fn truncate(&self, length: usize) -> Span {
		let mut span = self.clone();
		span.length = std::cmp::min(self.length, length);
		span
	}

	/// Return a string representation of this span's location.
	pub fn location(&self) -> Option<String> {
		let name = self.source().name();
		if name.len() > 0 {
			let (line, col) = self.line_column();
			let (line, col) = (line + 1, col + 1);
			let location = format!("{name}:{line}:{col}");
			Some(location)
		} else {
			None
		}
	}

	/// Return the zero-based line and column number for this span's starting
	/// location.
	pub fn line_column(&self) -> (usize, usize) {
		(self.line, self.column)
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Reading
	//----------------------------------------------------------------------------------------------------------------//

	/// True when at the end of the input.
	pub fn at_end(&self) -> bool {
		self.text().len() == 0
	}

	/// True when the span's position begins at the leading indentation
	/// for the line.
	pub fn is_indent(&self) -> bool {
		self.column == self.indent
	}

	/// Read the next character in the input and advance the cursor.
	pub fn read(&mut self) -> Option<char> {
		if let Some((char, size)) = self.next_char() {
			self.advance(size);
			Some(char)
		} else {
			None
		}
	}

	/// Read the next character in the input if it is the given character.
	pub fn read_if(&mut self, expected: char) -> bool {
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

	/// Return the next character in the input without advancing the cursor.
	pub fn next_char(&self) -> Option<(char, usize)> {
		let data = self.data();
		if data.len() > 0 {
			let size = char_size(data);
			let char = &data[..size];
			let char = std::str::from_utf8(char)
				.ok()
				.and_then(|x| x.chars().next())
				.unwrap_or('\u{FFFD}');
			Some((char, size))
		} else {
			None
		}
	}

	/// Advance the span starting position by a given byte length.
	pub fn advance(&mut self, length: usize) {
		let tab_width = self.tab_width();
		let data = self.source.text().as_bytes();
		let mut skip = &data[self.offset..self.offset + length];
		while skip.len() > 0 {
			let size = if let Some((char, size)) = check_space(skip) {
				let is_indent = self.is_indent();
				if char == '\t' {
					self.column += tab_width - (self.column % tab_width);
				} else {
					self.column += 1;
				}
				if is_indent {
					self.indent = self.column;
				}
				size
			} else if let Some(size) = check_line_break(skip) {
				self.column = 0;
				self.indent = 0;
				self.line += 1;
				size
			} else {
				self.column += 1;
				char_size(skip)
			};
			assert!(size > 0);
			self.offset += size;
			self.length -= size;
			skip = &skip[size..];
		}
	}

	/// Advance the span starting position by a given byte length, and returns
	/// a span corresponding to the range advanced.
	pub fn advance_span(&mut self, length: usize) -> Span {
		let start = self.clone();
		self.advance(length);
		self.span_from(&start)
	}

	/// Return a new span corresponding to the range from the given span to
	/// the current.
	///
	/// Both spans must be from the same source, and the given span must not
	/// be after the current.
	pub fn span_from(&self, other: &Span) -> Span {
		assert!(self.source == other.source && self.offset >= other.offset);
		let mut span = other.clone();
		span.length = self.offset - other.offset;
		span
	}
}

impl Debug for Span {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let name = self.source.name();
		if name.len() == 0 {
			return write!(f, "<span>");
		}

		let (line, column) = self.line_column();
		let (line, column) = (line + 1, column + 1);
		let length = self.len();

		let ctx = Context::get();
		let fmt = ctx.format();
		if fmt.mode() == Mode::Minimal {
			write!(f, "<L{line}:{column}")?;
			if length > 0 {
				write!(f, "+{length}")?;
			}
			write!(f, ">")
		} else {
			const HEAD: &str = "<span";
			const TAIL: &str = ">";
			let name = format!(" {name:?}");
			let location = format!(" L{line}:{column}");

			write!(f, "{HEAD}")?;
			if name != "" {
				write!(f, "{name}")?;
			}
			write!(f, "{location}")?;

			if length > 0 && fmt.mode() > Mode::Normal {
				let max_width = fmt.line_width();
				let width = HEAD.len() + TAIL.len() + name.len() + location.len();
				let text = self.text();
				if width + length + 5 <= max_width {
					write!(f, " = {text:?}")?;
				} else {
					const INDENT: &str = "    = ";
					write!(f, "\n{INDENT}")?;

					let max_width = max_width - INDENT.len() - 2;
					if length <= max_width {
						write!(f, "{text:?}")?;
					} else {
						let head = max_width * 60 / 100;
						let tail = max_width - head - 1;
						let head = text.chars().take(head).collect::<String>();
						let tail = text.chars().skip(text.len() - tail).collect::<String>();
						let text = format!("{head}…{tail}");
						write!(f, "{text:?}")?;
					}
					write!(f, "\n")?;
				}
			} else if length > 0 {
				write!(f, "+{length}")?;
			}
			write!(f, "{TAIL}")
		}
	}
}

impl Display for Span {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let name = self.source.name();
		if name.len() > 0 {
			let (line, column) = self.line_column();
			let (line, column) = (line + 1, column + 1);
			let ctx = Context::get();
			let fmt = ctx.format();

			let (line_end, column_end) = {
				let mut end = self.clone();
				end.advance(self.len());
				end.line_column()
			};
			let (line_end, column_end) = (line_end + 1, column_end + 1);

			write!(f, "{}", fmt.separator())?;
			match fmt.mode() {
				Mode::Minimal => {
					write!(f, "L{line}:{column}")?;
				}
				Mode::Normal => {
					write!(f, "{name}:{line}:{column}")?;
					if self.len() > 0 {
						write!(f, "…{line_end}:{column_end}")?;
					}
				}
				Mode::Detail => {
					write!(f, "{name} at L{line}:{column}")?;
					if self.len() > 0 {
						write!(f, " to L{line_end}:{column_end}")?;
					}
				}
			}
		}
		Ok(())
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn span_format() {
		let text = format!("{:?}", Span::default());
		assert_eq!(text, "<span>");

		let text = format!("{}", Span::default());
		assert_eq!(text, "");

		let minimal = Format::default().with_mode(Mode::Minimal);
		let normal = Format::default().with_mode(Mode::Normal);
		let detail = Format::default().with_mode(Mode::Detail).with_line_width(35);

		let ctx = Context::get();
		let prefix = "\n\n\n\t\t\t\t";
		let s = ctx.load_source_text("small.txt", format!("{prefix}123456"));
		let m = ctx.load_source_text("medium.txt", format!("{prefix}somewhat larger text"));
		let l = ctx.load_source_text(
			"large.txt",
			format!("{prefix}[the quick brown fox jumps over the lazy dog]"),
		);

		let mut s = s.span();
		let mut m = m.span();
		let mut l = l.span();

		s.advance(prefix.len());
		m.advance(prefix.len());
		l.advance(prefix.len());

		ctx.with_format(minimal.with_separator("at "), || {
			let text = format!("{s}");
			assert_eq!(text, "at L4:17");
		});

		ctx.with_format(minimal, || {
			let text = format!("{s:?}");
			assert_eq!(text, "<L4:17+6>");

			let text = format!("{s}");
			assert_eq!(text, "L4:17");

			let s = s.truncate(0);
			let text = format!("{s:?}");
			assert_eq!(text, "<L4:17>");
		});

		ctx.with_format(normal, || {
			let text = format!("{s:?}");
			assert_eq!(text, "<span \"small.txt\" L4:17+6>");

			let text = format!("{s}");
			assert_eq!(text, "small.txt:4:17…4:23");

			let s = s.truncate(0);
			let text = format!("{s}");
			assert_eq!(text, "small.txt:4:17");

			let text = format!("{m:?}");
			assert_eq!(text, "<span \"medium.txt\" L4:17+20>");

			let text = format!("{l:?}");
			assert_eq!(text, "<span \"large.txt\" L4:17+45>");
		});

		ctx.with_format(detail, || {
			let text = format!("{s:?}");
			assert_eq!(text, "<span \"small.txt\" L4:17 = \"123456\">");

			let text = format!("{s}");
			assert_eq!(text, "small.txt at L4:17 to L4:23");

			let s = s.truncate(0);
			let text = format!("{s}");
			assert_eq!(text, "small.txt at L4:17");

			let text = format!("{m:?}");
			assert_eq!(text, "<span \"medium.txt\" L4:17\n    = \"somewhat larger text\"\n>");

			let text = format!("{l:?}");
			assert_eq!(
				text,
				"<span \"large.txt\" L4:17\n    = \"[the quick brown… lazy dog]\"\n>"
			);
		});
	}

	#[test]
	fn reading() -> Result<()> {
		let context = Context::get();
		let input = context.load_source_text("input A", "123456");

		let mut cursor = input.span();
		assert_eq!(cursor.read(), Some('1'));
		assert_eq!(cursor.read(), Some('2'));
		assert_eq!(cursor.read(), Some('3'));

		let pos = cursor.clone();

		assert_eq!(cursor.read(), Some('4'));
		assert_eq!(cursor.read_if('!'), false);
		assert_eq!(cursor.read_if('5'), true);

		assert_eq!(cursor.span_from(&pos).text(), "45");

		assert_eq!(cursor.read(), Some('6'));
		assert_eq!(cursor.read(), None);
		assert_eq!(cursor.at_end(), true);

		Ok(())
	}
}
