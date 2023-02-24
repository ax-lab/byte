use super::IsToken;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
	None,
	Comment,
	LineBreak,
	Ident,
	Dedent,
	Identifier(String),
	Integer(u64),
	Literal(String),
	Symbol(&'static str),
}

impl IsToken for Token {}

impl Token {
	pub fn symbol(&self) -> Option<&str> {
		match self {
			Token::Identifier(s) => Some(s.as_str()),
			Token::Symbol(s) => Some(s),
			_ => None,
		}
	}
}
