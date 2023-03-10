use std::collections::VecDeque;

use super::{Cursor, Input, Lex, LexerResult, Span, Token};

/// Wraps the input reader and its list of tokens.
pub struct LexSource<'a> {
	pub reader: Cursor<'a>,
	pub tokens: Vec<(Token, Span)>,
}

impl<'a> LexSource<'a> {
	pub fn new(input: &'a dyn Input) -> LexSource<'a> {
		let reader = Cursor::new(input);
		let tokens = read_all(reader.clone());
		LexSource { reader, tokens }
	}

	pub fn first(&self) -> Lex {
		Lex::from(self)
	}

	pub fn read_text(&self, span: Span) -> &str {
		self.reader
			.source
			.read_text(span.pos.offset, span.end.offset)
	}
}

/// Represents a valid token position in the source.
#[derive(Copy, Clone)]
pub struct LexPosition<'a> {
	index: usize,
	source: &'a LexSource<'a>,
}

impl<'a> LexPosition<'a> {
	pub fn from(source: &'a LexSource) -> Self {
		LexPosition { index: 0, source }
	}

	pub fn source(&self) -> &LexSource {
		self.source
	}

	pub fn span(&self) -> Span {
		self.source.tokens[self.index].1
	}

	pub fn token(&self) -> Token {
		self.source.tokens[self.index].0
	}

	pub fn pair(&self) -> (Token, Span) {
		self.source.tokens[self.index]
	}

	pub fn triple(&self) -> (Token, Span, &str) {
		let pair = self.pair();
		(pair.0, pair.1, self.text())
	}

	pub fn next(mut self) -> Lex<'a> {
		if self.index < self.source.tokens.len() - 1 {
			self.index += 1;
			Lex::Some(self)
		} else {
			Lex::None(self.source)
		}
	}

	pub fn text(&self) -> &str {
		let span = self.span();
		self.source
			.reader
			.source
			.read_text(span.pos.offset, span.end.offset)
	}
}

fn read_all(mut input: Cursor) -> Vec<(Token, Span)> {
	let mut tokens = Vec::new();
	let mut indent = VecDeque::new();
	let mut parens = VecDeque::new();

	let mut is_last = false;
	let mut is_empty = true;
	while !is_last {
		// check if we are at the start of the line so we can compute indentation
		let start = input.pos();
		let new_line = start.column == 0;
		is_empty = is_empty || new_line;

		// read the next token
		let (result, span) = super::read_token(&mut input);
		let (token, end, skip) = match result {
			LexerResult::Token(token) => (token, false, false),
			LexerResult::None => (Token::Break, true, false),
			LexerResult::Skip => (Token::Symbol(""), false, true),
			LexerResult::Error(error) => panic!("{error} at {span}"),
		};

		let (closing, closing_level) = if let Some(&(closing, level)) = parens.back() {
			let symbol = match token {
				Token::Symbol(symbol) => Some(symbol),
				Token::Identifier => Some(input.source.read_text(span.pos.offset, span.end.offset)),
				_ => None,
			};
			if Some(closing) == symbol {
				parens.pop_back();
				(true, level)
			} else {
				(false, 0)
			}
		} else {
			(false, 0)
		};

		let need_indent = (new_line && token != Token::Break) || end || closing;
		let column = if end {
			0
		} else if closing {
			closing_level
		} else {
			span.pos.column
		};

		// check if we need indent or dedent tokens by comparing the first token level
		if need_indent {
			let level = indent.back().copied().unwrap_or(0);
			if column > level {
				indent.push_back(column);
				tokens.push((
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
					tokens.push((
						Token::Dedent,
						Span {
							pos: start,
							end: span.pos,
						},
					));
				}
			}
		}

		if let Some(closing) = token.closing() {
			let level = indent.back().copied().unwrap_or(0);
			parens.push_back((closing, level));
		}

		is_last = end;
		let skip = skip
			|| match token {
				Token::Break => is_empty || end,
				_ => false,
			};

		if !skip {
			is_empty = false;
			tokens.push((token, span));
		}
	}
	tokens
}
