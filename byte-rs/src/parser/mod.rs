use crate::lexer::{Input, Span, Token, TokenStream};

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

pub fn parse_statement<T: Input>(input: &mut TokenStream<T>) -> ParseResult {
	while input.get() == Token::LineBreak {
		input.shift();
	}

	let span = input.span();
	let result = match input.get() {
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
			_ => ParseResult::None,
		},
		Token::None => return ParseResult::None,

		other => ParseResult::Error(span, format!("unexpected token `{other:?}`")),
	};

	if let ParseResult::None = result {
		match parse_expression(input) {
			ExprResult::Expr(expr) => ParseResult::Ok(Statement::Expr(expr)),
			ExprResult::Error(span, error) => ParseResult::Error(span, error),
			ExprResult::None => ParseResult::Error(span, "unexpected identifier".into()),
		}
	} else {
		result
	}
}

fn parse_if<T: Input>(input: &mut TokenStream<T>) -> ParseResult {
	match parse_expression(input) {
		ExprResult::Expr(expr) => {
			let block = parse_block(input);
			if let ParseResult::Ok(block) = block {
				ParseResult::Ok(Statement::If(expr, block.into()))
			} else {
				block
			}
		}
		ExprResult::Error(span, error) => ParseResult::Error(span, error),
		ExprResult::None => {
			ParseResult::Error(input.span(), "expression expected after 'if'".into())
		}
	}
}

fn parse_for<T: Input>(input: &mut TokenStream<T>) -> ParseResult {
	if input.get() != Token::Identifier {
		return ParseResult::Error(input.span(), "identifier expected after 'for'".into());
	}

	let id = Id(input.text().into());
	input.shift();

	if input.get() != Token::Identifier || input.text() != "in" {
		return ParseResult::Error(input.span(), "for 'in' expected".into());
	}
	input.shift();

	let from = match parse_expression(input) {
		ExprResult::Expr(expr) => expr,
		ExprResult::Error(span, error) => return ParseResult::Error(span, error),
		ExprResult::None => {
			return ParseResult::Error(input.span(), "expression expected after 'for in'".into())
		}
	};

	if input.text() != ".." {
		return ParseResult::Error(input.span(), "for '..' expected".into());
	}
	input.shift();

	let to = match parse_expression(input) {
		ExprResult::Expr(expr) => expr,
		ExprResult::Error(span, error) => return ParseResult::Error(span, error),
		ExprResult::None => {
			return ParseResult::Error(input.span(), "expression expected after 'for in ..'".into())
		}
	};

	let block = parse_block(input);
	if let ParseResult::Ok(block) = block {
		ParseResult::Ok(Statement::For(id, from, to, block.into()))
	} else {
		block
	}
}

fn parse_block<T: Input>(input: &mut TokenStream<T>) -> ParseResult {
	if input.text() != ":" {
		return ParseResult::Error(input.span(), "block ':' expected".into());
	}
	input.shift();

	if input.get() != Token::LineBreak {
		return ParseResult::Error(input.span(), "end of line expected after ':'".into());
	}

	while input.get() == Token::LineBreak {
		input.shift();
	}

	if input.get() != Token::Ident {
		return ParseResult::Error(input.span(), "idented block expected".into());
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

fn parse_print<T: Input>(input: &mut TokenStream<T>) -> ParseResult {
	let mut expr_list = Vec::new();
	loop {
		match input.get() {
			Token::None | Token::LineBreak => {
				input.shift();
				let res = ParseResult::Ok(Statement::Print(expr_list));
				break res;
			}

			Token::Symbol(",") if expr_list.len() > 0 => input.shift(),

			_ => (),
		};

		let expr = match parse_expression(input) {
			ExprResult::Expr(expr) => expr,
			ExprResult::Error(span, error) => break ParseResult::Error(span, error),
			ExprResult::None => {
				break ParseResult::Error(input.span(), "expression expected".into())
			}
		};
		expr_list.push(expr);
	}
}

fn parse_let<T: Input>(input: &mut TokenStream<T>) -> ParseResult {
	if input.get() != Token::Identifier {
		return ParseResult::Error(input.span(), "identifier expected".into());
	}

	let id = Id(input.text().into());
	input.shift();

	if input.get() != Token::Symbol("=") {
		return ParseResult::Error(input.span(), "expected '='".into());
	}

	input.shift();
	match parse_expression(input) {
		ExprResult::Expr(expr) => {
			let res = ParseResult::Ok(Statement::Let(id, expr));
			parse_end(input, res)
		}
		ExprResult::Error(span, error) => ParseResult::Error(span, error),
		ExprResult::None => ParseResult::Error(input.span(), "expression expected".into()),
	}
}

fn parse_end<T: Input>(input: &mut TokenStream<T>, result: ParseResult) -> ParseResult {
	match input.get() {
		Token::None | Token::LineBreak => result,
		_ => ParseResult::Error(input.span(), "expected end of statement".into()),
	}
}
