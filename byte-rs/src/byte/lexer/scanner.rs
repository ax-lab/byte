use std::sync::Arc;

use super::symbols::*;
use super::*;

/// Provide the low-level lexical scanning for the language.
pub trait Matcher: Send + Sync {
	/// Read the [`Input`] stream and return a [`Node`] when matched.
	///
	/// Any changes to the [`Cursor`] or [`Errors`] are ignored if this
	/// returns [`None`].
	fn try_match(&self, cursor: &mut Cursor, errors: &mut Errors) -> Option<Node>;
}

/// Implements raw token scanning using a configurable list of [`Matcher`]
/// and a symbol table.
#[derive(Clone)]
pub struct Scanner {
	matchers: Arc<Vec<Arc<dyn Matcher>>>,
	symbols: Arc<SymbolTable>,
}

impl Scanner {
	pub fn new() -> Scanner {
		Scanner {
			matchers: Arc::new(Vec::new()),
			symbols: Arc::new(SymbolTable::default()),
		}
	}

	pub fn add_matcher<T: Matcher + 'static>(&mut self, matcher: T) {
		let matchers = Arc::make_mut(&mut self.matchers);
		matchers.push(Arc::new(matcher));
	}

	pub fn add_symbol<T: IsNode>(&mut self, symbol: &'static str, value: T) {
		let symbols = Arc::make_mut(&mut self.symbols);
		symbols.add_symbol(symbol, value);
	}

	pub fn read(&self, cursor: &mut Cursor, errors: &mut Errors) -> Option<Node> {
		self.skip(cursor);
		let start = cursor.clone();
		let node = self.read_next(cursor, errors);
		node.map(|node| {
			if node.span().is_none() {
				node.at(Some(Span::from(&start, cursor)))
			} else {
				node
			}
		})
	}

	fn skip(&self, cursor: &mut Cursor) {
		let is_start = cursor.location().is_line_start();
		let mut saved = cursor.clone();
		while let Some(next) = cursor.read() {
			if is_space(next) || (is_start && next == '\n') {
				saved = cursor.clone();
			} else {
				break;
			}
		}
		*cursor = saved;
	}

	fn read_next(&self, input: &mut Cursor, errors: &mut Errors) -> Option<Node> {
		let start = input.clone();
		if let Some(next) = input.peek() {
			if next == '\n' {
				input.read();
				return Some(Node::from(Token::Break));
			}
			let saved = (input.clone(), errors.clone());
			for matcher in self.matchers.iter() {
				if let Some(token) = matcher.try_match(input, errors) {
					return Some(token);
				} else {
					(*input, *errors) = saved.clone();
				}
			}

			// if none of the scanners matched, try the symbols
			if let Some(token) = self.symbols.try_match(input, errors) {
				Some(token)
			} else {
				errors.add("invalid symbol".at(Span::from(&start, input)));
				Some(Node::from(Token::Invalid))
			}
		} else {
			None
		}
	}
}
