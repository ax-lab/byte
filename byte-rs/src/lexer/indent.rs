use std::{
	rc::Rc,
	sync::atomic::{AtomicUsize, Ordering},
};

use crate::core::error::*;
use crate::core::input::*;

use super::*;

/// Manages the state and apply basic lexing rules for indentation.
#[derive(Clone)]
pub struct Indent {
	current: Option<Rc<IndentLevel>>,
	closing: Option<IndentRegion>,
}

impl Indent {
	pub fn new() -> Self {
		Indent {
			current: None,
			closing: None,
		}
	}

	pub fn open_region(&mut self) -> IndentRegion {
		static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

		if self.closing.is_some() {
			panic!("open_region: cannot open an indent region while closing another");
		}
		let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
		self.push(Kind::Block {
			level: self.level(),
			id,
		});
		IndentRegion(id)
	}

	pub fn close_region(&mut self, region: IndentRegion) {
		if self.closing.is_some() {
			panic!("close_region: already closing an indent region");
		}
		self.closing = Some(region);
	}

	pub fn check_indent(&mut self, input: &Cursor, errors: &mut ErrorList) -> Option<Token> {
		// Closing a region takes precedence over anything. If we are closing a
		// region, generate any pending Dedent before processing anything.
		if let Some(IndentRegion(close_id)) = self.closing {
			match self.pop().kind {
				Kind::Level(..) => return Some(Token::Dedent),
				Kind::Block { id, .. } => {
					if id == close_id {
						self.closing = None;
					} else {
						panic!("unbalanced indent region closing");
					}
				}
			}
		}

		// we only want to process indentation at the start of the line or at
		// the end of the input
		let new_level = if !input.at_end() {
			let indent = input.indent();
			if indent == input.col() {
				indent
			} else {
				// not at the start of the line
				return None;
			}
		} else {
			// indent is always zero at the end of the input
			0
		};

		// check if there is any indentation on the stack
		if let Some(current) = &self.current {
			let level = current.level();
			if new_level > level {
				// increase in indentation, save the new level and output Indent
				self.push(Kind::Level(new_level));
				Some(Token::Indent)
			} else if new_level < level {
				// decrease in indentation
				match current.kind {
					Kind::Level(..) => {
						// pop a single level of indentation in each call
						self.pop();

						// compare the dedent with the previous level...
						let base_level = if let Some(current) = &self.current {
							current.level()
						} else {
							0
						};

						if new_level > base_level {
							// ...don't allow a dedent between two levels
							errors.add(Error::new(input.pos(), LexerError::InvalidDedentIndent));
							Some(Token::Invalid)
						} else {
							// ...all good, generate the Token::Dedent
							Some(Token::Dedent)
						}
					}
					Kind::Block { .. } => {
						// cannot dedent out of an enclosed region
						errors.add(Error::new(input.pos(), LexerError::InvalidDedentInRegion));
						Some(Token::Invalid)
					}
				}
			} else {
				// no changes
				None
			}
		} else if new_level > 0 {
			// first indentation
			self.push(Kind::Level(new_level));
			Some(Token::Indent)
		} else {
			// no indentation and no changes
			None
		}
	}

	fn level(&self) -> usize {
		if let Some(current) = &self.current {
			current.level()
		} else {
			0
		}
	}

	fn push(&mut self, kind: Kind) {
		let prev = self.current.take();
		let new_level = IndentLevel { kind, prev };
		self.current = Some(Rc::new(new_level));
	}

	fn pop(&mut self) -> Rc<IndentLevel> {
		let current = self.current.take();
		let current = current.expect("pop empty indent stack");
		self.current = current.prev.clone();
		current
	}
}

#[derive(Clone)]
pub struct IndentRegion(usize);

struct IndentLevel {
	kind: Kind,
	prev: Option<Rc<IndentLevel>>,
}

impl IndentLevel {
	fn level(&self) -> usize {
		match self.kind {
			Kind::Level(level) => level,
			Kind::Block { level, .. } => level,
		}
	}
}

enum Kind {
	Level(usize),
	Block { level: usize, id: usize },
}
