use crate::lexer::{LexStream, Span, Token};

mod blocks;
pub use blocks::*;

#[allow(unused)]
mod operators;
pub use operators::*;

#[allow(unused)]
mod expr;
pub use expr::*;

#[derive(Debug)]
pub struct Id(pub String);

#[derive(Debug)]
pub enum Statement {
	Print(Vec<Expr>),
	Let(Id, Expr),
	If(Expr, Box<Statement>),
	For(Id, Expr, Expr, Box<Statement>),
	Block(Vec<Statement>),
	Expr(Expr),
}

pub enum ParseResult {
	Ok(Statement),
	Error(Span, String),
	None,
}

pub fn parse_statement(input: &mut LexStream) -> ParseResult {
	input
		.parse(|input, token, span| {
			let result = match token {
				Token::Identifier(id) => match id.as_str() {
					"print" => parse_print(input),
					"let" | "const" => parse_let(input),
					"for" => parse_for(input),
					"if" => parse_if(input),
					_ => {
						input.unget();
						ParseResult::None
					}
				},

				_ => {
					input.unget();
					ParseResult::None
				}
			};
			let result = if let ParseResult::None = result {
				match parse_expression(input) {
					ExprResult::Expr(expr) => {
						let (token, span) = input.read_pair().pair();
						if token != Token::Break {
							ParseResult::Error(
								span,
								format!("expected end of expression, got {token}"),
							)
						} else {
							ParseResult::Ok(Statement::Expr(expr))
						}
					}
					ExprResult::Error(span, error) => ParseResult::Error(span, error),
					ExprResult::None => {
						if input.at_end() {
							ParseResult::Error(
								span,
								format!("expected expression, got end of input"),
							)
						} else {
							let (token, span) = input.read_pair().pair();
							ParseResult::Error(span, format!("expected expression, got {token:?}"))
						}
					}
				}
			} else {
				result
			};
			Some(result)
		})
		.unwrap_or(ParseResult::None)
}

fn parse_if(input: &mut LexStream) -> ParseResult {
	match parse_expression(input) {
		ExprResult::Expr(expr) => {
			let block = parse_indented_block(input);
			if let ParseResult::Ok(block) = block {
				ParseResult::Ok(Statement::If(expr, block.into()))
			} else {
				block
			}
		}
		ExprResult::Error(span, error) => ParseResult::Error(span, error),
		ExprResult::None => {
			ParseResult::Error(input.next().span(), "expression expected after 'if'".into())
		}
	}
}

fn parse_for(input: &mut LexStream) -> ParseResult {
	let id = if let Some(id) = input.parse(|_, token, _| match token {
		Token::Identifier(id) => Some(id),
		_ => None,
	}) {
		id
	} else {
		return ParseResult::Error(
			input.next().span(),
			"identifier expected after 'for'".into(),
		);
	};

	if !input.read_symbol("in") {
		return ParseResult::Error(input.next().span(), "for 'in' expected".into());
	}

	let from = match parse_expression(input) {
		ExprResult::Expr(expr) => expr,
		ExprResult::Error(span, error) => return ParseResult::Error(span, error),
		ExprResult::None => {
			return ParseResult::Error(
				input.next().span(),
				"expression expected after 'for in'".into(),
			)
		}
	};

	if !input.read_symbol("..") {
		return ParseResult::Error(input.next().span(), "for '..' expected".into());
	}

	let to = match parse_expression(input) {
		ExprResult::Expr(expr) => expr,
		ExprResult::Error(span, error) => return ParseResult::Error(span, error),
		ExprResult::None => {
			return ParseResult::Error(
				input.next().span(),
				"expression expected after 'for in ..'".into(),
			)
		}
	};

	let block = parse_indented_block(input);
	if let ParseResult::Ok(block) = block {
		ParseResult::Ok(Statement::For(Id(id), from, to, block.into()))
	} else {
		block
	}
}

fn parse_indented_block(input: &mut LexStream) -> ParseResult {
	if !input.read_symbol(":") {
		return ParseResult::Error(input.next().span(), "block ':' expected".into());
	}

	if !input.read_if(|token| matches!(token, Token::Break)) {
		return ParseResult::Error(input.next().span(), "end of line expected after ':'".into());
	}

	if !input.read_if(|token| matches!(token, Token::Indent)) {
		return ParseResult::Error(input.next().span(), "indented block expected".into());
	}

	let mut block = Vec::new();
	loop {
		if input.read_if(|token| matches!(token, Token::Dedent)) {
			break;
		}

		let statement = parse_statement(input);
		if let ParseResult::Ok(statement) = statement {
			block.push(statement);
		} else {
			return statement;
		}
	}

	ParseResult::Ok(Statement::Block(block))
}

fn parse_print(input: &mut LexStream) -> ParseResult {
	let mut expr_list = Vec::new();
	loop {
		if input.read_if(|token| matches!(token, Token::Break)) {
			let res = ParseResult::Ok(Statement::Print(expr_list));
			break res;
		}

		if expr_list.len() > 0 {
			input.read_if(|token| matches!(token, Token::Symbol(",")));
		}

		let expr = match parse_expression(input) {
			ExprResult::Expr(expr) => expr,
			ExprResult::Error(span, error) => break ParseResult::Error(span, error),
			ExprResult::None => {
				break ParseResult::Error(input.next().span(), "expression expected".into())
			}
		};
		expr_list.push(expr);
	}
}

fn parse_let(input: &mut LexStream) -> ParseResult {
	let id = if let Some(id) = input.parse(|_, token, _| match token {
		Token::Identifier(id) => Some(id),
		_ => None,
	}) {
		Id(id)
	} else {
		return ParseResult::Error(input.next().span(), "identifier expected".into());
	};

	if !input.read_symbol("=") {
		return ParseResult::Error(input.next().span(), "expected '='".into());
	}

	match parse_expression(input) {
		ExprResult::Expr(expr) => {
			let res = ParseResult::Ok(Statement::Let(id, expr));
			parse_end(input, res)
		}
		ExprResult::Error(span, error) => ParseResult::Error(span, error),
		ExprResult::None => ParseResult::Error(input.next().span(), "expression expected".into()),
	}
}

fn parse_end(input: &mut LexStream, result: ParseResult) -> ParseResult {
	input.map_next(|token, span| match token {
		Token::Break => result,
		_ => ParseResult::Error(span, "expected end of statement".into()),
	})
}
