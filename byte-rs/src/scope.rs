use std::{
	cell::{Cell, Ref, RefCell, RefMut},
	collections::VecDeque,
};

use crate::{
	lexer::{Lex, LexStream, Token},
	Error, Input,
};

/// Scope actions at a given input position.
#[derive(Copy, Clone)]
#[allow(unused)]
pub enum Action {
	/// Output the current token.
	Output,
	/// Skip the current token.
	Skip,
	/// End the current scope, not including the current token.
	Stop,
	/// Skip the current token and end the current scope.
	SkipAndStop,
}

pub trait Scope {
	fn copy(&self) -> Box<dyn Scope>;

	/// Apply the scope to the current input stream position.
	///
	/// This is called when the scope is first applied to an input stream and
	/// each time the input advances to the next token.
	///
	/// Returns the action relative to the current input token.
	fn apply(&mut self, input: &dyn LexStream, as_parent: bool) -> Action;

	fn leave(&self, _input: &dyn LexStream) -> Option<Error> {
		None
	}
}

pub enum Scoped {
	Root,
	Child {
		mode: ChildMode,
		last: Option<(Lex, Action)>,
		scope: Box<dyn Scope>,
		parent: Option<Box<Scoped>>,
	},
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ChildMode {
	/// The child operates within the parent scope, deferring to it before
	/// applying its own rules.
	Secondary,
	/// The child overrides the parent, operating directly with the root input
	/// regardless of the parent rules.
	Override,
}

impl ChildMode {
	fn is_override(&self) -> bool {
		matches!(self, ChildMode::Override)
	}
}

impl Clone for Scoped {
	fn clone(&self) -> Self {
		match self {
			Self::Root => Self::Root,
			Self::Child {
				mode,
				last,
				scope,
				parent,
			} => Self::Child {
				mode: mode.clone(),
				last: last.clone(),
				scope: scope.copy(),
				parent: parent.clone(),
			},
		}
	}
}

impl Default for Scoped {
	fn default() -> Self {
		Scoped::Root
	}
}

impl Scoped {
	pub fn apply(&mut self, input: &dyn LexStream, is_parent: bool) -> Action {
		match self {
			Scoped::Root => Action::Output,
			Scoped::Child {
				mode,
				last,
				scope,
				parent,
			} => {
				// avoid applying the same position twice to the scope
				if let Some((lex, result)) = last {
					let next = input.next();
					if lex.span == next.span && lex.token == next.token {
						return *result;
					}
				}

				if !mode.is_override() {
					if let Some(parent) = parent.as_mut() {
						match parent.apply(input, true) {
							Action::Output => {}
							Action::Skip => return Action::Skip,
							action @ Action::Stop | action @ Action::SkipAndStop => return action,
						}
					}
				}

				scope.apply(input, is_parent)
			}
		}
	}

	pub fn enter(&mut self, scope: Box<dyn Scope>, mode: ChildMode) {
		let me = std::mem::take(self);
		*self = Scoped::Child {
			mode,
			last: None,
			scope: scope,
			parent: Some(Box::new(me)),
		};
	}

	pub fn leave(&mut self, input: &dyn LexStream) -> Option<Error> {
		match self {
			Scoped::Root => panic!("trying to leave root scope"),
			Scoped::Child { scope, parent, .. } => {
				if let Some(parent) = parent {
					let parent = std::mem::take(&mut *parent);
					let result = scope.leave(input);
					*self = *parent;
					result
				} else {
					panic!("trying to leave scope without a parent");
				}
			}
		}
	}
}

pub struct ScopedStream {
	input: RefCell<Box<dyn LexStream + 'static>>,
	scope: RefCell<Scoped>,
	next: Cell<Option<Lex>>,
	done: Cell<bool>,
}

impl ScopedStream {
	pub fn new<T: LexStream + 'static>(input: T) -> ScopedStream {
		ScopedStream {
			input: RefCell::new(Box::new(input)),
			scope: RefCell::new(Scoped::Root),
			next: Cell::new(None),
			done: Cell::new(false),
		}
	}

	pub fn enter(&mut self, scope: Box<dyn Scope>, mode: ChildMode) {
		self.scope.borrow_mut().enter(scope, mode);
		self.apply_scope();
		self.next.set(None);
	}

	pub fn leave(&mut self) {
		let result = {
			let mut scope = self.scope.borrow_mut();
			let input = self.input();
			scope.leave(&*input)
		};
		if let Some(error) = result {
			self.add_error(error);
		}
		self.next.set(None);
		self.done.set(false);
	}

	pub fn input(&self) -> Ref<dyn LexStream> {
		let input = self.input.borrow();
		Ref::map(input, |x| &**x)
	}

	pub fn input_mut(&self) -> RefMut<dyn LexStream> {
		let input = self.input.borrow_mut();
		RefMut::map(input, |x| &mut **x)
	}

