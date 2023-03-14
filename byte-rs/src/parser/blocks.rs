use crate::lexer::{Context, Lex, Span, Token};

/// Display a list of blocks in the input TokenStream. This is used only
/// for testing the tokenization.
pub fn list_blocks<'a>(input: &mut Context<'a>) {
	loop {
		match parse_block(input) {
			Block::None => break,
			Block::Error(error, span) => {
				println!("Error: {error} at {span}");
				std::process::exit(1);
			}
			block => {
				println!("{block}");
			}
		}
	}
}

enum Block<'a> {
	None,
	Item(Lex<'a>),
	Line {
		expr: Vec<Block<'a>>,
		next: Option<Vec<Block<'a>>>,
	},
	Parenthesis(Lex<'a>, Vec<Block<'a>>, Lex<'a>),
	Error(String, Span<'a>),
}

fn parse_block<'a>(input: &mut Context<'a>) -> Block<'a> {
	parse_line(input, 0, None)
}

fn parse_line<'a>(input: &mut Context<'a>, level: usize, stop: Option<&'static str>) -> Block<'a> {
	let expr = parse_expr(input, level, stop);
	if expr.len() == 0 {
		Block::None
	} else {
		input.next_if(|value| value.token == Token::Break);
		// read indented continuation
		let next = if input.next_if(|value| matches!(value.token, Token::Indent)) {
			let mut next = Vec::new();
			loop {
				match parse_line(input, level, stop) {
					Block::None => break,
					error @ Block::Error(..) => return error,
					line => next.push(line),
				}
			}

			if !input.next_if(|value| matches!(value.token, Token::Dedent)) {
				return Block::Error(
					format!("dedent expected, got {}", input.value()),
					input.span(),
				);
			}

			if next.len() > 0 {
				Some(next)
			} else {
				None
			}
		} else {
			None
		};
		Block::Line { expr, next }
	}
}

fn parse_expr<'a>(
	input: &mut Context<'a>,
	_level: usize,
	stop: Option<&'static str>,
) -> Vec<Block<'a>> {
	let mut expr = Vec::new();
	loop {
		match input.token() {
			Token::Indent => {
				panic!("unexpected {} at {}", input.value(), input.span());
			}
			Token::Break | Token::Dedent => {
				break expr;
			}
			Token::Symbol(sym) if Some(sym) == stop => {
				break expr;
			}
			Token::Symbol("(") => {
				let left = input.value();
				input.next();
				let item = parse_parenthesis(input, left, ")");
				expr.push(item);
			}
			Token::None => {
				break expr;
			}
			_ => {
				expr.push(Block::Item(input.value()));
				input.next();
			}
		}
	}
}

fn parse_parenthesis<'a>(input: &mut Context<'a>, left: Lex<'a>, right: &'static str) -> Block<'a> {
	let level = 0;
	input.next_if(|x| x.token == Token::Break);
	let indented = input.next_if(|x| x.token == Token::Indent);

	let mut inner = Vec::new();
	if indented {
		loop {
			input.next_if(|x| x.token == Token::Break);

			if input.next_if(|x| x.token == Token::Dedent) {
				break;
			}

			match parse_line(input, level, Some(right)) {
				Block::None => {
					return Block::Error(
						format!(
							"unexpected `{}` in indented {} parenthesis",
							input.value(),
							left
						),
						input.span(),
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

	let lex_closing = input.value();
	if !(right != "" && input.skip_symbol(right)) {
		let at = left.span;
		Block::Error(
			format!(
				"expected closing `{right}` for {left} at {at}, but got {}",
				input.value()
			),
			input.span(),
		)
	} else {
		Block::Parenthesis(left, inner, lex_closing)
	}
}

impl<'a> std::fmt::Display for Block<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.do_output(0, f)
	}
}

impl<'a> Block<'a> {
	fn do_output(&self, level: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Block::None => write!(f, "None"),
			Block::Error(error, span) => write!(f, "Error({error} at {span}"),
			Block::Item(lex) => write!(f, "{lex}"),
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
