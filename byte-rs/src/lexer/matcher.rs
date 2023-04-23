use crate::core::error::*;
use crate::core::input::*;

use super::*;

/// Trait used by the [`Lexer`] to match tokens.
pub trait Matcher: Send + Sync {
	fn try_match(&self, next: char, input: &mut Cursor, errors: &mut ErrorList) -> Option<Token>;
}
