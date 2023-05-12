use super::*;

/// Represent a location from a source file.
///
/// The location is composed of the line, column, and indent values.
///
/// Line starts at one, but may be zero if not available or irrelevant. It is
/// provided only for debugging and use in error messages.
///
/// The column and indent are zero based. Both have semantical meaning in the
/// language.
#[derive(Copy, Clone, Default)]
pub struct Location {
	line: usize,
	column: usize,
	indent: usize,
}

impl Location {
	/// Gives a location at the start of a given line.
	///
	/// If line is zero, then the returned location won't contain the line
	/// number.
	pub fn at_line(line: usize) -> Self {
		Self {
			line,
			column: 0,
			indent: 0,
		}
	}

	/// Return the location but with the given line.
	pub fn with_line(mut self, line: usize) -> Self {
		self.line = line;
		self
	}

	/// Advance the location position given the next character in the input.
	pub fn advance(&mut self, next: char) {
		let is_leading_space = self.is_indent();

		// update position
		if next == '\n' {
			if self.line > 0 {
				self.line += 1;
			}
			self.column = 0;
		} else if next == '\t' {
			self.column += TAB_WIDTH - (self.column % TAB_WIDTH)
		} else {
			self.column += 1;
		}

		// update indentation
		if next == '\n' || (is_space(next) && is_leading_space) {
			self.indent = self.column;
		}
	}

	/// Line number for this location, starting at one, if available.
	pub fn line(&self) -> Option<usize> {
		if self.line > 0 {
			Some(self.line)
		} else {
			None
		}
	}

	/// Column number for this location, starting at one.
	pub fn column(&self) -> usize {
		self.column + 1
	}

	/// Indentation level for the current location.
	///
	/// This is the width of the leading whitespace for the line, up to the
	/// current location.
	///
	/// Indentation is zero at the start of a new line and increments for each
	/// leading whitespace character in the line.
	///
	/// Once a non-whitespace character is found, indentation will remain
	/// constant throughout the rest of the line.
	///
	/// A tab character will increase the indentation to the next [`TAB_WIDTH`]
	/// multiple. Any other whitespace will increase indentation by one.
	pub fn indent(&self) -> usize {
		self.indent
	}

	/// True if the location contains the line number.
	pub fn has_line(&self) -> bool {
		self.line > 0
	}

	/// True if this location is at the start of a new line.
	pub fn is_line_start(&self) -> bool {
		self.column == 0
	}

	/// True if this location is part of the (possibly empty) indentation for
	/// the line.
	pub fn is_indent(&self) -> bool {
		self.column == self.indent
	}

	/// Output a location with an optional file name and label.
	pub fn output_location(
		&self,
		output: &mut Repr,
		name: Option<&str>,
		label: &str,
	) -> std::io::Result<()> {
		use std::io::Write;

		if name.is_some() || self.line > 0 {
			write!(output, "{label}")?;
		}
		if let Some(name) = name {
			write!(output, "{name}")?;
			if self.line > 0 {
				write!(output, ":")?;
			}
		}
		if self.line > 0 {
			write!(output, "{self}")?;
		}
		Ok(())
	}
}

impl std::fmt::Display for Location {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.line > 0 {
			write!(f, "{}:{}", self.line, self.column + 1)
		} else {
			write!(f, "")
		}
	}
}

impl std::fmt::Debug for Location {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{self}")?;
		if self.indent > 0 && (self.indent < self.column || self.line == 0) {
			write!(f, "(+{})", self.indent)?;
		}
		Ok(())
	}
}
