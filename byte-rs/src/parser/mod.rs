use crate::lexer::{Span, Token, TokenSource, TokenStream};

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

pub fn parse_statement<T: TokenSource>(input: &mut TokenStream<T>) -> ParseResult {
	input.skip_while(|token| matches!(token, Token::LineBreak));
	input
		.read(|input, token, span| {
			let result = match token {
				Token::Identifier(id) => match id.as_str() {
					"print" => parse_print(input),
					"let" | "const" => parse_let(input),
					"for" => parse_for(input),
					"if" => parse_if(input),
					_ => {
						let token = Token::Identifier(id);
						input.unget(token, span);
						ParseResult::None
					}
				},

				token => {
					input.unget(token, span);
					ParseResult::None
				}
			};
			let result = if let ParseResult::None = result {
				match parse_expression(input) {
					ExprResult::Expr(expr) => ParseResult::Ok(Statement::Expr(expr)),
					ExprResult::Error(span, error) => ParseResult::Error(span, error),
					ExprResult::None => {
						let token = input.next_token();
						ParseResult::Error(span, format!("expected expression, got {token:?}"))
					}
				}
			} else {
				result
			};
			Some(result)
		})
		.unwrap_or(ParseResult::None)
}

fn parse_if<T: TokenSource>(input: &mut TokenStream<T>) -> ParseResult {
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
			ParseResult::Error(input.next_span(), "expression expected after 'if'".into())
		}
	}
}

fn parse_for<T: TokenSource>(input: &mut TokenStream<T>) -> ParseResult {
	let id = if let Some(id) = input.read(|_, token, _| match token {
		Token::Identifier(id) => Some(id),
		_ => None,
	}) {
		id
	} else {
		return ParseResult::Error(input.next_span(), "identifier expected after 'for'".into());
	};

	if !input.read_symbol("in") {
		return ParseResult::Error(input.next_span(), "for 'in' expected".into());
	}

	let from = match parse_expression(input) {
		ExprResult::Expr(expr) => expr,
		ExprResult::Error(span, error) => return ParseResult::Error(span, error),
		ExprResult::None => {
			return ParseResult::Error(
				input.next_span(),
				"expression expected after 'for in'".into(),
			)
		}
	};

	if !input.read_symbol("..") {
		return ParseResult::Error(input.next_span(), "for '..' expected".into());
	}

	let to = match parse_expression(input) {
		ExprResult::Expr(expr) => expr,
		ExprResult::Error(span, error) => return ParseResult::Error(span, error),
		ExprResult::None => {
			return ParseResult::Error(
				input.next_span(),
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

fn parse_indented_block<T: TokenSource>(input: &mut TokenStream<T>) -> ParseResult {
	if !input.read_symbol(":") {
		return ParseResult::Error(input.next_span(), "block ':' expected".into());
	}

	if !input.read_if(|token| matches!(token, Token::LineBreak)) {
		return ParseResult::Error(input.next_span(), "end of line expected after ':'".into());
	}

	input.skip_while(|token| matches!(token, Token::LineBreak));

	if !input.read_if(|token| matches!(token, Token::Ident)) {
		return ParseResult::Error(input.next_span(), "idented block expected".into());
	}

	let mut block = Vec::new();
	loop {
		input.skip_while(|token| matches!(token, Token::LineBreak));

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

fn parse_print<T: TokenSource>(input: &mut TokenStream<T>) -> ParseResult {
	let mut expr_list = Vec::new();
	loop {
		if input.read_if(|token| matches!(token, Token::None | Token::LineBreak)) {
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
				break ParseResult::Error(input.next_span(), "expression expected".into())
			}
		};
		expr_list.push(expr);
	}
}

fn parse_let<T: TokenSource>(input: &mut TokenStream<T>) -> ParseResult {
	let id = if let Some(id) = input.read(|_, token, _| match token {
		Token::Identifier(id) => Some(id),
		_ => None,
	}) {
		Id(id)
	} else {
		return ParseResult::Error(input.next_span(), "identifier expected".into());
	};

	if !input.read_symbol("=") {
		return ParseResult::Error(input.next_span(), "expected '='".into());
	}

	match parse_expression(input) {
		ExprResult::Expr(expr) => {
			let res = ParseResult::Ok(Statement::Let(id, expr));
			parse_end(input, res)
		}
		ExprResult::Error(span, error) => ParseResult::Error(span, error),
		ExprResult::None => ParseResult::Error(input.next_span(), "expression expected".into()),
	}
}

fn parse_end<T: TokenSource>(input: &mut TokenStream<T>, result: ParseResult) -> ParseResult {
	input.map_next(|token, span| match token {
		Token::None | Token::LineBreak => result,
		_ => ParseResult::Error(span, "expected end of statement".into()),
	})
}
