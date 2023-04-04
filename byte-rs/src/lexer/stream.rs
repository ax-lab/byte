use crate::core::error::*;
use crate::core::input::*;

use super::*;

pub trait Stream {
	fn pos(&self) -> Cursor;

	fn copy(&self) -> Box<dyn Stream>;

	fn next(&self) -> TokenAt;

	fn read(&mut self) -> TokenAt;

	fn errors_ref(&self) -> &ErrorList;

	fn errors_mut(&mut self) -> &mut ErrorList;

	fn advance(&mut self) {
		self.read();
	}

	fn add_at(&mut self, span: Span, info: Rc<dyn ErrorInfo>) {
		self.errors_mut().add_at(span, info);
	}

	fn has_errors(&self) -> bool {
		!self.errors_ref().empty()
	}

	fn errors(&self) -> Vec<Error> {
		self.errors_ref().list()
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
			self.add_at(next.span(), LexerError::ExpectedEnd(next).upcast());
			false
		} else {
			true
		}
	}
}
