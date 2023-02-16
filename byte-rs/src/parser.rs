use crate::token::{Reader, Span, Token, TokenStream};

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

#[derive(Debug)]
pub enum Expr {
	Integer(String),
	Literal(String),
	Var(Id),
	Neg(Box<Expr>),
	Binary(BinaryOp, Box<Expr>, Box<Expr>),
	Equality(Box<Expr>, Box<Expr>),
	TernaryConditional(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BinaryOp {
	Add,
	Sub,
	Mul,
	Div,
	Mod,
}

impl std::fmt::Display for BinaryOp {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			BinaryOp::Add => write!(f, "Add"),
			BinaryOp::Sub => write!(f, "Sub"),
			BinaryOp::Mul => write!(f, "Mul"),
			BinaryOp::Div => write!(f, "Div"),
			BinaryOp::Mod => write!(f, "Mod"),
		}
	}
}

#[derive(Debug)]
pub struct Id(pub String);

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
			"let" => {
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
	if let Some(expr) = parse_expr(input) {
		let block = parse_block(input);
		if let ParseResult::Ok(block) = block {
			ParseResult::Ok(Statement::If(expr, block.into()))
		} else {
			block
		}
	} else {
		ParseResult::Invalid(input.span(), "expression expected after 'if'".into())
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

	let from = if let Some(expr) = parse_expr(input) {
		expr
	} else {
		return ParseResult::Invalid(input.span(), "expression expected after 'for in'".into());
	};

	if input.text() != ".." {
		return ParseResult::Invalid(input.span(), "for '..' expected".into());
	}
	input.shift();

	let to = if let Some(expr) = parse_expr(input) {
		expr
	} else {
		return ParseResult::Invalid(input.span(), "expression expected after 'for in ..'".into());
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

fn parse_expr<T: Reader>(input: &mut TokenStream<T>) -> Option<Expr> {
	return parse_expr_cond(input);
}

fn parse_expr_cond<T: Reader>(input: &mut TokenStream<T>) -> Option<Expr> {
	if let Some(cond) = parse_expr_comparison(input) {
		if input.text() == "?" {
			input.shift();
			if let Some(left) = parse_expr_cond(input) {
				if input.text() != ":" {
					return None;
				} else {
					input.shift();
					if let Some(right) = parse_expr_cond(input) {
						Some(Expr::TernaryConditional(
							cond.into(),
							left.into(),
							right.into(),
						))
					} else {
						None
					}
				}
			} else {
				None
			}
		} else {
			Some(cond)
		}
	} else {
		None
	}
}

fn parse_expr_comparison<T: Reader>(input: &mut TokenStream<T>) -> Option<Expr> {
	if let Some(left) = parse_expr_add(input) {
		if input.text() == "==" {
			input.shift();
			if let Some(right) = parse_expr_add(input) {
				Some(Expr::Equality(left.into(), right.into()))
			} else {
				None
			}
		} else {
			Some(left)
		}
	} else {
		None
	}
}

fn parse_expr_add<T: Reader>(input: &mut TokenStream<T>) -> Option<Expr> {
	let mut expr = parse_expr_mul(input);
	loop {
		if let Some(left) = expr {
			expr = if input.get() == Token::Symbol {
				let op = match input.text() {
					"+" => Some(BinaryOp::Add),
					"-" => Some(BinaryOp::Sub),
					_ => None,
				};
				if let Some(op) = op {
					input.shift();
					let right = parse_expr_mul(input);
					if let Some(right) = right {
						let expr = Expr::Binary(op, left.into(), right.into());
						Some(expr)
					} else {
						return None;
					}
				} else {
					return Some(left);
				}
			} else {
				return Some(left);
			}
		} else {
			return expr;
		};
	}
}

fn parse_expr_mul<T: Reader>(input: &mut TokenStream<T>) -> Option<Expr> {
	let mut expr = parse_expr_unary(input);
	loop {
		if let Some(left) = expr {
			expr = if input.get() == Token::Symbol {
				let op = match input.text() {
					"*" => Some(BinaryOp::Mul),
					"/" => Some(BinaryOp::Div),
					"%" => Some(BinaryOp::Mod),
					_ => None,
				};
				if let Some(op) = op {
					input.shift();
					let right = parse_expr_unary(input);
					if let Some(right) = right {
						let expr = Expr::Binary(op, left.into(), right.into());
						Some(expr)
					} else {
						return None;
					}
				} else {
					return Some(left);
				}
			} else {
				return Some(left);
			}
		} else {
			return expr;
		};
	}
}

fn parse_expr_unary<T: Reader>(input: &mut TokenStream<T>) -> Option<Expr> {
	match input.get() {
		Token::Identifier => {
			let expr = Expr::Var(Id(input.text().into()));
			input.shift();
			Some(expr)
		}

		Token::Integer => {
			let expr = Expr::Integer(input.text().into());
			input.shift();
			Some(expr)
		}

		Token::String => {
			let text = input.text();
			let text = text.strip_prefix("'").unwrap();
			let text = text.strip_suffix("'").unwrap();
			let expr = Expr::Literal(text.into());
			input.shift();
			Some(expr)
		}

		Token::Symbol => {
			let text = input.text();
			if text == "(" {
				input.shift();
				if let Some(expr) = parse_expr(input) {
					if input.text() != ")" {
						None
					} else {
						input.shift();
						Some(expr)
					}
				} else {
					None
				}
			} else {
				let is_minus = text == "-";
				if is_minus || text == "+" {
					input.shift();
					let expr = parse_expr_unary(input);
					if let Some(expr) = expr {
						let expr = if is_minus {
							Expr::Neg(expr.into())
						} else {
							expr
						};
						Some(expr)
					} else {
						None
					}
				} else {
					None
				}
			}
		}

		_ => None,
	}
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

		let expr = parse_expr(input);
		if let Some(expr) = expr {
			expr_list.push(expr);
		} else {
			break ParseResult::Invalid(input.span(), "expression expected".into());
		}
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
	let expr = parse_expr(input);
	if let Some(expr) = expr {
		let res = ParseResult::Ok(Statement::Let(id, expr));
		parse_end(input, res)
	} else {
		ParseResult::Invalid(input.span(), "expression expected".into())
	}
}

fn parse_end<T: Reader>(input: &mut TokenStream<T>, result: ParseResult) -> ParseResult {
	match input.get() {
		Token::None | Token::LineBreak => result,
		_ => ParseResult::Invalid(input.span(), "expected end of statement".into()),
	}
}
