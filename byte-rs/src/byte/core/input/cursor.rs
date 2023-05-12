use super::*;

/// Cursor indexes a position in an [`Input`] and provides methods for reading
/// characters from that position forward.
#[derive(Clone)]
pub struct Cursor {
	src: Input,
	pos: usize,
	end: usize,
	location: Location,
}

impl Cursor {
	pub(crate) fn new(src: Input, pos: usize, end: usize, location: Location) -> Self {
		assert!(end <= src.len());
		assert!(pos <= end);
		Cursor {
			src,
			pos,
			end,
			location,
		}
	}

	/// Location for this cursor.
	pub fn location(&self) -> Location {
		self.location
	}

	/// Source input for this cursor.
	pub fn input(&self) -> &Input {
		&self.src
	}

	/// Span from this cursor to the end of the input.
	pub fn span(&self) -> Span {
		Span::new(self.src.clone(), self.pos, self.end, self.location)
	}

	/// Byte offset for the current position from the start of the input.
	pub fn offset(&self) -> usize {
		self.pos
	}

	/// True when not at the end of the input.
	pub fn has_next(&self) -> bool {
		self.pos < self.end
	}

	/// True at the end of the input.
	pub fn at_end(&self) -> bool {
		!self.has_next()
	}

	/// Read the next character in the input and move the cursor forward.
	pub fn read(&mut self) -> Option<char> {
		let text = self.src.range(self.pos..);
		if let Some(next) = text.chars().next() {
			// update offset to next char
			self.pos += next.len_utf8();

			// translate CR and CR+LF to a single LF
			let next = if next == '\r' && text.len() > 1 {
				if text.as_bytes()[1] == '\n' as u8 {
					self.pos += 1;
				}
				'\n'
			} else {
				next
			};

			// update the location
			self.location.advance(next);

			Some(next)
		} else {
			None
		}
	}

	/// Peek at the next character in the input without changing the cursor.
	pub fn peek(&self) -> Option<char> {
		let mut copy = self.clone();
		copy.read()
	}

	/// Read the next character only if it's the given expected character,
	/// otherwise the cursor is not changed.
	pub fn try_read(&mut self, expected: char) -> bool {
		let mut copy = self.clone();
		if let Some(next) = copy.read() {
			if next == expected {
				*self = copy;
				return true;
			}
		}
		false
	}

	pub fn skip_while<P: Fn(char) -> bool>(&mut self, predicate: P) {
		let mut copy = self.clone();
		while let Some(next) = self.read() {
			if predicate(next) {
				copy = self.clone();
			} else {
				break;
			}
		}
		*self = copy;
	}

	pub fn skip_spaces(&mut self) {
		self.skip_while(is_space);
	}
}

//====================================================================================================================//
// Traits
//====================================================================================================================//

impl std::fmt::Display for Cursor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(name) = self.src.name() {
			write!(f, "{}:{}", name, self.location)
		} else {
			write!(f, "{}", self.location)
		}
	}
}

impl std::fmt::Debug for Cursor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(name) = self.src.name() {
			write!(f, "<cursor {}:{:?}>", name, self.location)
		} else {
			write!(f, "<cursor {:?}>", self.location)
		}
	}
}

impl PartialEq for Cursor {
	fn eq(&self, other: &Self) -> bool {
		self.src == other.src && self.pos == other.pos
	}
}

impl Eq for Cursor {}
