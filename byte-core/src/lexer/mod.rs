use crate::tokens::{Pos, Span, Token};

pub fn parse_string<T: AsRef<str>, U: AsRef<str>>(filename: T, _text: U) -> TokenIterator {
	let filename = filename.as_ref().to_string();
	TokenIterator {
		filename,
		at_end: false,
	}
}

pub struct TokenIterator {
	filename: String,
	at_end: bool,
}

impl Iterator for TokenIterator {
	type Item = Token;

	fn next(&mut self) -> Option<Self::Item> {
		if self.at_end {
			None
		} else {
			self.at_end = true;
			Some(Token::End(Span {
				filename: self.filename.clone(),
				start: Pos {
					line: 0,
					column: 0,
					offset: 0,
				},
				end: Pos {
					line: 0,
					column: 0,
					offset: 0,
				},
			}))
		}
	}
}
