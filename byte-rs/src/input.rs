#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct Pos {
	pub line: usize,
	pub column: usize,
	pub offset: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Span {
	pub pos: Pos,
	pub end: Pos,
}

impl std::fmt::Display for Span {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.pos)
	}
}

impl std::fmt::Display for Pos {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:03},{:02}", self.line + 1, self.column + 1)
	}
}

/// Trait implemented by any input to a [super::Lexer].
pub trait Input {
	fn offset(&self) -> usize;
	fn set_offset(&mut self, pos: usize);

	fn read(&mut self) -> Option<char>;
	fn read_text(&self, pos: usize, end: usize) -> &str;
}
