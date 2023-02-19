use crate::token::{Reader, Span, Token, TokenStream};

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
}

pub enum ParseResult {
	Ok(Statement),
	Invalid(Span, String),
	EndOfInput,
}

pub fn parse_statement<T: Reader>(input: &mut TokenStream<T>) -> ParseResult {
	while input.get() == Token::LineBreak {
		input.shift();
	}

	let span = input.span();
	match input.get() {
		Token::Identifier => match input.text() {
			"print" => {
				input.shift();
				parse_print(input)
			}
			"let" | "const" => {
				input.shift();
				parse_let(input)
			}
			"for" => {
				input.shift();
				parse_for(input)
			}
			"if" => {
				input.shift();
				parse_if(input)
			}
			_ => ParseResult::Invalid(span, "unexpected identifier".into()),
		},
		Token::None => ParseResult::EndOfInput,

		other => ParseResult::Invalid(span, format!("unexpected token `{other:?}`")),
	}
}

fn parse_if<T: Reader>(input: &mut TokenStream<T>) -> ParseResult {
	match parse_expression(input) {
		ExprResult::Expr(expr) => {
			let block = parse_block(input);
			if let ParseResult::Ok(block) = block {
				ParseResult::Ok(Statement::If(expr, block.into()))
			} else {
				block
			}
		}
		ExprResult::Error(span, err) => ParseResult::Invalid(span, err),
		ExprResult::None => {
			ParseResult::Invalid(input.span(), "expression expected after 'if'".into())
		}
	}
}

fn parse_for<T: Reader>(input: &mut TokenStream<T>) -> ParseResult {
	if input.get() != Token::Identifier {
		return ParseResult::Invalid(input.span(), "identifier expected after 'for'".into());
	}

	let id = Id(input.text().into());
	input.shift();

	if input.get() != Token::Identifier || input.text() != "in" {
		return ParseResult::Invalid(input.span(), "for 'in' expected".into());
	}
	input.shift();

	let from = match parse_expression(input) {
		ExprResult::Expr(expr) => expr,
		ExprResult::Error(span, err) => return ParseResult::Invalid(span, err),
		ExprResult::None => {
			return ParseResult::Invalid(input.span(), "expression expected after 'for in'".into())
		}
	};

	if input.text() != ".." {
		return ParseResult::Invalid(input.span(), "for '..' expected".into());
	}
	input.shift();

	let to = match parse_expression(input) {
		ExprResult::Expr(expr) => expr,
		ExprResult::Error(span, err) => return ParseResult::Invalid(span, err),
		ExprResult::None => {
			return ParseResult::Invalid(
				input.span(),
				"expression expected after 'for in ..'".into(),
			)
		}
	};

	let block = parse_block(input);
	if let ParseResult::Ok(block) = block {
		ParseResult::Ok(Statement::For(id, from, to, block.into()))
	} else {
		block
	}
}

fn parse_block<T: Reader>(input: &mut TokenStream<T>) -> ParseResult {
	if input.text() != ":" {
		return ParseResult::Invalid(input.span(), "block ':' expected".into());
	}
	input.shift();

	if input.get() != Token::LineBreak {
		return ParseResult::Invalid(input.span(), "end of line expected after ':'".into());
	}

	while input.get() == Token::LineBreak {
		input.shift();
	}

	if input.get() != Token::Ident {
		return ParseResult::Invalid(input.span(), "idented block expected".into());
	}
	input.shift();

	let mut block = Vec::new();
	loop {
		while input.get() == Token::LineBreak {
			input.shift();
		}

		if input.get() == Token::Dedent {
			input.shift();
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

fn parse_print<T: Reader>(input: &mut TokenStream<T>) -> ParseResult {
	let mut expr_list = Vec::new();
	loop {
		match input.get() {
			Token::None | Token::LineBreak => {
				input.shift();
				let res = ParseResult::Ok(Statement::Print(expr_list));
				break res;
			}

			Token::Comma if expr_list.len() > 0 => input.shift(),

			_ => (),
		};

		let expr = match parse_expression(input) {
			ExprResult::Expr(expr) => expr,
			ExprResult::Error(span, err) => break ParseResult::Invalid(span, err),
			ExprResult::None => {
				break ParseResult::Invalid(input.span(), "expression expected".into())
			}
		};
		expr_list.push(expr);
	}
}

fn parse_let<T: Reader>(input: &mut TokenStream<T>) -> ParseResult {
	if input.get() != Token::Identifier {
		return ParseResult::Invalid(input.span(), "identifier expected".into());
	}

	let id = Id(input.text().into());
	input.shift();

	if input.get() != Token::Symbol || input.text() != "=" {
		return ParseResult::Invalid(input.span(), "expected '='".into());
	}

	input.shift();
	match parse_expression(input) {
		ExprResult::Expr(expr) => {
			let res = ParseResult::Ok(Statement::Let(id, expr));
			parse_end(input, res)
		}
		ExprResult::Error(span, err) => ParseResult::Invalid(span, err),
		ExprResult::None => ParseResult::Invalid(input.span(), "expression expected".into()),
	}
}

fn parse_end<T: Reader>(input: &mut TokenStream<T>, result: ParseResult) -> ParseResult {
	match input.get() {
		Token::None | Token::LineBreak => result,
		_ => ParseResult::Invalid(input.span(), "expected end of statement".into()),
	}
}
