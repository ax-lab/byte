use std::path::Path;

use crate::lexer::{Input, Pos, Span, TokenStream};

pub struct SourceFile {
	path: String,
	text: Vec<u8>,
	pos: Pos,
	prev: Pos,
}

pub fn open_file<P: AsRef<Path>>(path: P) -> std::io::Result<SourceFile> {
	let text = std::fs::read(path.as_ref())?;
	Ok(SourceFile {
		path: path.as_ref().to_string_lossy().to_string(),
		text,
		pos: Default::default(),
		prev: Default::default(),
	})
}

impl SourceFile {
	pub fn tokens(&mut self) -> TokenStream<SourceFile> {
		TokenStream::new(self)
	}
}

impl Input for SourceFile {
	fn read_text(&mut self, span: Span) -> &str {
		let (pos, end) = (span.pos, span.end);
		unsafe { std::str::from_utf8_unchecked(&self.text[pos.offset..end.offset]) }
	}

	fn pos(&self) -> Pos {
		self.pos
	}

	fn rewind(&mut self, pos: Pos) {
		self.pos = pos;
	}

	fn read(&mut self) -> Option<char> {
		self.prev = self.pos;

		let text = &self.text[self.pos.offset..];
		let text = unsafe { std::str::from_utf8_unchecked(text) };

		let mut chars = text.char_indices();
		if let Some((_, char)) = chars.next() {
			self.pos.offset = chars
				.next()
				.map(|x| self.pos.offset + x.0)
				.unwrap_or(self.text.len());
			match char {
				'\n' => {
					if !self.pos.was_cr {
						self.pos.line += 1;
						self.pos.column = 0;
					}
				}
				'\t' => {
					self.pos.column += 4 - (self.pos.column % 4);
				}
				_ => {
					self.pos.column += 1;
				}
			}
			self.pos.was_cr = char == '\r';
			Some(char)
		} else {
			None
		}
	}

	fn error(&self) -> Option<std::io::Error> {
		None
	}
}

impl std::fmt::Display for SourceFile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.path)
	}
}
