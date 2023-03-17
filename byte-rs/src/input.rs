/// Trait implemented by any input to a [super::Lexer].
pub trait Input {
	fn len(&self) -> usize;
	fn read(&self, pos: usize, end: usize) -> &[u8];
	fn name(&self) -> &str;

	fn read_text(&self, pos: usize, end: usize) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.read(pos, end)) }
	}
}

impl Input for &str {
	fn len(&self) -> usize {
		str::len(self)
	}

	fn name(&self) -> &str {
		"eval"
	}

	fn read(&self, pos: usize, end: usize) -> &[u8] {
		&self.as_bytes()[pos..end]
	}
}
