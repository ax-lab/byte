use std::rc::Rc;

use super::{
	matcher::{MatchSymbol, SymbolTable},
	Indent, LexerError, LexerResult, Matcher, MatcherResult, Token,
};
use crate::{Cursor, Span};

#[derive(Default)]
pub struct Config {
	symbols: Rc<SymbolTable>,
	matchers: Vec<Box<dyn Matcher>>,
}

impl Clone for Config {
	fn clone(&self) -> Self {
		let mut matchers = Vec::with_capacity(self.matchers.len());
		for it in self.matchers.iter() {
			matchers.push(it.clone_box());
		}
		Self {
			symbols: self.symbols.clone(),
			matchers,
		}
	}
}

impl Config {
	pub fn add_symbol(&mut self, symbol: &'static str, value: Token) {
		let symbols = Rc::make_mut(&mut self.symbols);
		symbols.add_symbol(symbol, value);
	}

	pub fn add_matcher(&mut self, matcher: Box<dyn Matcher>) {
		self.matchers.push(matcher);
	}

	pub fn read_token(&self, input: &mut Cursor) -> (LexerResult, Span) {
		let mut pos = *input;
		let mut indent = Indent(pos.indent());
		let result = 'main: loop {
			if let Some(next) = input.read() {
				let symbol_matcher: Box<dyn Matcher> =
					Box::new(MatchSymbol::new(self.symbols.clone()));
				let matchers = self.matchers.iter().chain(std::iter::once(&symbol_matcher));

				let start = *input;
				let mut skipped = false;
				for it in matchers {
					*input = start;
					match it.try_match(next, input) {
						MatcherResult::None => continue,
						MatcherResult::Error(error) => break 'main LexerResult::Error(error),
						next @ (MatcherResult::Skip | MatcherResult::Comment) => {
							pos = *input;
							if let MatcherResult::Skip = next {
								indent = Indent(pos.indent());
							}
							skipped = true;
							break;
						}
						MatcherResult::Token(token) => {
							break 'main LexerResult::Token(token, indent);
						}
					}
				}
				if !skipped {
					break LexerResult::Error(LexerError::InvalidToken);
				}
			} else {
				break LexerResult::None;
			}
		};

		(
			result,
			Span {
				sta: pos,
				end: *input,
			},
		)
	}
}
