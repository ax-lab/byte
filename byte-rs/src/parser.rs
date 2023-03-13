use crate::lexer::{Context, Span, Token};

mod blocks;
pub use blocks::*;

#[allow(unused)]
mod operators;
pub use operators::*;

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

pub enum ParseResult<'a> {
	Ok(Statement),
	Error(Span<'a>, String),
	None,
}

pub fn parse_statement<'a>(input: &mut Context<'a>) -> ParseResult<'a> {
	match input.token() {
		Token::None => ParseResult::None,
		Token::Identifier => match input.value().text() {
			"print" => parse_print(input),
			"let" | "const" => parse_let(input),
			"for" => parse_for(input),
			"if" => parse_if(input),
			_ => parse_statement_expr(input),
		},
		_ => parse_statement_expr(input),
	}
}

fn parse_statement_expr<'a>(input: &mut Context<'a>) -> ParseResult<'a> {
	match parse_expression(input) {
		ExprResult::Expr(expr) => assert_break(input, ParseResult::Ok(Statement::Expr(expr))),
		ExprResult::Error(span, error) => ParseResult::Error(span, error),
		ExprResult::None => match input.token() {
			Token::None => ParseResult::Error(
				input.span(),
				format!("expected expression, got end of input"),
			),
			token => {
				ParseResult::Error(input.span(), format!("expected expression, got {token:?}"))
			}
		},
	}
}

fn parse_if<'a>(input: &mut Context<'a>) -> ParseResult<'a> {
	input.next(); // skip `if`
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
			ParseResult::Error(input.span(), "expression expected after 'if'".into())
		}
	}
}

fn parse_for<'a>(input: &mut Context<'a>) -> ParseResult<'a> {
	input.next(); // skip `for`
	let id = match input.token() {
		Token::Identifier => {
			let id = input.value().text().to_string();
			input.next();
			id
		}
		_ => return ParseResult::Error(input.span(), "identifier expected after 'for'".into()),
	};

	if !input.skip_symbol("in") {
		return ParseResult::Error(input.span(), "for 'in' expected".into());
	}

	let from = match parse_expression(input) {
		ExprResult::Expr(expr) => expr,
		ExprResult::Error(span, error) => return ParseResult::Error(span, error),
		ExprResult::None => {
			return ParseResult::Error(input.span(), "expression expected after 'for in'".into());
		}
	};

	if !input.skip_symbol("..") {
		return ParseResult::Error(input.span(), "for '..' expected".into());
	}

	let to = match parse_expression(input) {
		ExprResult::Expr(expr) => expr,
		ExprResult::Error(span, error) => return ParseResult::Error(span, error),
		ExprResult::None => {
			return ParseResult::Error(
				input.span(),
				"expression expected after 'for in ..'".into(),
			);
		}
	};

	let block = parse_indented_block(input);
	if let ParseResult::Ok(block) = block {
		ParseResult::Ok(Statement::For(Id(id), from, to, block.into()))
	} else {
		block
	}
}

fn parse_indented_block<'a>(input: &mut Context<'a>) -> ParseResult<'a> {
	if !input.skip_symbol(":") {
		return ParseResult::Error(input.span(), "block ':' expected".into());
	}

	if !input.next_if(|value| matches!(value.token, Token::Break | Token::None)) {
		return ParseResult::Error(input.span(), "end of line expected after ':'".into());
	}

	if !input.next_if(|value| matches!(value.token, Token::Indent)) {
		return ParseResult::Error(input.span(), "indented block expected".into());
	}

	let mut block = Vec::new();
	while !input.next_if(|value| matches!(value.token, Token::Dedent)) {
		let statement = parse_statement(input);
		if let ParseResult::Ok(statement) = statement {
			block.push(statement);
		} else {
			return statement;
		}
	}

	ParseResult::Ok(Statement::Block(block))
}

fn parse_print<'a>(input: &mut Context<'a>) -> ParseResult<'a> {
	input.next(); // skip `print`
	let mut expr_list = Vec::new();
	loop {
		if input.next_if(|x| matches!(x.token, Token::Break | Token::None)) {
			let res = ParseResult::Ok(Statement::Print(expr_list));
			break res;
		}

		if expr_list.len() > 0 {
			input.next_if(|x| matches!(x.token, Token::Symbol(",")));
		}

		let expr = match parse_expression(input) {
			ExprResult::Expr(expr) => expr,
			ExprResult::Error(span, error) => break ParseResult::Error(span, error),
			ExprResult::None => {
				break ParseResult::Error(input.span(), "expression expected in print".into());
			}
		};
		expr_list.push(expr);
	}
}

fn parse_let<'a>(input: &mut Context<'a>) -> ParseResult<'a> {
	input.next(); // skip `let`
	let id = match input.token() {
		Token::Identifier => {
			let id = input.value().text().to_string();
			input.next();
			id
		}
		_ => return ParseResult::Error(input.span(), "identifier expected".into()),
	};

	if !input.skip_symbol("=") {
		return ParseResult::Error(input.span(), "expected '='".into());
	}

	match parse_expression(input) {
		ExprResult::Expr(expr) => {
			let res = ParseResult::Ok(Statement::Let(Id(id), expr));
			assert_break(input, res)
		}
		ExprResult::Error(span, error) => ParseResult::Error(span, error),
		ExprResult::None => ParseResult::Error(input.span(), "expression expected in let".into()),
	}
}

fn assert_break<'a>(input: &mut Context<'a>, result: ParseResult<'a>) -> ParseResult<'a> {
	match input.token() {
		Token::Break | Token::None => {
			input.next();
			result
		}
		_ => ParseResult::Error(
			input.span(),
			format!("expected end of statement, got {}", input.value()),
		),
	}
}
