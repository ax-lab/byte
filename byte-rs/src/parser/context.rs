use crate::core::error::*;
use crate::core::input::*;
use crate::lexer::*;
use crate::old::stream::Stream;

/// Full context for a file being parsed.
#[derive(Clone)]
pub struct Context {
	lexer: Lexer,
	stop: StopAt,
}

impl Context {
	pub fn new(lexer: Lexer) -> Self {
		Self {
			lexer,
			stop: StopAt::None,
		}
	}

	pub fn limit(&mut self, stop: StopAt) -> StopAt {
		self.stop.replace(stop)
	}

	pub fn release(&mut self, saved: StopAt) {
		self.stop.restore(saved);
	}

	pub fn pos(&self) -> Cursor {
		self.next().span().sta
	}

	pub fn next(&self) -> TokenAt {
		self.lookahead(0)
	}

	pub fn lookahead(&self, next: usize) -> TokenAt {
		for i in 0..=next {
			if !self.stop.can_continue(&self.lexer, i) {
				return self.lexer.next().as_none();
			}
		}
		self.lexer.lookahead(next)
	}

	pub fn read(&mut self) -> TokenAt {
		let next = self.next();
		if next.is_some() {
			self.lexer.read()
		} else {
			next
		}
	}

	pub fn has_errors(&self) -> bool {
		self.lexer.has_errors()
	}

	pub fn errors(&self) -> ErrorList {
		self.lexer.errors()
	}

	pub fn add_error(&mut self, error: Error) {
		self.lexer.add_error(error);
	}
}

#[derive(Clone)]
pub enum StopAt {
	None,
	Line,
}

impl Default for StopAt {
	fn default() -> Self {
		StopAt::None
	}
}

impl StopAt {
	pub fn can_continue(&self, next: &Lexer, offset: usize) -> bool {
		match self {
			StopAt::None => true,
			StopAt::Line => next.lookahead(offset).token() != Token::Break,
		}
	}

	pub fn replace(&mut self, other: StopAt) -> Self {
		std::mem::replace(self, other)
	}

	pub fn restore(&mut self, saved: StopAt) {
		*self = saved;
	}
}
