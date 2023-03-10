use crate::lexer::{Lex, Span, Token};

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

pub fn parse_statement(input: Lex) -> (Lex, ParseResult) {
	let token = match input.token() {
		Some(token) => token,
		None => return (input, ParseResult::None),
	};

	if let Token::Identifier = token {
		match input.text() {
			"print" => parse_print(input),
			"let" | "const" => parse_let(input),
			"for" => parse_for(input),
			"if" => parse_if(input),
			_ => parse_statement_expr(input),
		}
	} else {
		parse_statement_expr(input)
	}
}

fn parse_statement_expr(input: Lex) -> (Lex, ParseResult) {
	match parse_expression(input) {
		(input, expr) => match expr {
			ExprResult::Expr(expr) => assert_break(input, ParseResult::Ok(Statement::Expr(expr))),
			ExprResult::Error(span, error) => (input, ParseResult::Error(span, error)),
			ExprResult::None => match input {
				Lex::None(..) => (
					input,
					ParseResult::Error(
						input.span(),
						format!("expected expression, got end of input"),
					),
				),
				Lex::Some(lex) => {
					let (token, span) = lex.pair();
					(
						input,
						ParseResult::Error(span, format!("expected expression, got {token:?}")),
					)
				}
			},
		},
	}
}

fn parse_if(input: Lex) -> (Lex, ParseResult) {
	let input = input.next(); // skip `if`
	match parse_expression(input) {
		(input, expr) => match expr {
			ExprResult::Expr(expr) => {
				let (input, block) = parse_indented_block(input);
				let block = if let ParseResult::Ok(block) = block {
					ParseResult::Ok(Statement::If(expr, block.into()))
				} else {
					block
				};
				(input, block)
			}
			ExprResult::Error(span, error) => (input, ParseResult::Error(span, error)),
			ExprResult::None => (
				input,
				ParseResult::Error(input.span(), "expression expected after 'if'".into()),
			),
		},
	}
}

fn parse_for(input: Lex) -> (Lex, ParseResult) {
	let input = input.next(); // skip `for`
	let (input, id) = match input.token() {
		Some(Token::Identifier) => (input.next(), input.text().to_string()),
		_ => {
			return (
				input,
				ParseResult::Error(input.span(), "identifier expected after 'for'".into()),
			)
		}
	};

	let (input, ok) = input.skip_symbol("in");
	if !ok {
		return (
			input,
			ParseResult::Error(input.span(), "for 'in' expected".into()),
		);
	}

	let (input, from) = match parse_expression(input) {
		(input, expr) => match expr {
			ExprResult::Expr(expr) => (input, expr),
			ExprResult::Error(span, error) => return (input, ParseResult::Error(span, error)),
			ExprResult::None => {
				return (
					input,
					ParseResult::Error(input.span(), "expression expected after 'for in'".into()),
				)
			}
		},
	};

	let (input, ok) = input.skip_symbol("..");
	if !ok {
		return (
			input,
			ParseResult::Error(input.span(), "for '..' expected".into()),
		);
	}

	let (input, to) = match parse_expression(input) {
		(input, expr) => match expr {
			ExprResult::Expr(expr) => (input, expr),
			ExprResult::Error(span, error) => return (input, ParseResult::Error(span, error)),
			ExprResult::None => {
				return (
					input,
					ParseResult::Error(
						input.span(),
						"expression expected after 'for in ..'".into(),
					),
				)
			}
		},
	};

	let (input, block) = parse_indented_block(input);
	if let ParseResult::Ok(block) = block {
		(
			input,
			ParseResult::Ok(Statement::For(Id(id), from, to, block.into())),
		)
	} else {
		(input, block)
	}
}

fn parse_indented_block(input: Lex) -> (Lex, ParseResult) {
	let (input, ok) = input.skip_symbol(":");
	if !ok {
		return (
			input,
			ParseResult::Error(input.span(), "block ':' expected".into()),
		);
	}

	let (input, ok) = input.next_if(|token| matches!(token, Token::Break));
	if !ok {
		return (
			input,
			ParseResult::Error(input.span(), "end of line expected after ':'".into()),
		);
	}

	let (input, ok) = input.next_if(|token| matches!(token, Token::Indent));
	if !ok {
		return (
			input,
			ParseResult::Error(input.span(), "indented block expected".into()),
		);
	}

	let mut block = Vec::new();
	let mut input = input;
	loop {
		let ok;
		(input, ok) = input.next_if(|token| matches!(token, Token::Dedent));
		if ok {
			break;
		}

		let statement;
		(input, statement) = parse_statement(input);
		if let ParseResult::Ok(statement) = statement {
			block.push(statement);
		} else {
			return (input, statement);
		}
	}

	(input, ParseResult::Ok(Statement::Block(block)))
}

fn parse_print(input: Lex) -> (Lex, ParseResult) {
	let input = input.next(); // skip `print`
	let mut expr_list = Vec::new();
	let mut input = input;
	loop {
		let ok;
		(input, ok) = input.next_if(|token| matches!(token, Token::Break));
		if ok {
			let res = ParseResult::Ok(Statement::Print(expr_list));
			break (input, res);
		}

		if expr_list.len() > 0 {
			(input, _) = input.next_if(|token| matches!(token, Token::Symbol(",")));
		}

		let expr;
		(input, expr) = match parse_expression(input) {
			(input, expr) => match expr {
				ExprResult::Expr(expr) => (input, expr),
				ExprResult::Error(span, error) => break (input, ParseResult::Error(span, error)),
				ExprResult::None => {
					break (
						input,
						ParseResult::Error(input.span(), "expression expected".into()),
					);
				}
			},
		};
		expr_list.push(expr);
	}
}

fn parse_let(input: Lex) -> (Lex, ParseResult) {
	let input = input.next(); // skip `let`
	let (input, id) = match input.token() {
		Some(Token::Identifier) => (input.next(), input.text().to_string()),
		_ => {
			return (
				input,
				ParseResult::Error(input.span(), "identifier expected".into()),
			)
		}
	};

	let (input, ok) = input.skip_symbol("=");
	if !ok {
		return (
			input,
			ParseResult::Error(input.span(), "expected '='".into()),
		);
	}

	match parse_expression(input) {
		(input, expr) => match expr {
			ExprResult::Expr(expr) => {
				let res = ParseResult::Ok(Statement::Let(Id(id), expr));
				assert_break(input, res)
			}
			ExprResult::Error(span, error) => (input, ParseResult::Error(span, error)),
			ExprResult::None => (
				input,
				ParseResult::Error(input.span(), "expression expected".into()),
			),
		},
	}
}

fn assert_break(input: Lex, result: ParseResult) -> (Lex, ParseResult) {
	match input.token() {
		Some(Token::Break) | None => (input.next(), result),
		Some(_) => (
			input,
			ParseResult::Error(
				input.span(),
				format!("expected end of statement, got {input}"),
			),
		),
	}
}
