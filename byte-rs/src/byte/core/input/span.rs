use std::io::Write;

use super::*;

/// Represents a range of unprocessed raw source code text, either from
/// an [`Input`] or from a string.
///
/// The [`Span`] is the defacto representation for any unparsed block of
/// text in the language.
///
/// Besides the text, the span also carries a [`Location`], which can be
/// used for debugging and messages, but also has semantical meaning in
/// the language.
#[derive(Clone)]
pub struct Span {
	src: Input,
	pos: usize,
	end: usize,
	location: Location,
}

has_traits!(Span);

impl Default for Span {
	fn default() -> Self {
		Self {
			src: Input::from(Str::Empty),
			pos: 0,
			end: 0,
			location: Location::default(),
		}
	}
}

impl Span {
	pub(crate) fn new(src: Input, pos: usize, end: usize, location: Location) -> Self {
		assert!(end <= src.len());
		assert!(pos <= end);
		Span {
			src,
			pos,
			end,
			location,
		}
	}

	/// Create a new span for the whole input.
	pub fn from_input(input: &Input) -> Span {
		Span {
			src: input.clone(),
			pos: 0,
			end: input.len(),
			location: Location::at_line(1),
		}
	}

	/// Create a new span for the range between the given cursors. The cursors
	/// must be from the same input.
	pub fn from(a: &Cursor, b: &Cursor) -> Span {
		let a = a.span();
		let b = b.span();
		assert!(a.src == b.src);
		Span {
			src: a.src,
			pos: std::cmp::min(a.pos, b.pos),
			end: std::cmp::max(a.pos, b.pos),
			location: if a.pos < b.pos {
				a.location
			} else {
				b.location
			},
		}
	}

	/// Returns the span without any line information.
	pub fn without_line(mut self) -> Span {
		self.location = self.location.with_line(0);
		self
	}

	/// Starting location for this span.
	pub fn location(&self) -> Location {
		self.location
	}

	/// Compute the end location for this span.
	pub fn end_location(&self) -> Location {
		let mut location = self.location;
		for char in self.text().chars() {
			location.advance(char)
		}
		location
	}

	/// Source input for this span.
	pub fn input(&self) -> &Input {
		&self.src
	}

	/// Return a new cursor from the start of this span.
	pub fn cursor(&self) -> Cursor {
		Cursor::new(self.src.clone(), self.pos, self.end, self.location)
	}

	/// Length of the text span in bytes.
	pub fn len(&self) -> usize {
		self.end - self.pos
	}

	/// Return a range of text from this span.
	pub fn range<R: RangeBounds<usize>>(&self, range: R) -> &str {
		let text = self.text();
		let range = Str::compute_range(range, text.len());
		&text[range]
	}

	/// Return the full text for this span.
	pub fn text(&self) -> &str {
		self.src.range(self.pos..self.end)
	}

	/// Merge two spans from the same input source.
	pub fn merge(a: Span, b: Span) -> Self {
		assert!(a.src == b.src);
		Span {
			src: a.src,
			pos: std::cmp::min(a.pos, b.pos),
			end: std::cmp::max(a.end, b.end),
			location: if a.pos < b.pos {
				a.location
			} else {
				b.location
			},
		}
	}
}

//====================================================================================================================//
// Traits
//====================================================================================================================//

impl HasRepr for Span {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		let debug = output.is_debug();
		if debug {
			write!(output, "<span ")?;
		}

		if output.format() > ReprFormat::Compact {
			if let Some(name) = self.src.name() {
				if name != "" {
					write!(output, "{name}:")?;
				}
			}
		}

		let pos = self.location();
		if output.is_display() {
			write!(output, "{pos}")?;
			if output.is_full() && self.len() > 0 && pos.has_line() {
				let end = self.end_location();
				write!(output, "…{end}")?;
			}
		} else {
			write!(output, "{pos:?}")?;
			if self.len() > 0 {
				if pos.has_line() {
					let end = self.end_location();
					write!(output, " end={end}")?;
				} else {
					write!(output, " len={}", self.len())?;
				}
			}
			write!(output, ">")?;
		}

		Ok(())
	}
}

impl std::fmt::Debug for Span {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.fmt_debug(f)
	}
}

impl std::fmt::Display for Span {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.text())
	}
}

impl PartialEq for Span {
	fn eq(&self, other: &Self) -> bool {
		self.text() == other.text()
	}
}

impl Eq for Span {}

//====================================================================================================================//
// Value extensions
//====================================================================================================================//

impl Value {
	pub fn get_span(&self) -> Option<Span> {
		self.get_field::<Span>().or_else(|| self.get()).cloned()
	}

	pub fn with_span(&self, span: Span) -> Value {
		self.with_field(span)
	}
}

pub trait ValueAtSpan {
	fn at(self, span: Span) -> Value;
}

impl<T: IsValue> ValueAtSpan for T {
	fn at(self, span: Span) -> Value {
		Value::from(self).with_span(span)
	}
}
