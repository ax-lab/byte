use std::collections::VecDeque;

use super::{Input, Reader, Span, State};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(unused)]
pub enum Token {
	None,
	Invalid,
	Comment,
	LineBreak,
	Ident,
	Dedent,
	Identifier,
	Integer,
	String,
	Symbol,
	Comma,
}

pub struct TokenStream<'a, T: Input> {
	input: &'a mut Reader<T>,
	next: VecDeque<(Token, Span)>,
	ident: VecDeque<usize>,
	state: State,
}

#[allow(unused)]
impl<'a, T: Input> TokenStream<'a, T> {
	pub fn new(input: &'a mut Reader<T>) -> TokenStream<'a, T> {
		let mut state = State::default();
		state.symbols.add_symbol("+");
		state.symbols.add_symbol("-");
		state.symbols.add_symbol("*");
		state.symbols.add_symbol("/");
		state.symbols.add_symbol("%");
		state.symbols.add_symbol("=");
		state.symbols.add_symbol("==");
		state.symbols.add_symbol("!");
		state.symbols.add_symbol("?");
		state.symbols.add_symbol(":");
		state.symbols.add_symbol("(");
		state.symbols.add_symbol(")");
		state.symbols.add_symbol(".");
		state.symbols.add_symbol("..");
		TokenStream {
			input,
			next: Default::default(),
			ident: Default::default(),
			state,
		}
	}

	/// Returns the next available token in this stream or None at the end
	/// of the stream.
	///
	/// This does not consume the token. Multiple calls to this method will
	/// return the same token until [`TokenStream::shift`] is called.
	pub fn get(&mut self) -> Token {
		self.read().0
	}

	pub fn span(&mut self) -> Span {
		self.read().1
	}

	pub fn text(&mut self) -> &str {
		let span = self.span();
		self.input.read_text(span)
	}

	/// Returns a token to the stream, making it available for reading again.
	pub fn unget(&mut self, token: Token) {
		todo!()
	}

	/// Consumes the next token in the stream.
	pub fn shift(&mut self) {
		self.next.pop_front();
	}

	/// Returns a child `TokenStream` that will iterate tokens from the
	/// current one, but will stop at a [`Token::Ident`].
	pub fn indented(&mut self) -> TokenStream<T> {
		todo!()
	}

	/// Returns a child `TokenStream` that will iterate tokens from the
	/// current one, but will stop at tokens for which `f` returns true.
	pub fn until(&mut self, f: fn(Token) -> bool) -> TokenStream<T> {
		todo!()
	}

	/// Returns a child `TokenStream` that will iterate tokens from the
	/// current one, but will stop at the given right parenthesis.
	///
	/// Note that this overrides an [`TokenStream::until`] limitation.
	pub fn parenthesized(&mut self, right: Token) -> TokenStream<T> {
		todo!()
	}

	fn read(&mut self) -> (Token, Span) {
		while self.next.is_empty() {
			// check if we are at the start of the line so we can compute identation
			let start = self.input.pos();
			let new_line = start.column == 0;

			// read the next token
			let (token, pos) = loop {
				let pos = self.input.pos();
				let (token, ok) = super::read_token(self.input, &mut self.state);
				if token == Token::Invalid {
					let span = Span {
						pos,
						end: self.input.pos(),
					};
					panic!(
						"invalid token at {} (`{}`)",
						span,
						self.input.read_text(span)
					);
				} else if token != Token::None {
					break (token, pos);
				} else if !ok {
					break (Token::None, pos);
				}
			};
			let end = self.input.pos();
			if token != Token::Comment {
				self.next.push_back((token, Span { pos, end }));
			}

			// check if we need indent or dedent tokens by comparing the first token level
			if (new_line && token != Token::LineBreak) || token == Token::None {
				let column = if token == Token::None { 0 } else { pos.column };
				let ident = self.ident.back().copied().unwrap_or(0);
				if column > ident {
					self.ident.push_back(column);
					self.next.push_front((
						Token::Ident,
						Span {
							pos: start,
							end: pos,
						},
					));
				} else {
					let mut ident = ident;
					while column < ident {
						self.ident.pop_back();
						ident = self.ident.back().copied().unwrap_or(0);
						self.next.push_front((
							Token::Dedent,
							Span {
								pos: start,
								end: pos,
							},
						))
					}
				}
			}
		}

		self.next.front().copied().unwrap()
	}
}
