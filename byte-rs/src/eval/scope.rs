use std::cell::{Cell, Ref, RefCell, RefMut};

use crate::{
	lexer::{Lex, LexStream, Stream, Token},
	Error,
};

pub enum Action<'a> {
	None,
	EnterChild {
		scope: Box<dyn Scope<'a> + 'a>,
		mode: ChildMode,
	},
}

pub enum Filter {
	No,
	Skip,
	Stop { skip: bool },
}

pub enum Stop<'a> {
	Ok,
	Error(Error<'a>),
}

pub trait Scope<'a> {
	fn copy(&self) -> Box<dyn Scope<'a> + 'a>;
	fn check_action(&mut self, input: &mut dyn LexStream<'a>) -> Action<'a>;
	fn check_filter(&mut self, input: &dyn LexStream<'a>) -> Filter;
	fn leave(&self, input: &dyn LexStream<'a>) -> Stop<'a>;
}

pub enum Scoped<'a> {
	Root,
	Child {
		mode: ChildMode,
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
				scope,
				parent,
			} => Self::Child {
				mode: mode.clone(),
				scope: scope.copy(),
				parent: parent.clone(),
			},
		}
	}
}

#[derive(Clone, Debug)]
pub enum LexResult<'a> {
	Ok(Lex<'a>),
	Error(Error<'a>),
}

impl<'a> Default for Scoped<'a> {
	fn default() -> Self {
		Scoped::Root
	}
}

impl<'a> Scoped<'a> {
	pub fn next(&mut self, input: &mut dyn LexStream<'a>) -> LexResult<'a> {
		loop {
			match self {
				// the root scope just reads the next value without filtering
				Scoped::Root => {
					let next = input.next();
					return LexResult::Ok(next);
				}

				Scoped::Child {
					mode,
					scope,
					parent,
				} => {
					let mut stopped = false;
					if !mode.is_override() {
						if let Some(parent) = parent {
							match parent.check_filter(input) {
								Filter::No => {}
								Filter::Skip => {
									input.advance();
									continue;
								}
								Filter::Stop { .. } => {
									stopped = true;
								}
							}
						}
					}

					stopped = stopped || input.next().is_none();

					// check the current scope action
					match scope.check_action(input) {
						Action::None => {
							let next = input.next();
							let skip = match scope.check_filter(input) {
								Filter::No => false,
								Filter::Skip => true,
								Filter::Stop { skip } => {
									stopped = true;
									if skip {
										input.advance();
									}
									false
								}
							};
							if !stopped {
								if skip {
									input.advance();
								} else {
									return LexResult::Ok(next);
								}
							}
						}
						Action::EnterChild { scope, mode } => {
							let current = std::mem::take(self);
							*self = Scoped::Child {
								mode,
								scope,
								parent: Some(current.into()),
							};
							continue;
						}
					}

					if stopped {
						if let Stop::Error(error) = scope.leave(input) {
							return LexResult::Error(error);
						}
						if let Some(parent) = parent {
							let parent = std::mem::take(&mut **parent);
							std::mem::replace(self, parent);
						} else {
							return LexResult::Ok(input.next().as_none());
						}
					}
				}
			}
		}
	}

	fn enter(&mut self, scope: Box<dyn Scope<'a>>, mode: ChildMode) {
		let me = std::mem::take(self);
		*self = Scoped::Child {
			mode,
			scope: scope,
			parent: Some(Box::new(me)),
		};
	}

	fn check_filter(&mut self, input: &dyn LexStream<'a>) -> Filter {
		match self {
			Scoped::Root => Filter::No,
			Scoped::Child {
				mode,
				scope,
				parent,
			} => {
				if !mode.is_override() {
					if let Some(parent) = parent {
						match parent.check_filter(input) {
							Filter::No => {}
							filter @ (Filter::Skip | Filter::Stop { .. }) => return filter,
						}
					}
				}
				scope.check_filter(input)
			}
		}
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

	fn check_action(&mut self, input: &mut dyn LexStream<'a>) -> Action<'a> {
		Action::None
	}

	fn check_filter(&mut self, input: &dyn LexStream<'a>) -> Filter {
		match input.token() {
			Token::Indent => {
				self.level += 1;
				if self.level == 1 {
					Filter::Skip // skip first indent
				} else {
					Filter::No
				}
			}
			Token::Break => {
				if self.level == 1 {
					let mut input = input.copy();
					input.advance();
					if input.token() == Token::Dedent {
						Filter::Skip // strip break before last dedent
					} else {
						Filter::No
					}
				} else {
					Filter::No
				}
			}
			Token::Dedent => {
				self.level -= 1;
				if self.level == 0 {
					Filter::Stop { skip: true }
				} else {
					Filter::No
				}
			}
			_ => Filter::No,
		}
	}

	fn leave(&self, input: &dyn LexStream<'a>) -> Stop<'a> {
		if self.level > 0 {
			panic!(
				"lexer generated unbalanced indentation for {}",
				input.span()
			)
		}
		Stop::Ok
	}
}

struct ScopeLine {
	ended: bool,
}

impl<'a> ScopeLine {
	fn new() -> Box<dyn Scope<'a> + 'a> {
		Box::new(ScopeLine { ended: false })
	}
}

impl<'a> Scope<'a> for ScopeLine {
	fn copy(&self) -> Box<dyn Scope<'a> + 'a> {
		Box::new(ScopeLine { ended: self.ended })
	}

	fn check_action(&mut self, input: &mut dyn LexStream<'a>) -> Action<'a> {
		if input.token() == Token::Break {
			input.advance();
			if input.token() == Token::Indent {
				Action::EnterChild {
					scope: ScopeIndented::new(),
					mode: ChildMode::Override,
				}
			} else {
				self.ended = true;
				Action::None
			}
		} else {
			Action::None
		}
	}

	fn check_filter(&mut self, input: &dyn LexStream<'a>) -> Filter {
		if self.ended {
			Filter::Stop { skip: false }
		} else {
			Filter::No
		}
	}

	fn leave(&self, input: &dyn LexStream<'a>) -> Stop<'a> {
		Stop::Ok
	}
}

struct ScopeParenthesized<'a> {
	open: bool,
	err: Option<Error<'a>>,
	lex: Lex<'a>,
	sta: &'static str,
	end: &'static str,
}

impl<'a> Scope<'a> for ScopeParenthesized<'a> {
	fn copy(&self) -> Box<dyn Scope<'a> + 'a> {
		Box::new(ScopeParenthesized::<'a> {
			open: self.open,
			err: self.err.clone(),
			lex: self.lex,
			sta: self.sta,
			end: self.end,
		})
	}

	fn check_action(&mut self, input: &mut dyn LexStream<'a>) -> Action<'a> {
		Action::None
	}

	fn check_filter(&mut self, input: &dyn LexStream<'a>) -> Filter {
		if !self.open {
			self.open = true;
			let sta = self.sta;
			let cur = input.next();
			assert_eq!(
				cur.symbol(),
				Some(self.sta),
				"parenthesis for scope does not match (expected {sta}, got {cur})"
			);
			Filter::Skip
		} else if input.next().symbol() == Some(self.end) {
			self.open = false;
			Filter::Stop { skip: true }
		} else {
			match input.token() {
				Token::Break => {
					let mut input = input.copy();
					input.advance();
					if input.token() != Token::Indent {
						self.err = Some(Error::ExpectedIndent(input.span()));
						Filter::Stop { skip: false }
					} else {
						Filter::Skip
					}
				}
				_ => Filter::No,
			}
		}
	}

	fn leave(&self, input: &dyn LexStream<'a>) -> Stop<'a> {
		if self.open {
			let left = self.lex;
			let next = input.next();
			Stop::Error(
				Error::ExpectedSymbol(self.end, next.span)
					.at(format!("opening `{left}` at {}", left.span)),
			)
		} else if let Some(err) = &self.err {
			Stop::Error(err.clone())
		} else {
			Stop::Ok
		}
	}
}

pub struct ScopedStream<'a> {
	state: RefCell<(Box<dyn LexStream<'a> + 'a>, Scoped<'a>)>,
	next: Cell<Option<Lex<'a>>>,
}

impl<'a> ScopedStream<'a> {
	pub fn new<T: LexStream<'a> + 'a>(input: T) -> ScopedStream<'a> {
		ScopedStream {
			state: RefCell::new((Box::new(input), Scoped::Root)),
			next: Cell::new(None),
		}
	}

	pub fn enter<T: Scope<'a>>(&mut self, new_scope: T, isolated: bool) {
		let new_scope = Box::new(new_scope);
		let mut state = self.state.borrow_mut();
		let (_, scope) = &mut *state;
	}

	pub fn input(&self) -> Ref<dyn LexStream<'a>> {
		let input = self.state.borrow();
		Ref::map(input, |x| &*x.0)
	}

	pub fn input_mut(&self) -> RefMut<dyn LexStream<'a>> {
		let input = self.state.borrow_mut();
		RefMut::map(input, |x| &mut *x.0)
	}
}

impl<'a> Clone for ScopedStream<'a> {
	fn clone(&self) -> Self {
		let (input, scope) = &*self.state.borrow();

		Self {
			state: RefCell::new((input.copy(), scope.clone())),
			next: self.next.clone(),
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
			let mut state = self.state.borrow_mut();
			let (input, scope) = &mut *state;
			let next = match scope.next(&mut **input) {
				LexResult::Ok(next) => next,
				LexResult::Error(error) => {
					input.add_error(error);
					input.next().as_none()
				}
			};
			self.next.set(Some(next));
			next
		}
	}

	fn advance(&mut self) {
		let mut state = self.state.borrow_mut();
		let mut input = &mut state.0;
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
}
