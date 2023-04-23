use std::sync::Arc;

use crate::core::error::*;
use crate::core::input::*;

use super::*;

/// Low-level stream of raw [`Token`] from an input.
///
/// This is a thin wrapper over a [`Scanner`] and input [`Cursor`]
/// providing access to a stream of tokens from that position.
///
/// ## Note for future implementation
///
/// This low-level [`TokenStream`] is the perfect position to inject
/// custom tokenization, such as token generators, and processing lexing
/// pragmas from the input.
///
/// It sits at a low enough level that it's not encumbered with higher level
/// lexer semantics such as indentation. It also has access to both parsing
/// and generating skipped tokens such as [`Comment`].
///
/// On the other hand, it sits at a high enough level to not need to handle
/// raw text parsing.
#[derive(Clone)]
pub struct TokenStream {
	scanner: Arc<Scanner>,

	/// Current input position.
	input: Cursor,

	/// Current list of errors.
	errors: ErrorList,
}

impl TokenStream {
	pub fn new(input: Cursor, scanner: Scanner) -> Self {
		TokenStream {
			errors: ErrorList::new(),
			input,
			scanner: Arc::new(scanner),
		}
	}

	pub fn pos(&self) -> &Cursor {
		&self.input
	}

	pub fn errors(&self) -> &ErrorList {
		&self.errors
	}

	pub fn errors_mut(&mut self) -> &mut ErrorList {
		&mut self.errors
	}

	pub fn config<F: FnOnce(&mut Scanner)>(&mut self, config: F) {
		let scanner = Arc::make_mut(&mut self.scanner);
		config(scanner)
	}

	pub fn skip(&mut self) {
		self.scanner.skip(&mut self.input);
	}

	pub fn read(&mut self) -> TokenAt {
		self.skip();

		let sta = self.input.clone();
		let next = self.scanner.read(&mut self.input, &mut self.errors);
		let span = Span {
			sta,
			end: self.input.clone(),
		};
		TokenAt(span, next)
	}
}
