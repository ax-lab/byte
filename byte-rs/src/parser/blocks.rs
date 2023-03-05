use crate::lexer::{ReadToken, Span, Token, TokenSource, TokenStream};

pub enum Block {
	None,
	Item(Token, Span),
	Line {
		expr: Vec<Block>,
		next: Option<Vec<Block>>,
	},
	Parenthesis(Token, Vec<Block>, Token),
	Error(String, Span),
}

pub fn parse_block<T: TokenSource>(input: &mut TokenStream<T>) -> Block {
	parse_line(input, 0, None)
}

fn parse_line<T: TokenSource>(
	input: &mut TokenStream<T>,
	level: usize,
	stop: Option<&'static str>,
) -> Block {
	input.skip_blank_lines();
	let expr = parse_expr(input, level, stop);
	if expr.len() == 0 {
		Block::None
	} else {
		input.skip_blank_lines();
		let result = Block::Line {
			expr,

			// read indented continuation
			next: if input.read_if(|token| matches!(token, Token::Indent)) {
				let mut next = Vec::new();
				loop {
					match parse_line(input, level, stop) {
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
	}
}

fn parse_expr<T: TokenSource>(
	input: &mut TokenStream<T>,
	level: usize,
	stop: Option<&'static str>,
) -> Vec<Block> {
	let mut expr = Vec::new();
	let mut stopped = false;
	while let Some((token, span)) = input.try_read(|_, token, span| {
		stopped = if let (Token::Symbol(symbol), Some(stop)) = (&token, stop) {
			*symbol == stop
		} else {
			false
		};
		if stopped {
			ReadToken::Unget(token)
		} else {
			match token {
				Token::LineBreak | Token::Dedent => ReadToken::Unget(token),
				_ => ReadToken::MapTo((token, span)),
			}
		}
	}) {
		match token {
			Token::Comment => continue,
			Token::Indent => {
				panic!("unexpected {token} at {span}");
			}
			Token::Symbol("(") => {
				let item = parse_parenthesis(input, (token, span), ")");
				expr.push(item);
			}
			token => expr.push(Block::Item(token, span)),
		}
	}
	if stopped {
		input.inner_mut().pop_indent(level);
	}
	expr
}

fn parse_parenthesis<T: TokenSource>(
	input: &mut TokenStream<T>,
	left: (Token, Span),
	right: &'static str,
) -> Block {
	let level = input.inner().indent_level();
	input.skip_blank_lines();
	let indented = input.read_if(|next| next == &Token::Indent);

	let mut inner = Vec::new();
	if indented {
		loop {
			input.skip_blank_lines();
			if input.read_if(|token| token == &Token::Dedent) {
				break;
			}
			let block = parse_line(input, level, Some(right));
			match block {
				Block::None => {
					let (token, span) = input.next_token();
					return Block::Error(
						format!("unexpected {token} in indented parenthesis"),
						span,
					);
				}
				error @ Block::Error(..) => return error,
				block => inner.push(block),
			}
		}
	} else {
		let block = parse_line(input, level, Some(right));
		match block {
			Block::None => {}
			error @ Block::Error(..) => return error,
			block => inner.push(block),
		}
	}

	if !input.read_if(|next| next.symbol() == Some(right)) {
		let (next, span) = input.next_token();
		let (left, at) = left;
		Block::Error(
			format!("expected closing `{right}` for `{left}` at {at}, but got {next}"),
			span,
		)
	} else {
		Block::Parenthesis(left.0, inner, Token::Symbol(right))
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
			Block::Item(token, _) => write!(f, "{token}"),
			Block::Parenthesis(left, inner, right) => {
				write!(f, "P{left}")?;
				if inner.len() > 0 {
					for it in inner.iter() {
						write!(f, "\n\t")?;
						Self::indent(f, level)?;
						it.do_output(level + 1, f)?;
					}
					write!(f, "\n")?;
					Self::indent(f, level)?;
				}
				write!(f, "{right}")
			}
			Block::Line { expr, next } => {
				write!(f, "Line(")?;
				for (i, it) in expr.iter().enumerate() {
					if i == 0 {
						write!(f, "\n\t")?;
						Self::indent(f, level)?;
					} else {
						write!(f, " ")?;
					}
					it.do_output(level + 1, f)?;
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