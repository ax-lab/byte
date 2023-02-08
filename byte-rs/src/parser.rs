use crate::{
	input::{Span, Token, TokenStream},
	lexer::TokenKind,
};

#[derive(Debug)]
pub enum Statement {
	Print(Vec<Expr>),
	Let(Id, Expr),
}

pub enum ParseResult {
	Ok(Statement),
	Error(String),
	Invalid(Span, String),
	EndOfInput,
}

#[derive(Debug)]
pub enum Expr {
	Integer(String),
	Literal(String),
	Var(Id),
	Neg(Box<Expr>),
	Binary(BinaryOp, Box<Expr>, Box<Expr>),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BinaryOp {
	Add,
	Sub,
	Mul,
	Div,
}

impl std::fmt::Display for BinaryOp {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			BinaryOp::Add => write!(f, "Add"),
			BinaryOp::Sub => write!(f, "Sub"),
			BinaryOp::Mul => write!(f, "Mul"),
			BinaryOp::Div => write!(f, "Div"),
		}
	}
}

#[derive(Debug)]
pub struct Id(pub String);

pub fn parse_statement<T: TokenStream>(input: &mut T, mut next: Token) -> (ParseResult, Token) {
	while next.kind == TokenKind::LineBreak {
		next = input.next();
	}

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
	return parse_expr_add(input, next);
}

fn parse_expr_add<T: TokenStream>(input: &mut T, next: Token) -> (Option<Expr>, Token) {
	let (mut expr, mut next) = parse_expr_mul(input, next);
	loop {
		if let Some(left) = expr {
			(expr, next) = if next.kind == TokenKind::Symbol {
				let op = match next.text.as_str() {
					"+" => Some(BinaryOp::Add),
					"-" => Some(BinaryOp::Sub),
					_ => None,
				};
				if let Some(op) = op {
					let next = input.next();
					let (right, next) = parse_expr_mul(input, next);
					if let Some(right) = right {
						let expr = Expr::Binary(op, left.into(), right.into());
						(Some(expr), next)
					} else {
						return (None, next);
					}
				} else {
					return (Some(left), next);
				}
			} else {
				return (Some(left), next);
			}
		} else {
			return (expr, next);
		};
	}
}

fn parse_expr_mul<T: TokenStream>(input: &mut T, next: Token) -> (Option<Expr>, Token) {
	let (mut expr, mut next) = parse_expr_unary(input, next);
	loop {
		if let Some(left) = expr {
			(expr, next) = if next.kind == TokenKind::Symbol {
				let op = match next.text.as_str() {
					"*" => Some(BinaryOp::Mul),
					"/" => Some(BinaryOp::Div),
					_ => None,
				};
				if let Some(op) = op {
					let next = input.next();
					let (right, next) = parse_expr_unary(input, next);
					if let Some(right) = right {
						let expr = Expr::Binary(op, left.into(), right.into());
						(Some(expr), next)
					} else {
						return (None, next);
					}
				} else {
					return (Some(left), next);
				}
			} else {
				return (Some(left), next);
			}
		} else {
			return (expr, next);
		};
	}
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

		TokenKind::String => {
			let text = next.text;
			let text = text.strip_prefix("'").unwrap();
			let text = text.strip_suffix("'").unwrap();
			let expr = Expr::Literal(text.into());
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
		next = match next.kind {
			TokenKind::EndOfFile | TokenKind::LineBreak => {
				let res = ParseResult::Ok(Statement::Print(expr_list));
				break (res, input.next());
			}

			TokenKind::Comma if expr_list.len() > 0 => input.next(),

			_ => next,
		};

		let expr;
		(expr, next) = parse_expr(input, next);
		if let Some(expr) = expr {
			expr_list.push(expr);
		} else {
			let err = ParseResult::Invalid(next.span, "expression expected".into());
			break (err, next);
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
