use super::Cursor;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Range<'a> {
	pub pos: Cursor<'a>,
	pub end: Cursor<'a>,
}

impl<'a> Range<'a> {
	pub fn text(&self) -> &str {
		self.pos.source.read_text(self.pos.offset, self.end.offset)
	}
}

impl<'a> std::fmt::Display for Range<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.pos)
	}
}
