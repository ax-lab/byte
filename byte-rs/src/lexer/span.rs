#[derive(Copy, Clone, Default, Debug)]
pub struct Pos {
	pub line: usize,
	pub column: usize,
	pub offset: usize,
	pub was_cr: bool,
}

#[derive(Copy, Clone, Debug)]
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