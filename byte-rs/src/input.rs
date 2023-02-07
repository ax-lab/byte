use std::{fmt::Display, path::Path};

use crate::lexer::{read_token, Input, TokenKind};

#[derive(Debug)]
pub struct Token {
	pub kind: TokenKind,
	pub span: Span,
	pub text: String,
}

pub trait TokenStream {
	fn next(&mut self) -> Token;
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Pos {
	line: usize,
	column: usize,
	offset: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct Span {
	pub pos: Pos,
	pub end: Pos,
}

pub trait Source: Display + TokenStream {}

pub fn open_file<P: AsRef<Path>>(path: P) -> std::io::Result<SourceFile> {
	let text = std::fs::read(path.as_ref())?;
	Ok(SourceFile {
		path: path.as_ref().to_string_lossy().to_string(),
		text,
		pos: Default::default(),
		prev: Default::default(),
		was_cr: false,
		prev_cr: false,
	})
}

pub struct SourceFile {
	path: String,
	text: Vec<u8>,
	pos: Pos,
	prev: Pos,
	was_cr: bool,
	prev_cr: bool,
}

impl Source for SourceFile {}

impl TokenStream for SourceFile {
	fn next(&mut self) -> Token {
		loop {
			let pos = self.pos;
			let kind = if let Some(kind) = read_token(self) {
				kind
			} else {
				continue;
			};
			let end = self.pos;
			break Token {
				kind: kind,
				span: Span { pos, end },
				text: unsafe { std::str::from_utf8_unchecked(&self.text[pos.offset..end.offset]) }
					.into(),
			};
		}
	}
}

impl Input for SourceFile {
	fn read(&mut self) -> Option<char> {
		self.prev = self.pos;
		self.prev_cr = self.was_cr;

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
					if !self.was_cr {
						self.pos.line += 1;
						self.pos.column = 0;
					}
				}
				_ => {
					self.pos.column += 1;
				}
			}
			self.was_cr = char == '\r';
			Some(char)
		} else {
			None
		}
	}

	fn putback(&mut self) {
		self.pos = self.prev;
		self.was_cr = self.prev_cr;
	}

	fn error(&self) -> Option<std::io::Error> {
		None
	}
}

impl Display for SourceFile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.path)
	}
}

impl Display for Span {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.pos)
	}
}

impl Display for Pos {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}:{}", self.line + 1, self.column + 1)
	}
}
