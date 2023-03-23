use crate::{
	lexer::{Lex, Stream, Token},
	Error,
};

pub enum Action {
	None,
	EnterChild {
		scope: Box<dyn Scope>,
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

pub trait Scope {
	fn check_action(&mut self, input: &mut Stream) -> Action;
	fn check_filter(&mut self, input: &Stream) -> Filter;
	fn leave<'a>(&self, input: &Stream<'a>) -> Stop<'a>;
}

pub enum Scoped {
	Root,
	Child {
		isolated: bool,
		scope: Box<dyn Scope>,
		parent: Option<Box<Scoped>>,
	},
}

#[derive(Clone, Debug)]
pub enum LexResult<'a> {
	Ok(Lex<'a>),
	Error(Error<'a>),
}

impl Default for Scoped {
	fn default() -> Self {
		Scoped::Root
	}
}

impl Scoped {
	pub fn next<'a>(&mut self, input: &mut Stream<'a>) -> LexResult<'a> {
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
									stopped = false;
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

	fn check_filter(&mut self, input: &Stream) -> Filter {
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

impl ScopeIndented {
	fn new() -> Box<dyn Scope> {
		Box::new(ScopeIndented { level: 0 })
	}
}

impl Scope for ScopeIndented {
	fn check_action(&mut self, input: &mut Stream) -> Action {
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

	fn leave<'a>(&self, input: &Stream<'a>) -> Stop<'a> {
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

impl ScopeLine {
	fn new() -> Box<dyn Scope> {
		Box::new(ScopeLine { ended: false })
	}
}

impl Scope for ScopeLine {
	fn check_action(&mut self, input: &mut Stream) -> Action {
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

	fn leave<'a>(&self, input: &Stream<'a>) -> Stop<'a> {
		Stop::Ok
	}
}
