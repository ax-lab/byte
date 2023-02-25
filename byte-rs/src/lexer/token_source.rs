use std::collections::VecDeque;

use super::{Input, LexerResult, Reader, Span, Token};

pub trait TokenSource {
	fn peek(&mut self) -> &(Token, Span);
	fn read(&mut self) -> (Token, Span);
	fn unget(&mut self, token: Token, span: Span);
	fn read_text(&self, span: Span) -> &str;
}

pub struct ReaderTokenSource<T: Input> {
	input: Reader<T>,
	next: VecDeque<(Token, Span)>,
	indent: VecDeque<usize>,
}

impl<T: Input> From<T> for ReaderTokenSource<T> {
	fn from(input: T) -> Self {
		ReaderTokenSource {
			input: Reader::from(input),
			next: Default::default(),
			indent: Default::default(),
		}
	}
}

impl<T: Input> TokenSource for ReaderTokenSource<T> {
	fn peek(&mut self) -> &(Token, Span) {
		self.fill_next();
		self.next.front().unwrap()
	}

	fn read(&mut self) -> (Token, Span) {
		self.fill_next();
		self.next.pop_front().unwrap()
	}

	fn unget(&mut self, token: Token, span: Span) {
		let next = self.peek().1;
		assert!(span.end.offset <= next.pos.offset);
		self.next.push_front((token, span));
	}

	fn read_text(&self, span: Span) -> &str {
		self.input.read_text(span)
	}
}

impl<T: Input> ReaderTokenSource<T> {
	pub fn inner(&self) -> &T {
		self.input.inner()
	}

	#[allow(unused)]
	pub fn inner_mut(&mut self) -> &mut T {
		self.input.inner_mut()
	}

	fn fill_next(&mut self) {
		while self.next.is_empty() {
			// check if we are at the start of the line so we can compute indentation
			let start = self.input.pos();
			let new_line = start.column == 0;

			// read the next token
			let (result, span) = super::read_token(&mut self.input);
			let token = match result {
				LexerResult::Token(token) => token,
				LexerResult::None => Token::None,
				LexerResult::Error(error) => panic!("{error} at {span}"),
			};

			let need_indent = (new_line && token != Token::LineBreak) || token == Token::None;
			let column = if token == Token::None {
				0
			} else {
				span.pos.column
			};
			if token != Token::Comment {
				self.next.push_back((token, span));
			}

			// check if we need indent or dedent tokens by comparing the first token level
			if need_indent {
				let ident = self.indent.back().copied().unwrap_or(0);
				if column > ident {
					self.indent.push_back(column);
					self.next.push_front((
						Token::Ident,
						Span {
							pos: start,
							end: span.pos,
						},
					));
				} else {
					let mut ident = ident;
					while column < ident {
						self.indent.pop_back();
						ident = self.indent.back().copied().unwrap_or(0);
						self.next.push_front((
							Token::Dedent,
							Span {
								pos: start,
								end: span.pos,
							},
						))
					}
				}
			}
		}
	}
}
