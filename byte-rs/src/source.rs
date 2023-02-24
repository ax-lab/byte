use std::path::Path;

use crate::lexer::Input;

pub struct SourceFile {
	path: String,
	text: Vec<u8>,
	offset: usize,
}

pub fn open_file<P: AsRef<Path>>(path: P) -> std::io::Result<SourceFile> {
	let text = std::fs::read(path.as_ref())?;
	Ok(SourceFile {
		path: path.as_ref().to_string_lossy().to_string(),
		text,
		offset: 0,
	})
}

impl Input for SourceFile {
	type Error = std::io::Error;

	fn read_text(&mut self, pos: usize, end: usize) -> &str {
		unsafe { std::str::from_utf8_unchecked(&self.text[pos..end]) }
	}

	fn offset(&self) -> usize {
		self.offset
	}

	fn set_offset(&mut self, pos: usize) {
		self.offset = pos;
	}

	fn read(&mut self) -> Option<char> {
		let text = &self.text[self.offset..];
		let text = unsafe { std::str::from_utf8_unchecked(text) };
		let mut chars = text.char_indices();

		if let Some((_, char)) = chars.next() {
			self.offset = chars
				.next()
				.map(|x| self.offset + x.0)
				.unwrap_or(self.text.len());
			Some(char)
		} else {
			None
		}
	}
}

impl std::fmt::Display for SourceFile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.path)
	}
}
