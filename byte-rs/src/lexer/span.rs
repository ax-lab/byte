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
