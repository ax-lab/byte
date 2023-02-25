use crate::lexer::{ReadToken, Span, Token, TokenSource, TokenStream};

pub enum Block {
	None,
	Expr(Vec<(Token, Span)>),
	#[allow(dead_code)]
	Error(String, Span),
}

pub fn parse_block<T: TokenSource>(input: &mut TokenStream<T>) -> Block {
	input.skip_while(|token| matches!(token, Token::LineBreak | Token::Comment));
	let mut tokens = Vec::new();
	while let Some(next) = input.try_read(|_, token, span| match token {
		Token::LineBreak => ReadToken::Unget(token),
		_ => ReadToken::MapTo((token, span)),
	}) {
		tokens.push(next);
	}
	if tokens.len() > 0 {
		Block::Expr(tokens)
	} else {
		Block::None
	}
}

pub struct BlockFormatter<'a, T: TokenSource>(pub &'a TokenStream<'a, T>, pub &'a Block);

impl<'a, T: TokenSource> std::fmt::Display for BlockFormatter<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.1.do_output(self.0, 0, f)
	}
}

impl Block {
	fn do_output<T: TokenSource>(
		&self,
		input: &TokenStream<T>,
		level: usize,
		f: &mut std::fmt::Formatter<'_>,
	) -> std::fmt::Result {
		Self::indent(f, level)?;
		match self {
			Block::None => write!(f, "None"),
			Block::Error(error, span) => write!(f, "Error({error} at {span}"),
			Block::Expr(tokens) => {
				write!(f, "Expr(")?;
				for it in tokens.iter() {
					write!(f, "\n")?;
					Self::indent(f, level)?;
					write!(f, "\t{}", input.read_text(it.1))?;
				}
				write!(f, "\n)")?;
				Ok(())
			}
		}
	}

	fn indent(f: &mut std::fmt::Formatter<'_>, level: usize) -> std::fmt::Result {
		for _ in 0..level {
			write!(f, "\t")?;
		}
		Ok(())
	}
}
