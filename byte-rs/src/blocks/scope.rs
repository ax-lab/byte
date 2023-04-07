use crate::lexer::*;

pub trait Scope: Clone {
	fn read(&mut self) -> TokenAt;

	fn peek(&self) -> TokenAt {
		let mut clone = self.clone();
		clone.read()
	}
}
