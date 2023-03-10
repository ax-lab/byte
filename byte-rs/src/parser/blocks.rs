use crate::{
	input::Span,
	lexer::{Lex, Token},
};

/// Display a list of blocks in the input TokenStream. This is used only
/// for testing the tokenization.
pub fn list_blocks(mut input: Lex) {
	loop {
		let block;
		(input, block) = parse_block(input);
		match block {
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
	Error(String, Span),
}

fn parse_block(input: Lex) -> (Lex, Block) {
	parse_line(input, 0, None)
}

fn parse_line<'a>(
	input: Lex<'a>,
	level: usize,
	stop: Option<&'static str>,
) -> (Lex<'a>, Block<'a>) {
	let (input, expr) = parse_expr(input, level, stop);
	if expr.len() == 0 {
		(input, Block::None)
	} else {
		let (input, _) = input.next_if(|x| x == Token::Break);
		let (input, cont) = input.next_if(|token| matches!(token, Token::Indent));

		// read indented continuation
		let (input, next) = if cont {
			let mut next = Vec::new();
			let mut input = input;
			loop {
				let line;
				(input, line) = parse_line(input, level, stop);
				match line {
					Block::None => break,
					error @ Block::Error(..) => return (input, error),
					line => next.push(line),
				}
			}

			let (input, ok) = input.next_if(|x| matches!(x, Token::Dedent));
			if !ok {
				return (
					input,
					Block::Error(format!("dedent expected, got {input}"), input.span()),
				);
			}

			let next = if next.len() > 0 { Some(next) } else { None };
			(input, next)
		} else {
			(input, None)
		};
		let result = Block::Line { expr, next };
		(input, result)
	}
}

fn parse_expr<'a>(
	input: Lex<'a>,
	_level: usize,
	stop: Option<&'static str>,
) -> (Lex<'a>, Vec<Block<'a>>) {
	let mut expr = Vec::new();
	let mut input = input;
	let input = loop {
		match input.token() {
			Some(Token::Indent) => {
				let span = input.span();
				panic!("unexpected {input} at {span}");
			}
			Some(Token::Break | Token::Dedent) => {
				break input;
			}
			Some(Token::Symbol(sym)) if Some(sym) == stop => {
				break input;
			}
			Some(Token::Symbol("(")) => {
				let left = input;
				let item;
				input = input.next();
				(input, item) = parse_parenthesis(input, left, ")");
				expr.push(item);
			}
			Some(_) => {
				expr.push(Block::Item(input));
				input = input.next();
			}
			None => {
				break input;
			}
		}
	};
	(input, expr)
}

fn parse_parenthesis<'a>(
	input: Lex<'a>,
	left: Lex<'a>,
	right: &'static str,
) -> (Lex<'a>, Block<'a>) {
	let level = 0;
	let (input, _) = input.next_if(|x| x == Token::Break);
	let (input, indented) = input.next_if(|next| next == Token::Indent);

	let mut inner = Vec::new();
	let mut input = input;
	if indented {
		loop {
			(input, _) = input.next_if(|x| x == Token::Break);

			let dedent;
			(input, dedent) = input.next_if(|token| token == Token::Dedent);
			if dedent {
				break;
			}

			let block;
			(input, block) = parse_line(input, level, Some(right));
			match block {
				Block::None => {
					return (
						input,
						Block::Error(
							format!("unexpected `{input}` in indented {} parenthesis", left),
							input.span(),
						),
					);
				}
				error @ Block::Error(..) => return (input, error),
				block => inner.push(block),
			}
		}
	} else {
		let block;
		(input, block) = parse_line(input, level, Some(right));
		match block {
			Block::None => {}
			error @ Block::Error(..) => return (input, error),
			block => inner.push(block),
		}
	}

	let lex_closing = input;
	let (input, ok) = if right != "" {
		input.skip_symbol(right)
	} else {
		(input, false)
	};
	if !ok {
		let at = left.span();
		let error = Block::Error(
			format!("expected closing `{right}` for {left} at {at}, but got {input}"),
			input.span(),
		);
		(input, error)
	} else {
		(input, Block::Parenthesis(left, inner, lex_closing))
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
