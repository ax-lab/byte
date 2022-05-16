#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Span {
	pub filename: String,
	pub start: Pos,
	pub end: Pos,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Pos {
	pub line: usize,
	pub column: usize,
	pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
	End(Span),
}
