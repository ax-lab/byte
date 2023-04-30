use std::sync::{
	atomic::{AtomicUsize, Ordering},
	Arc,
};

use crate::core::error::*;
use crate::core::input::*;

use super::*;

/// Manages the state and apply basic lexing rules for indentation.
#[derive(Clone)]
pub struct Indent {
	current: Option<Arc<IndentLevel>>,
	closing: Option<IndentBlock>,
}

impl Default for Indent {
	fn default() -> Self {
		Self {
			current: None,
			closing: None,
		}
	}
}

impl Indent {
	pub fn new() -> Self {
		Indent {
			current: None,
			closing: None,
		}
	}

	pub fn pop_levels(&mut self, levels: usize) -> Result<(), ()> {
		for _ in 0..levels {
			let ok = if let Some(current) = &self.current {
				if let Kind::Level(..) = current.kind {
					self.pop();
					true
				} else {
					false
				}
			} else {
				false
			};
			if !ok {
				return Err(());
			}
		}
		Ok(())
	}

	pub fn check_for_closed_regions(&mut self, next: &TokenAt) -> bool {
		if self.closing.is_some() {
			false
		} else {
			let mut current = self.current.clone();
			while let Some(node) = current {
				match node.kind {
					Kind::Level(_) => {}
					Kind::Block { id, region, .. } => {
						if region.should_close(next) {
							self.close_region(IndentBlock(id));
							return true;
						} else if !region.is_soft() {
							return false;
						}
					}
				}
				current = node.prev.clone();
			}
			false
		}
	}

	pub fn open_region(&mut self, region: IndentRegion) -> IndentBlock {
		static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

		if self.closing.is_some() {
			panic!("open_region: cannot open an indent region while closing another");
		}
		let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
		self.push(Kind::Block {
			level: self.level(),
			region,
			id,
		});
		IndentBlock(id)
	}

	pub fn close_region(&mut self, block: IndentBlock) {
		if self.closing.is_some() {
			panic!("close_region: already closing an indent region");
		}
		self.closing = Some(block);
	}

	pub fn check_indent(&mut self, input: &Cursor, errors: &mut ErrorList) -> Option<TokenAt> {
		self.check_indent_token(input, errors).map(|token| {
			let span = Span {
				sta: input.clone(),
				end: input.clone(),
			};
			TokenAt(span, token)
		})
	}

	fn check_indent_token(&mut self, input: &Cursor, errors: &mut ErrorList) -> Option<Token> {
		// Closing a region takes precedence over anything. If we are closing a
		// region, generate any pending Dedent before processing anything.
		if let Some(IndentBlock(close_id)) = self.closing {
			match self.pop().kind {
				Kind::Level(..) => return Some(Token::Dedent),
				Kind::Block { id, region, .. } => {
					if id == close_id {
						self.closing = None;
					} else if !region.is_soft() {
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
							errors.add_error(
								Error::new(LexerError::InvalidDedentIndent).at(input.pos()),
							);
							None
						} else {
							// ...all good, generate the Token::Dedent
							Some(Token::Dedent)
						}
					}
					Kind::Block { .. } => {
						// cannot dedent out of an enclosed region
						errors.add_error(
							Error::new(LexerError::InvalidDedentInRegion).at(input.pos()),
						);
						None
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
		self.current = Some(Arc::new(new_level));
	}

	fn pop(&mut self) -> Arc<IndentLevel> {
		let current = self.current.take();
		let current = current.expect("pop empty indent stack");
		self.current = current.prev.clone();
		current
	}
}

#[derive(Copy, Clone)]
pub enum IndentRegion {
	UntilSymbol(&'static str),
}

impl IndentRegion {
	fn should_close(&self, next: &TokenAt) -> bool {
		if next.is_none() {
			true
		} else {
			match self {
				IndentRegion::UntilSymbol(symbol) => next.symbol() == Some(symbol),
			}
		}
	}

	/// Returns true if this is a soft region that can be closed with the
	/// parent region.
	fn is_soft(&self) -> bool {
		match self {
			IndentRegion::UntilSymbol(_) => false,
		}
	}
}

#[derive(Clone)]
pub struct IndentBlock(usize);

struct IndentLevel {
	kind: Kind,
	prev: Option<Arc<IndentLevel>>,
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
	Block {
		level: usize,
		id: usize,
		region: IndentRegion,
	},
}
