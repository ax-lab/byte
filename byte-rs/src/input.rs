/// Trait implemented by any input to a [super::Lexer].
pub trait Input {
	fn len(&self) -> usize;
	fn read(&self, pos: usize, end: usize) -> &[u8];

	fn read_text(&self, pos: usize, end: usize) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.read(pos, end)) }
	}
}
