use std::{
	collections::VecDeque,
	sync::{Arc, RwLock},
};

use super::*;

mod symbols;
mod token;

pub use token::*;

use symbols::SymbolTable;

/// Trait used by the [`Lexer`] to match tokens.
pub trait Matcher: Send + Sync {
	fn try_match(&self, next: char, input: &mut Cursor, errors: &mut Errors) -> Option<Node>;
}

#[derive(Clone)]
pub struct TokenStream {
	cursor: Cursor,
	lexer: Lexer,
	next: Arc<RwLock<Arc<VecDeque<(Node, Cursor, Errors)>>>>,
}

impl TokenStream {
	pub fn new(input: Cursor, lexer: Lexer) -> Self {
		Self {
			cursor: input,
			lexer,
			next: Default::default(),
		}
	}

	pub fn config<F: FnOnce(&mut Lexer)>(&mut self, config: F) {
		config(&mut self.lexer);
		self.flush_next();
	}

	pub fn lookahead(&self, n: usize) -> Node {
		{
			let next = self.next.read().unwrap();
			if let Some((node, ..)) = next.get(n) {
				return node.clone();
			} else if let Some((last, ..)) = next.back() {
				if last.is(Token::None) {
					return last.clone();
				}
			}
		}

		let mut next = self.next.write().unwrap();
		let next = Arc::make_mut(&mut next);
		let (mut cursor, mut errors) = next
			.back()
			.map(|(_, cursor, errors)| (cursor.clone(), errors.clone()))
			.unwrap_or_else(|| (self.cursor.clone(), Errors::default()));
		while n >= next.len() {
			let node = self.lexer.read(&mut cursor, &mut errors);
			let is_none = node.is(Token::None);
			next.push_back((node, cursor.clone(), errors.clone()));
			if is_none {
				break;
			}
		}
		next.back().map(|x| x.0.clone()).unwrap()
	}

	pub fn next(&self) -> Node {
		self.lookahead(0)
	}

	pub fn read(&mut self, errors: &mut Errors) -> Node {
		let mut next = self.next.write().unwrap();
		let next = Arc::make_mut(&mut next);
		if let Some((node, cursor, node_errors)) = next.pop_front() {
			if node_errors.len() > 0 {
				errors.append(node_errors);
			}
			self.cursor = cursor;
			return node;
		} else {
			self.lexer.read(&mut self.cursor, errors)
		}
	}

	pub fn skip(&mut self, count: usize, errors: &mut Errors) {
		for _ in 0..count {
			self.read(errors);
		}
	}

	fn flush_next(&mut self) {
		let mut next = self.next.write().unwrap();
		let next = Arc::make_mut(&mut next);
		next.clear();
	}
}

//====================================================================================================================//
// Lexer
//====================================================================================================================//

/// Implements raw token scanning using a configurable list of matchers
/// and a symbol table.
#[derive(Clone)]
pub struct Lexer {
	matchers: Arc<Vec<Arc<dyn Matcher>>>,
	symbols: Arc<SymbolTable>,
}

impl Lexer {
	pub fn new() -> Lexer {
		Lexer {
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

	fn read(&self, cursor: &mut Cursor, errors: &mut Errors) -> Node {
		self.skip(cursor);
		let start = cursor.clone();
		let node = self.read_next(cursor, errors);
		if node.span().is_none() {
			node.at(Some(Span::new(&start, cursor)))
		} else {
			node
		}
	}

	fn skip(&self, cursor: &mut Cursor) {
		let is_start = cursor.is_new_line();
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

	fn read_next(&self, input: &mut Cursor, errors: &mut Errors) -> Node {
		let start = input.clone();
		if let Some(next) = input.read() {
			if next == '\n' {
				return Node::from(Token::Break);
			}
			let saved = (input.clone(), errors.clone());
			for matcher in self.matchers.iter() {
				if let Some(token) = matcher.try_match(next, input, errors) {
					return token;
				} else {
					(*input, *errors) = saved.clone();
				}
			}

			// if none of the scanners matched, try the symbols
			if let Some(token) = self.symbols.try_match(next, input, errors) {
				token
			} else {
				errors.add("invalid symbol".at_span(Span::new(&start, input)));
				Node::from(Token::Invalid)
			}
		} else {
			Node::from(Token::None)
		}
	}
}
