use std::rc::Rc;

use crate::core::error::*;
use crate::core::input::*;

use super::*;

/// Low-level stream of raw [`Token`] from an input.
///
/// This is a thin wrapper over a [`Scanner`] and input [`Cursor`]
/// providing access to a stream of tokens from that position.
///
/// ## Note for future implementation
///
/// This low-level [`TokenStream`] is the perfect position to inject
/// custom tokenization, such as token generators, and processing lexing
/// pragmas from the input.
///
/// It sits at a low enough level that it's not encumbered with higher level
/// lexer semantics such as indentation. It also has access to both parsing
/// and generating skipped tokens such as [`Comment`].
///
/// On the other hand, it sits at a high enough level to not need to handle
/// raw text parsing.
#[derive(Clone)]
pub struct TokenStream {
	scanner: Rc<Scanner>,

	/// Current input position.
	input: Cursor,

	/// Current list of errors.
	errors: ErrorList,
}

/// Trait for any type providing a stream of [`Token`].
pub trait Stream {
	fn pos(&self) -> Cursor;

	fn copy(&self) -> Box<dyn Stream>;

	fn next(&self) -> TokenAt;

	fn read(&mut self) -> TokenAt;

	fn errors(&self) -> ErrorList;

	fn add_error(&mut self, error: Error);

	fn advance(&mut self) {
		self.read();
	}

	fn has_errors(&self) -> bool {
		!self.errors().empty()
	}

	fn list_errors(&self) -> Vec<Error> {
		self.errors().list()
	}

	fn token(&self) -> Token {
		self.next().token()
	}

	fn span(&self) -> Span {
		self.next().span()
	}

	fn peek_after(&self) -> TokenAt {
		let mut input = self.copy();
		input.advance();
		input.next()
	}

	//----[ Reader helpers ]--------------------------------------------------//

	fn at_end(&self) -> bool {
		self.token() == Token::None
	}

	fn has_some(&self) -> bool {
		!self.at_end()
	}

	fn from(&self, pos: Cursor) -> Span {
		Span {
			sta: pos,
			end: self.pos(),
		}
	}

	/// Return the next token and true if the predicate matches the current
	/// token.
	fn next_if(&mut self, predicate: &dyn Fn(TokenAt) -> bool) -> bool {
		if predicate(self.next()) {
			self.advance();
			true
		} else {
			false
		}
	}

	/// Read the next token if it is the specific symbol.
	fn skip_symbol(&mut self, symbol: &str) -> bool {
		self.next_if(&|value| value.symbol() == Some(symbol))
	}

	fn check_end(&mut self) -> bool {
		if self.has_some() {
			let next = self.next();
			self.add_error(Error::new(next.span(), LexerError::ExpectedEnd(next)));
			false
		} else {
			true
		}
	}
}

impl TokenStream {
	pub fn new(input: Cursor, scanner: Scanner) -> Self {
		TokenStream {
			errors: ErrorList::new(),
			input,
			scanner: Rc::new(scanner),
		}
	}

	pub fn pos(&self) -> Cursor {
		self.input.clone()
	}

	pub fn errors(&self) -> &ErrorList {
		&self.errors
	}

	pub fn errors_mut(&mut self) -> &mut ErrorList {
		&mut self.errors
	}

	pub fn config<F: FnOnce(&mut Scanner)>(&mut self, config: F) {
		let scanner = Rc::make_mut(&mut self.scanner);
		config(scanner)
	}

	pub fn skip(&mut self) {
		self.scanner.skip(&mut self.input);
	}

	pub fn read(&mut self) -> Token {
		self.scanner.read(&mut self.input, &mut self.errors)
	}
}
