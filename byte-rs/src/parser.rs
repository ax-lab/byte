use crate::{
	input::{Span, Token, TokenStream},
	lexer::TokenKind,
};

pub enum Statement {
	Print(Vec<Expr>),
	Let(Id, Expr),
	Assign(Id, Expr),
}

pub enum ParseResult {
	Ok(Statement),
	Error(String),
	Invalid(Span, String),
	EndOfInput,
}

pub enum Expr {
	Integer(String),
	Literal(String),
	Var(Id),
	Neg(Box<Expr>),
	Sum(Box<Expr>, Box<Expr>),
	Sub(Box<Expr>, Box<Expr>),
	Mul(Box<Expr>, Box<Expr>),
	Div(Box<Expr>, Box<Expr>),
}

pub struct Id(pub String);

pub fn parse_statement<T: TokenStream>(input: &mut T, next: Token) -> (ParseResult, Token) {
	match next.kind {
		TokenKind::Identifier => match next.text.as_str() {
			"print" => {
				let next = input.next();
				parse_print(input, next)
			}
			"let" => {
				let next = input.next();
				parse_let(input, next)
			}
			_ => {
				let res = ParseResult::Invalid(next.span, "unexpected identifier".into());
				(res, input.next())
			}
		},
		TokenKind::Invalid => {
			let res = ParseResult::Invalid(next.span, "invalid token".into());
			(res, input.next())
		}
		TokenKind::Error(io_err) => {
			let res = ParseResult::Error(io_err);
			(res, input.next())
		}
		TokenKind::EndOfFile => (ParseResult::EndOfInput, input.next()),
		_ => {
			let res = ParseResult::Invalid(next.span, "unexpected token".into());
			(res, input.next())
		}
	}
}

fn parse_expr<T: TokenStream>(input: &mut T, next: Token) -> (Option<Expr>, Token) {
	return parse_expr_unary(input, next);
}

fn parse_expr_unary<T: TokenStream>(input: &mut T, next: Token) -> (Option<Expr>, Token) {
	match next.kind {
		TokenKind::Identifier => {
			let expr = Expr::Var(Id(next.text));
			(Some(expr), input.next())
		}

		TokenKind::Integer => {
			let expr = Expr::Integer(next.text);
			(Some(expr), input.next())
		}

		TokenKind::Symbol => {
			let is_minus = next.text == "-";
			if is_minus || next.text == "+" {
				let next = input.next();
				let (expr, next) = parse_expr_unary(input, next);
				if let Some(expr) = expr {
					let expr = if is_minus {
						Expr::Neg(expr.into())
					} else {
						expr
					};
					(Some(expr), next)
				} else {
					(None, next)
				}
			} else {
				(None, next)
			}
		}

		_ => (None, next),
	}
}

fn parse_print<T: TokenStream>(input: &mut T, mut next: Token) -> (ParseResult, Token) {
	let mut expr_list = Vec::new();
	loop {
		next = if expr_list.len() > 0 {
			if next.kind != TokenKind::Comma {
				let err = ParseResult::Invalid(next.span, "expected ','".into());
				return (err, next);
			} else {
				input.next()
			}
		} else {
			next
		};

		match next.kind {
			TokenKind::EndOfFile | TokenKind::LineBreak => {
				let res = ParseResult::Ok(Statement::Print(expr_list));
				break (res, input.next());
			}
			_ => {
				let expr;
				(expr, next) = parse_expr(input, next);
				if let Some(expr) = expr {
					expr_list.push(expr);
				} else {
					let err = ParseResult::Invalid(next.span, "expression expected".into());
					return (err, next);
				}
			}
		}
	}
}

fn parse_let<T: TokenStream>(input: &mut T, next: Token) -> (ParseResult, Token) {
	let id = next;
	if id.kind != TokenKind::Identifier {
		let err = ParseResult::Invalid(id.span, "identifier expected".into());
		return (err, id);
	}

	let next = input.next();
	let next = if next.kind != TokenKind::Symbol || next.text != "=" {
		let err = ParseResult::Invalid(next.span, "expected '='".into());
		return (err, next);
	} else {
		input.next()
	};

	let (expr, next) = parse_expr(input, next);
	if let Some(expr) = expr {
		let res = ParseResult::Ok(Statement::Let(Id(id.text), expr));
		parse_end(input, res, next)
	} else {
		let err = ParseResult::Invalid(next.span, "expression expected".into());
		(err, next)
	}
}

fn parse_end<T: TokenStream>(
	input: &mut T,
	result: ParseResult,
	next: Token,
) -> (ParseResult, Token) {
	match next.kind {
		TokenKind::EndOfFile | TokenKind::LineBreak => (result, input.next()),
		_ => {
			let res = ParseResult::Invalid(next.span, "expected end of statement".into());
			(res, next)
		}
	}
}
