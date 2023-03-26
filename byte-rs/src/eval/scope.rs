use std::{
	cell::{Cell, Ref, RefCell, RefMut},
	collections::VecDeque,
};

use crate::{
	lexer::{Lex, LexStream, Stream, Token},
	Error,
};

/// Scope actions at a given input position.
#[derive(Copy, Clone)]
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

pub trait Scope<'a> {
	fn copy(&self) -> Box<dyn Scope<'a> + 'a>;

	/// Apply the scope to the current input stream position.
	///
	/// This is called when the scope is first applied to an input stream and
	/// each time the input advances to the next token.
	///
	/// Returns the action relative to the current input token.
	fn apply(&mut self, input: &dyn LexStream<'a>) -> Action;

	fn leave(&self, input: &dyn LexStream<'a>) -> Option<Error<'a>> {
		None
	}
}

pub enum Scoped<'a> {
	Root,
	Child {
		mode: ChildMode,
		last: Option<(Lex<'a>, Action)>,
		scope: Box<dyn Scope<'a> + 'a>,
		parent: Option<Box<Scoped<'a>>>,
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

impl<'a> Clone for Scoped<'a> {
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

impl<'a> Default for Scoped<'a> {
	fn default() -> Self {
		Scoped::Root
	}
}

impl<'a> Scoped<'a> {
	pub fn apply(&mut self, input: &dyn LexStream<'a>) -> Action {
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
						match parent.apply(input) {
							Action::Output => {}
							Action::Skip => return Action::Skip,
							action @ Action::Stop | action @ Action::SkipAndStop => return action,
						}
					}
				}

				scope.apply(input)
			}
		}
	}

	pub fn enter(&mut self, scope: Box<dyn Scope<'a> + 'a>, mode: ChildMode) {
		let me = std::mem::take(self);
		*self = Scoped::Child {
			mode,
			last: None,
			scope: scope,
			parent: Some(Box::new(me)),
		};
	}

	pub fn leave(&mut self, input: &dyn LexStream<'a>) -> Option<Error<'a>> {
		match self {
			Scoped::Root => panic!("trying to leave root scope"),
			Scoped::Child { scope, parent, .. } => {
				if let Some(parent) = parent {
					let parent = std::mem::take(&mut *parent);
					let result = scope.leave(input);
					std::mem::replace(self, *parent);
					result
				} else {
					panic!("trying to leave scope without a parent");
				}
			}
		}
	}
}

pub struct ScopedStream<'a> {
	input: RefCell<Box<dyn LexStream<'a> + 'a>>,
	scope: RefCell<(Scoped<'a>)>,
	next: Cell<Option<Lex<'a>>>,
	done: Cell<bool>,
}

impl<'a> ScopedStream<'a> {
	pub fn new<T: LexStream<'a> + 'a>(input: T) -> ScopedStream<'a> {
		ScopedStream {
			input: RefCell::new(Box::new(input)),
			scope: RefCell::new(Scoped::Root),
			next: Cell::new(None),
			done: Cell::new(false),
		}
	}

	pub fn enter_parenthesis(&mut self) {
		let scope = ScopeParenthesized::new();
		self.enter(Box::new(scope), ChildMode::Override);
	}

	pub fn enter(&mut self, scope: Box<dyn Scope<'a> + 'a>, mode: ChildMode) {
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

	pub fn input(&self) -> Ref<dyn LexStream<'a>> {
		let input = self.input.borrow();
		Ref::map(input, |x| &**x)
	}

	pub fn input_mut(&self) -> RefMut<dyn LexStream<'a>> {
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
				match scope.apply(&*input) {
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

impl<'a> Clone for ScopedStream<'a> {
	fn clone(&self) -> Self {
		Self {
			input: RefCell::new(self.input().copy()),
			scope: RefCell::new(self.scope.borrow().clone()),
			next: self.next.clone(),
			done: self.done.clone(),
		}
	}
}

impl<'a> LexStream<'a> for ScopedStream<'a> {
	fn copy(&self) -> Box<dyn LexStream<'a> + 'a> {
		Box::new(self.clone())
	}

	fn source(&self) -> &'a dyn crate::input::Input {
		self.input().source()
	}

	fn next(&self) -> Lex<'a> {
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

	fn errors(&self) -> Vec<Error<'a>> {
		self.input().errors()
	}

	fn add_error(&mut self, error: Error<'a>) {
		self.input_mut().add_error(error)
	}

	fn has_errors(&self) -> bool {
		self.input().has_errors()
	}
}