	fn apply_scope(&self) {
		if self.done.get() {
			return;
		}

		let stopped = {
			let mut input = self.input_mut();
			let mut scope = self.scope.borrow_mut();
			let mut stopped = false;
			while !stopped {
				match scope.apply(&*input, false) {
					Action::Output => {
						break;
					}
					Action::Skip => {
						input.advance();
					}
					Action::Stop => {
						stopped = true;
					}
					Action::SkipAndStop => {
						input.advance();
						stopped = true;
					}
				}
			}
			stopped
		};
		self.done.set(stopped);
	}
}

impl Clone for ScopedStream {
	fn clone(&self) -> Self {
		Self {
			input: RefCell::new(self.input().copy()),
			scope: RefCell::new(self.scope.borrow().clone()),
			next: self.next.clone(),
			done: self.done.clone(),
		}
	}
}

impl LexStream for ScopedStream {
	fn copy(&self) -> Box<dyn LexStream> {
		Box::new(self.clone())
	}

	fn source(&self) -> Input {
		self.input().source()
	}

	fn next(&self) -> Lex {
		if let Some(next) = self.next.get() {
			next
		} else {
			self.apply_scope();
			let input = self.input();
			let next = if self.done.get() {
				input.next().as_none()
			} else {
				input.next()
			};
			self.next.set(Some(next));
			next
		}
	}

	fn advance(&mut self) {
		if self.done.get() {
			return;
		}
		let mut input = self.input_mut();
		input.advance();
		self.next.set(None);
	}

	fn errors(&self) -> Vec<Error> {
		self.input().errors()
	}

	fn add_error(&mut self, error: Error) {
		self.input_mut().add_error(error)
	}

	fn has_errors(&self) -> bool {
		self.input().has_errors()
	}
}

pub struct ScopeLine {
	ended: bool,
	level: usize,
	split: Option<&'static str>,
}

impl ScopeLine {
	pub fn new() -> Box<dyn Scope> {
		Box::new(ScopeLine {
			ended: false,
			level: 0,
			split: None,
		})
	}

	pub fn new_with_break(split: &'static str) -> Box<dyn Scope> {
		Box::new(ScopeLine {
			ended: false,
			level: 0,
			split: Some(split),
		})
	}
}

impl Scope for ScopeLine {
	fn copy(&self) -> Box<dyn Scope> {
		Box::new(ScopeLine {
			ended: self.ended,
			level: self.level,
			split: self.split,
		})
	}

	fn apply(&mut self, input: &dyn LexStream, as_parent: bool) -> Action {
		if self.ended {
			return Action::Stop;
		}

		match input.token() {
			Token::Indent => {
				self.level += 1;
				Action::Output
			}
			Token::Dedent => {
				let level = self.level - 1;
				if as_parent && level == 0 {
					return Action::Stop;
				}
				self.level = level;
				if level == 0 {
					self.ended = true;
				}
				Action::Output
			}
			Token::Break => {
				if self.level == 0 {
					let mut input = input.copy();
					input.advance();
					if input.token() == Token::Indent {
						Action::Output
					} else {
						Action::Stop
					}
				} else {
					Action::Output
				}
			}
			_ => {
				if let Some(_) = self.split {
					if self.split == input.next().symbol() {
						return Action::Stop;
					}
				}
				Action::Output
			}
		}
	}
}

pub struct ScopeParenthesized {
	open: VecDeque<Lex>,
}

impl ScopeParenthesized {
	pub fn new() -> Box<dyn Scope> {
		let scope = ScopeParenthesized {
			open: Default::default(),
		};
		Box::new(scope)
	}
}

impl Scope for ScopeParenthesized {
	fn copy(&self) -> Box<dyn Scope> {
		Box::new(ScopeParenthesized {
			open: self.open.clone(),
		})
	}

	fn apply(&mut self, input: &dyn LexStream, as_parent: bool) -> Action {
		if let Some(open) = self.open.front() {
			if input.next().symbol() == open.token.get_closing() {
				if self.open.len() == 1 && as_parent {
					return Action::Stop;
				}

				self.open.pop_front();
				if self.open.len() == 0 {
					Action::Stop
				} else {
					Action::Output
				}
			} else {
				if let Some(_) = input.token().get_closing() {
					self.open.push_front(input.next());
				}
				Action::Output
			}
		} else {
			let next = input.next();
			assert!(
				next.token.get_closing().is_some(),
				"scope does not start at a valid parenthesized symbol (got {next} at {})",
				next.span
			);
			self.open.push_front(next);
			Action::Skip
		}
	}

	fn leave(&self, input: &dyn LexStream) -> Option<Error> {
		if let Some(open) = self.open.front() {
			let next = input.next();
			let end = open.token.get_closing().unwrap();
			Some(
				Error::ExpectedSymbol(end, next.span)
					.at(format!("opening `{open}` at {}", open.span)),
			)
		} else {
			None
		}
	}
}

