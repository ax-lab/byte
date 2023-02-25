use crate::lexer::{Span, Token, TokenSource, TokenStream};

pub enum Block {
	None,
	Line {
		expr: Vec<(Token, Span)>,
		next: Option<Vec<Block>>,
	},
	Error(String, Span),
}

pub fn parse_block<T: TokenSource>(input: &mut TokenStream<T>) -> Block {
	do_parse_block(input, 0)
}

fn do_parse_block<T: TokenSource>(input: &mut TokenStream<T>, level: usize) -> Block {
	input.skip_while(|token| matches!(token, Token::LineBreak | Token::Comment));
	let mut tokens = Vec::new();
	while let Some((token, span)) = input.read(|_, token, span| match token {
		Token::LineBreak => None,
		_ => Some((token, span)),
	}) {
		match token {
			Token::Comment => continue,
			Token::Dedent => {
				if level == 0 {
					return Block::Error("invalid indentation".into(), span);
				} else {
					input.unget(token, span);
					break;
				}
			}
			token => tokens.push((token, span)),
		}
	}
	if tokens.len() > 0 {
		let result = Block::Line {
			expr: tokens,

			// read indented continuation
			next: if input.read_if(|token| matches!(token, Token::Indent)) {
				let mut next = Vec::new();
				loop {
					match do_parse_block(input, level + 1) {
						Block::None => break,
						error @ Block::Error(..) => return error,
						line => next.push(line),
					}
				}

				if let Some((error, span)) = input.expect_dedent() {
					return Block::Error(error, span);
				}

				if next.len() > 0 {
					Some(next)
				} else {
					None
				}
			} else {
				None
			},
		};

		result
	} else {
		Block::None
	}
}

impl std::fmt::Display for Block {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.do_output(0, f)
	}
}

impl Block {
	fn do_output(&self, level: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Block::None => write!(f, "None"),
			Block::Error(error, span) => write!(f, "Error({error} at {span}"),
			Block::Line { expr, next } => {
				write!(f, "Line(")?;
				for (i, it) in expr.iter().enumerate() {
					if i == 0 {
						write!(f, "\n\t")?;
						Self::indent(f, level)?;
					} else {
						write!(f, " ")?;
					}
					write!(f, "{}", it.0)?;
				}
				if let Some(next) = next {
					for it in next.iter() {
						write!(f, "\n\t")?;
						Self::indent(f, level)?;
						write!(f, "...")?;
						it.do_output(level + 1, f)?;
					}
				}
				write!(f, "\n")?;
				Self::indent(f, level)?;
				write!(f, ")")?;
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
