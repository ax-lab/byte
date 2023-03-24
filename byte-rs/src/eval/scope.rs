use std::cell::{Cell, RefCell};

use crate::{
	lexer::{Lex, LexStream, Stream, Token},
	Error,
};

pub enum Action<'a> {
	None,
	EnterChild {
		scope: Box<dyn Scope<'a> + 'a>,
		isolated: bool,
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
	fn check_action(&mut self, input: &mut Stream<'a>) -> Action<'a>;
	fn check_filter(&mut self, input: &Stream<'a>) -> Filter;
	fn leave(&self, input: &Stream<'a>) -> Stop<'a>;
}

pub enum Scoped<'a> {
	Root,
	Child {
		isolated: bool,
		scope: Box<dyn Scope<'a> + 'a>,
		parent: Option<Box<Scoped<'a>>>,
	},
}

impl<'a> Clone for Scoped<'a> {
	fn clone(&self) -> Self {
		match self {
			Self::Root => Self::Root,
			Self::Child {
				isolated,
				scope,
				parent,
			} => Self::Child {
				isolated: isolated.clone(),
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
	pub fn next(&mut self, input: &mut Stream<'a>) -> LexResult<'a> {
		loop {
			match self {
				// the root scope just reads the next value without filtering
				Scoped::Root => {
					let next = input.value();
					return LexResult::Ok(next);
				}

				Scoped::Child {
					isolated,
					scope,
					parent,
				} => {
					// non-isolated child scopes defer to the parent first
					let mut stopped = false;
					if !*isolated {
						if let Some(parent) = parent {
							match parent.check_filter(input) {
								Filter::No => {}
								Filter::Skip => {
									input.next();
									continue;
								}
								Filter::Stop { .. } => {
									stopped = true;
								}
							}
						}
					}

					stopped = stopped || input.value().is_none();

					// check the current scope action
					match scope.check_action(input) {
						Action::None => {
							let next = input.value();
							let skip = match scope.check_filter(input) {
								Filter::No => false,
								Filter::Skip => true,
								Filter::Stop { skip } => {
									stopped = true;
									if skip {
										input.next();
									}
									false
								}
							};
							if !stopped {
								if skip {
									input.next();
								} else {
									return LexResult::Ok(next);
								}
							}
						}
						Action::EnterChild { scope, isolated } => {
							let current = std::mem::take(self);
							*self = Scoped::Child {
								isolated,
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
							return LexResult::Ok(input.value().as_none());
						}
					}
				}
			}
		}
	}

	pub fn read(&mut self, input: &mut Stream<'a>) -> LexResult<'a> {
		loop {
			match self {
				// the root scope just reads the next value without filtering
				Scoped::Root => {
					let next = input.value();
					input.next();
					return LexResult::Ok(next);
				}

				Scoped::Child {
					isolated,
					scope,
					parent,
				} => {
					// non-isolated child scopes defer to the parent first
					let mut stopped = false;
					if !*isolated {
						if let Some(parent) = parent {
							match parent.check_filter(input) {
								Filter::No => {}
								Filter::Skip => {
									input.next();
									continue;
								}
								Filter::Stop { skip } => {
									if skip {
										input.next();
									}
									stopped = true;
								}
							}
						}
					}

					stopped = stopped || input.value().is_none();

					// check the current scope action
					match scope.check_action(input) {
						Action::None => {
							let next = input.value();
							let skip = match scope.check_filter(input) {
								Filter::No => false,
								Filter::Skip => true,
								Filter::Stop { skip } => {
									stopped = true;
									if skip {
										input.next();
									}
									false
								}
							};
							if !stopped {
								input.next();
								if !skip {
									return LexResult::Ok(next);
								}
							}
						}
						Action::EnterChild { scope, isolated } => {
							let current = std::mem::take(self);
							*self = Scoped::Child {
								isolated,
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
							return LexResult::Ok(input.value().as_none());
						}
					}
				}
			}
		}
	}

	fn check_filter(&mut self, input: &Stream<'a>) -> Filter {
		match self {
			Scoped::Root => Filter::No,
			Scoped::Child {
				isolated,
				scope,
				parent,
			} => {
				if !*isolated {
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

	fn check_action(&mut self, input: &mut Stream<'a>) -> Action<'a> {
		Action::None
	}

	fn check_filter(&mut self, input: &Stream) -> Filter {
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
					let mut input = input.clone();
					input.next();
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

	fn leave(&self, input: &Stream<'a>) -> Stop<'a> {
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

	fn check_action(&mut self, input: &mut Stream<'a>) -> Action<'a> {
		if input.token() == Token::Break {
			input.next();
			if input.token() == Token::Indent {
				Action::EnterChild {
					scope: ScopeIndented::new(),
					isolated: true,
				}
			} else {
				self.ended = true;
				Action::None
			}
		} else {
			Action::None
		}
	}

	fn check_filter(&mut self, input: &Stream) -> Filter {
		if self.ended {
			Filter::Stop { skip: false }
		} else {
			Filter::No
		}
	}

	fn leave(&self, input: &Stream<'a>) -> Stop<'a> {
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

	fn check_action(&mut self, input: &mut Stream<'a>) -> Action<'a> {
		Action::None
	}

	fn check_filter(&mut self, input: &Stream<'a>) -> Filter {
		if !self.open {
			self.open = true;
			let sta = self.sta;
			let cur = input.value();
			assert_eq!(
				cur.symbol(),
				Some(self.sta),
				"parenthesis for scope does not match (expected {sta}, got {cur})"
			);
			Filter::Skip
		} else if input.value().symbol() == Some(self.end) {
			self.open = false;
			Filter::Stop { skip: true }
		} else {
			match input.token() {
				Token::Break => {
					let mut input = input.clone();
					input.next();
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

	fn leave(&self, input: &Stream<'a>) -> Stop<'a> {
		if self.open {
			let left = self.lex;
			let next = input.value();
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

#[derive(Clone)]
pub struct ScopedStream<'a> {
	state: RefCell<(Stream<'a>, Scoped<'a>)>,
	next: Cell<Option<Lex<'a>>>,
}

impl<'a> ScopedStream<'a> {
	pub fn new(input: Stream<'a>) -> ScopedStream<'a> {
		ScopedStream {
			state: RefCell::new((input, Scoped::Root)),
			next: Cell::new(None),
		}
	}

	pub fn value(&self) -> Lex<'a> {
		if let Some(next) = self.next.get() {
			next
		} else {
			let mut state = self.state.borrow_mut();
			let (input, scope) = &mut *state;
			let next = match scope.next(input) {
				LexResult::Ok(next) => next,
				LexResult::Error(error) => {
					input.add_error(error);
					input.value().as_none()
				}
			};
			self.next.set(Some(next));
			next
		}
	}

	pub fn next(&mut self) {
		let mut state = self.state.borrow_mut();
		let mut input = &mut state.0;
		input.next();
		self.next.set(None);
	}
}