pub struct ScopeExpression {}

impl ScopeExpression {
	pub fn new() -> Box<dyn Scope> {
		Box::new(ScopeExpression {})
	}
}

impl Scope for ScopeExpression {
	fn copy(&self) -> Box<dyn Scope> {
		Box::new(ScopeExpression {})
	}

	fn apply(&mut self, input: &dyn LexStream, _as_parent: bool) -> Action {
		match input.token() {
			Token::Break => {
				return Action::Stop;
			}
			_ => {
				if let Some(symbol) = input.next().symbol() {
					match symbol {
						";" | ":" => return Action::Stop,
						_ => {}
					}
				}
			}
		}
		Action::Output
	}
}

#[cfg(test)]
mod tests {
	use crate::{input, lexer};

	use super::*;

	#[test]
	fn scoped_stream_read() {
		let input = lexer::open(input::open_str("literal", "1 2 3"));
		let mut input = ScopedStream::new(input);

		assert_eq!(input.next().token, Token::Integer(1));
		input.advance();

		assert_eq!(input.next().token, Token::Integer(2));
		input.advance();

		assert_eq!(input.next().token, Token::Integer(3));
		assert_eq!(input.next().token, Token::Integer(3));
		input.advance();

		assert_eq!(input.next().token, Token::None);
	}

	#[test]
	fn scoped_stream_clone() {
		let input = lexer::open(input::open_str("literal", "1 2"));
		let mut a = ScopedStream::new(input);
		let mut b = a.clone();

		assert_eq!(a.next().token, Token::Integer(1));

		let mut c = a.clone();
		assert_eq!(c.next().token, Token::Integer(1));
		c.advance();
		assert_eq!(c.next().token, Token::Integer(2));
		assert_eq!(b.next().token, Token::Integer(1));
		assert_eq!(a.next().token, Token::Integer(1));

		a.advance();
		assert_eq!(a.next().token, Token::Integer(2));
		assert_eq!(b.next().token, Token::Integer(1));

		b.advance();
		assert_eq!(a.next().token, Token::Integer(2));
		assert_eq!(b.next().token, Token::Integer(2));
		assert_eq!(c.next().token, Token::Integer(2));
	}

	#[test]
	fn scoped_stream_scope() {
		let input = lexer::open(input::open_str("literal", "1 (2 3) 4"));
		let mut input = ScopedStream::new(input);

		assert_eq!(input.token(), Token::Integer(1));
		input.advance();
		assert_eq!(input.token(), Token::Symbol("("));

		input.enter(ScopeParenthesized::new(), ChildMode::Override);

		assert_eq!(input.token(), Token::Integer(2));
		input.advance();

		assert_eq!(input.token(), Token::Integer(3));
		input.advance();

		assert_eq!(input.token(), Token::None);
		input.leave();

		assert_eq!(input.token(), Token::Symbol(")"));
		input.advance();

		assert_eq!(input.token(), Token::Integer(4));
	}

	#[test]
	fn scope_indented_line() {
		let input = lexer::open(input::open_str("literal", "1\n\t2\n"));
		let mut input = ScopedStream::new(input);

		input.enter(ScopeLine::new(), ChildMode::Override);

		assert_eq!(input.token(), Token::Integer(1));
		input.advance();

		assert_eq!(input.token(), Token::Break);
		input.advance();

		assert_eq!(input.token(), Token::Indent);
		input.advance();

		assert_eq!(input.token(), Token::Integer(2));
		input.advance();

		assert_eq!(input.token(), Token::Break);
		input.advance();

		assert_eq!(input.token(), Token::Dedent);
		input.advance();

		assert_eq!(input.token(), Token::None);
	}

	#[test]
	fn scope_nested_indented_line() {
		let input = lexer::open(input::open_str("literal", "1\n\t2\n\t\t3\n"));
		let mut input = ScopedStream::new(input);

		input.enter(ScopeLine::new(), ChildMode::Secondary);

		assert_eq!(input.token(), Token::Integer(1));
		input.advance();

		assert_eq!(input.token(), Token::Break);
		input.advance();

		assert_eq!(input.token(), Token::Indent);
		input.advance();

		input.enter(ScopeLine::new(), ChildMode::Secondary);

		assert_eq!(input.token(), Token::Integer(2));
		input.advance();

		assert_eq!(input.token(), Token::Break);
		input.advance();

		assert_eq!(input.token(), Token::Indent);
		input.advance();

		assert_eq!(input.token(), Token::Integer(3));
		input.advance();

		assert_eq!(input.token(), Token::Break);
		input.advance();

		assert_eq!(input.token(), Token::Dedent);
		input.advance();

		assert_eq!(input.token(), Token::None);

		input.leave();

		assert_eq!(input.token(), Token::Dedent);
	}
}
