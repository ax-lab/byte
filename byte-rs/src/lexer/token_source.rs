use std::{
	cell::{Ref, RefCell},
	collections::VecDeque,
};

use super::{Input, LexerResult, Reader, Span, Token};

pub trait TokenSource {
	fn peek(&self) -> Ref<(Token, Span)>;
	fn read(&mut self) -> (Token, Span);
	fn unget(&mut self, token: Token, span: Span);
	fn indent_level(&self) -> usize;
	fn pop_indent(&mut self, level: usize);
}

pub struct ReaderTokenSource {
	input: RefCell<Reader>,
	next: RefCell<VecDeque<(Token, Span)>>,
	indent: RefCell<VecDeque<usize>>,
	last_span: Option<Span>,
}

impl<T: Input + 'static> From<T> for ReaderTokenSource {
	fn from(input: T) -> Self {
		ReaderTokenSource {
			input: Reader::from(input).into(),
			next: Default::default(),
			indent: Default::default(),
			last_span: None,
		}
	}
}

impl TokenSource for ReaderTokenSource {
	fn peek(&self) -> Ref<(Token, Span)> {
		self.fill_next();
		let next = self.next.borrow();
		Ref::map(next, |x| x.front().unwrap())
	}

	fn read(&mut self) -> (Token, Span) {
		self.fill_next();
		let (token, span) = self.next.borrow_mut().pop_front().unwrap();
		self.last_span = Some(span);
		(token, span)
	}

	fn unget(&mut self, token: Token, span: Span) {
		assert_eq!(
			Some(span),
			self.last_span,
			"unget should only be used for the last token"
		);
		self.last_span = None;
		self.next.borrow_mut().push_front((token, span));
	}

	fn indent_level(&self) -> usize {
		self.indent.borrow().len()
	}

	fn pop_indent(&mut self, level: usize) {
		let span = self.peek().1;
		let mut next = self.next.borrow_mut();
		let mut indent = self.indent.borrow_mut();
		assert!(indent.len() >= level);
		while indent.len() > level {
			indent.pop_back().unwrap();
			next.push_front((
				Token::Dedent,
				Span {
					pos: span.pos,
					end: span.pos,
				},
			));
		}
	}
}

impl ReaderTokenSource {
	pub fn read_text(&self, pos: usize, end: usize) -> String {
		self.input.borrow().read_text(pos, end)
	}

	fn fill_next(&self) {
		let mut next = self.next.borrow_mut();
		let mut indent = self.indent.borrow_mut();
		let mut input = self.input.borrow_mut();
		while next.is_empty() {
			// check if we are at the start of the line so we can compute indentation
			let start = input.pos();
			let new_line = start.column == 0;

			// read the next token
			let (result, span) = super::read_token(&mut input);
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
				next.push_back((token, span));
			}

			// check if we need indent or dedent tokens by comparing the first token level
			if need_indent {
				let level = indent.back().copied().unwrap_or(0);
				if column > level {
					indent.push_back(column);
					next.push_front((
						Token::Indent,
						Span {
							pos: start,
							end: span.pos,
						},
					));
				} else {
					let mut level = level;
					while column < level {
						indent.pop_back();
						level = indent.back().copied().unwrap_or(0);
						next.push_front((
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
