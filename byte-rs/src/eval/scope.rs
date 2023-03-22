use crate::lexer::{Lex, Stream};

pub enum Action {
	None,
	Complete,
	EnterChild {
		scope: Box<dyn Scope>,
		isolated: bool,
	},
}

pub trait Scope {
	fn check_action(&mut self, input: &mut Stream) -> Action;
	fn should_skip(&mut self, input: &Stream) -> bool;
}

pub enum Scoped {
	Root,
	Child {
		isolated: bool,
		scope: Box<dyn Scope>,
		parent: Option<Box<Scoped>>,
	},
}

impl Default for Scoped {
	fn default() -> Self {
		Scoped::Root
	}
}

impl Scoped {
	pub fn next<'a>(&mut self, input: &mut Stream<'a>) -> Lex<'a> {
		loop {
			// a child scope will only operate on its parent output
			if let Scoped::Child { parent, .. } = self {
				if let Some(parent) = parent {
					if parent.should_skip(input) {
						input.next();
						continue;
					}
				}
			}

			match self {
				// the root scope just reads the next value without filtering
				Scoped::Root => {
					let next = input.value();
					input.next();
					return next;
				}

				Scoped::Child {
					isolated,
					scope,
					parent,
				} => {
					// non-isolated child scopes respect the parent skipping
					if !*isolated {
						if let Some(parent) = parent {
							if parent.should_skip(input) {
								input.next();
								continue;
							}
						}
					}

					// check the current scope action
					match scope.check_action(input) {
						Action::None => {
							let next = input.value();
							let skip = scope.should_skip(&input);
							input.next();
							if !skip {
								return next;
							}
						}
						Action::Complete => {
							if let Some(parent) = parent {
								let parent = std::mem::take(&mut **parent);
								std::mem::replace(self, parent);
							} else {
								return input.value().as_none();
							}
						}
						Action::EnterChild { scope, isolated } => {
							let current = std::mem::take(self);
							*self = Scoped::Child {
								isolated,
								scope,
								parent: Some(current.into()),
							}
						}
					}
				}
			}
		}
	}

	fn should_skip(&mut self, input: &Stream) -> bool {
		match self {
			Scoped::Root => false,
			Scoped::Child {
				isolated,
				scope,
				parent,
			} => {
				let should_skip = if *isolated {
					false
				} else {
					if let Some(parent) = parent {
						parent.should_skip(input)
					} else {
						false
					}
				};
				should_skip || scope.should_skip(input)
			}
		}
	}
}