struct ScopeIndented {
	level: usize,
}

impl<'a> ScopeIndented {
	fn new() -> Box<dyn Scope<'a> + 'a> {
		Box::new(ScopeIndented { level: 0 })
	}
}

impl<'a> Scope<'a> for ScopeIndented {
	fn copy(&self) -> Box<dyn Scope<'a> + 'a> {
		Box::new(ScopeIndented { level: self.level })
	}

	fn apply(&mut self, input: &dyn LexStream<'a>) -> Action {
		if self.level == 0 {
			if input.token() != Token::Indent {
				panic!("indented scope expected an Indent at {}", input.span());
			}
			self.level += 1;
			return Action::Skip;
		}
		let next = input.next();
		match next.token {
			Token::Indent => {
				self.level += 1;
				Action::Output
			}
			Token::Dedent => {
				self.level -= 1;
				if self.level == 0 {
					Action::SkipAndStop
				} else {
					Action::Output
				}
			}
			Token::Break => {
				if self.level == 1 {
					// trim the line break before the final dedent
					let mut input = input.copy();
					input.advance();
					if input.token() == Token::Dedent {
						Action::Skip
					} else {
						Action::Output
					}
				} else {
					Action::Output
				}
			}
			_ => Action::Output,
		}
	}

	fn leave(&self, input: &dyn LexStream<'a>) -> Option<Error<'a>> {
		if self.level > 0 {
			panic!(
				"lexer generated unbalanced indentation for {}",
				input.span()
			)
		}
		None
	}
}

struct ScopeLine {
	ended: bool,
	level: usize,
}

impl<'a> ScopeLine {
	fn new() -> Box<dyn Scope<'a> + 'a> {
		Box::new(ScopeLine {
			ended: false,
			level: 0,
		})
	}
}

impl<'a> Scope<'a> for ScopeLine {
	fn copy(&self) -> Box<dyn Scope<'a> + 'a> {
		Box::new(ScopeLine {
			ended: self.ended,
			level: self.level,
		})
	}

	fn apply(&mut self, input: &dyn LexStream<'a>) -> Action {
		if self.ended {
			return Action::Stop;
		}

		match input.token() {
			Token::Indent => {
				self.level += 1;
				Action::Output
			}
			Token::Dedent => {
				self.level -= 1;
				if self.level == 0 {
					self.ended = true;
				}
				Action::Output
			}
			Token::Break => {
				if self.level == 0 {
					let mut input = input.copy();
					input.advance();
					if input.token() == Token::Indent {
						Action::Skip
					} else {
						Action::SkipAndStop
					}
				} else {
					Action::SkipAndStop
				}
			}
			_ => Action::Output,
		}
	}
}

struct ScopeParenthesized<'a> {
	open: VecDeque<Lex<'a>>,
}

impl<'a> ScopeParenthesized<'a> {
	fn new() -> Self {
		ScopeParenthesized {
			open: Default::default(),
		}
	}
}

impl<'a> Scope<'a> for ScopeParenthesized<'a> {
	fn copy(&self) -> Box<dyn Scope<'a> + 'a> {
		Box::new(ScopeParenthesized::<'a> {
			open: self.open.clone(),
		})
	}

	fn apply(&mut self, input: &dyn LexStream<'a>) -> Action {
		if let Some(open) = self.open.front() {
			if input.next().symbol() == open.token.get_closing() {
				self.open.pop_front();
				if self.open.len() == 0 {
					Action::Stop
				} else {
					Action::Output
				}
			} else {
				if let Some(opening) = input.token().get_closing() {
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

	fn leave(&self, input: &dyn LexStream<'a>) -> Option<Error<'a>> {
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

#[cfg(test)]
mod tests {
	use crate::lexer;

	use super::*;

	#[test]
	fn scoped_stream_read() {
		let input = lexer::open(&"1 2 3");
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
		let input = lexer::open(&"1 2");
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
		let input = lexer::open(&"1 (2 3) 4");
		let mut input = ScopedStream::new(input);

		assert_eq!(input.token(), Token::Integer(1));
		input.advance();
		assert_eq!(input.token(), Token::Symbol("("));

		input.enter_parenthesis();

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
}
