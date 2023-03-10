use super::lex_string::LexString;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Token {
	Break,
	Indent,
	Dedent,
	Identifier,
	Integer(u64),
	Literal(LexString),
	Symbol(&'static str),
}

impl Token {
	/// Returns the closing symbol for an opening parenthesis token.
	pub fn closing(&self) -> Option<&'static str> {
		let right = match self {
			Token::Symbol("(") => ")",
			_ => return None,
		};
		Some(right)
	}
}
