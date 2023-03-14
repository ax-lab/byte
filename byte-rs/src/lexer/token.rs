#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Token {
	None,
	Invalid,
	Break,
	Indent,
	Dedent,
	Identifier,
	Integer(u64),
	Literal(usize, usize),
	Symbol(&'static str),
}

impl Token {
	/// Returns the closing symbol for an opening parenthesis token.
	pub fn get_closing(&self) -> Option<&'static str> {
		let right = match self {
			Token::Symbol("(") => ")",
			_ => return None,
		};
		Some(right)
	}

	pub fn is_closing(&self) -> Option<&'static str> {
		let symbol = match self {
			Token::Symbol(sym @ ")") => sym,
			_ => return None,
		};
		Some(symbol)
	}

	pub fn is_none(&self) -> bool {
		matches!(self, Token::None)
	}
}
