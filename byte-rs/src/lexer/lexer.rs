use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::RwLock;

use crate::core::error::*;
use crate::core::input::*;

use super::*;

/// [`Lexer`] holds the entire lexing state and configuration for a position
/// in the input, given by a [`Cursor`].
///
/// The lexer output is a stream of [`Token`] ready to be parsed.
///
/// Cloning a [`Lexer`] is a low-overhead operation and allows saving an
/// input position and configuration state, allowing a parser to backtrack
/// fully.
///
/// It is the lexer's responsibility to apply high-level language tokenization
/// semantics such as indentation (i.e. [`Token::Indent`] and [`Token::Dedent`])
/// and filtering of ignored tokens such as [`Comment`].
///
/// The lexer can be reconfigured during parsing, either from the input source
/// text or by the parser itself.
pub struct Lexer {
	state: State,
	next: RwLock<Arc<VecDeque<(TokenAt, State)>>>,
}

impl Clone for Lexer {
	fn clone(&self) -> Self {
		let next = self.next.read().unwrap().clone();
		Self {
			state: self.state.clone(),
			next: RwLock::new(next),
		}
	}
}

impl Default for Lexer {
	fn default() -> Self {
		let input = Input::open_str("empty", "");
		Lexer::new(input.start(), Scanner::new())
	}
}

impl Lexer {
	pub fn new(input: Cursor, scanner: Scanner) -> Self {
		Lexer {
			state: State {
				stream: TokenStream::new(input, scanner),
				indent: Indent::new(),
				unread: VecDeque::new(),
			},
			next: RwLock::new(Arc::new(VecDeque::new())),
		}
	}

	pub fn is_parenthesis(&self, token: &TokenAt) -> Option<&'static str> {
		self.state.is_parenthesis(token)
	}

	pub fn pop_indent_levels(&mut self, levels: usize) {
		self.state.indent.pop_levels(levels);
	}

	pub fn config<F: FnOnce(&mut Scanner)>(&mut self, config: F) {
		let mut next = self.next.write().unwrap();
		*next = Arc::new(VecDeque::new());
		self.state.stream.config(config);
	}

	pub fn errors(&self) -> ErrorList {
		self.state.stream.errors().clone()
	}

	pub fn add_error(&mut self, error: Error) {
		self.state.stream.errors_mut().add(error)
	}

	pub fn pos(&self) -> Cursor {
		self.state.stream.pos().clone()
	}

	pub fn lookahead(&self, n: usize) -> TokenAt {
		{
			let next = self.next.read().unwrap();
			if let Some((token, ..)) = next.get(n) {
				return token.clone();
			} else if let Some((last, ..)) = next.back() {
				if last.is_none() {
					return last.clone();
				}
			}
		}

		let mut next = self.next.write().unwrap();
		let next = Arc::make_mut(&mut next);
		let mut state = next
			.back()
			.map(|x| x.1.clone())
			.unwrap_or_else(|| self.state.clone());
		while n >= next.len() {
			let token = state.read();
			let is_none = token.is_none();
			next.push_back((token.clone(), state.clone()));
			if is_none {
				break;
			}
		}
		next.back().map(|x| x.0.clone()).unwrap()
	}

	pub fn next(&self) -> TokenAt {
		self.lookahead(0)
	}

	pub fn read(&mut self) -> TokenAt {
		let mut next = self.next.write().unwrap();
		let next = Arc::make_mut(&mut next);
		if let Some((token, state)) = next.pop_front() {
			self.state = state;
			return token;
		} else {
			self.state.read()
		}
	}

	pub fn skip(&mut self, count: usize) {
		for _ in 0..count {
			self.read();
		}
	}
}

#[derive(Clone)]
struct State {
	stream: TokenStream,
	indent: Indent,
	unread: VecDeque<TokenAt>,
}

impl State {
	fn is_parenthesis(&self, token: &TokenAt) -> Option<&'static str> {
		match token.symbol() {
			Some("(") => Some(")"),
			Some("[") => Some("]"),
			Some("{") => Some("}"),
			_ => None,
		}
	}

	fn read(&mut self) -> TokenAt {
		use crate::lang::Comment;

		let empty = self.stream.pos().col() == 0;
		loop {
			self.stream.skip();

			// the start position for the next token
			let start = self
				.unread
				.front()
				.map(|x| x.span().sta.clone())
				.unwrap_or_else(|| self.stream.pos().clone());

			// check for indentation tokens
			let errors = self.stream.errors_mut();
			let next = if let Some(next) = self.indent.check_indent(&start, errors) {
				next
			} else {
				self.unread
					.pop_front()
					.unwrap_or_else(|| self.stream.read())
			};

			// parenthesized regions group indentation
			if let Some(close) = self.is_parenthesis(&next) {
				self.indent.open_region(IndentRegion::UntilSymbol(close));
			} else if self.indent.check_for_closed_regions(&next) {
				// unread the physical token and run the indent logic again to
				// account for the closed region
				self.unread.push_front(next);
				continue;
			}

			let token = next.token();
			if token.is::<Comment>() || (empty && token == Token::Break) {
				continue;
			}
			break next;
		}
	}
}
