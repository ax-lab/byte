use std::{collections::HashMap, env};

use parser::{parse_statement, BinaryOp, Expr, Id, ParseResult, Statement};

use crate::token::Token;

mod input;
mod lexer;
mod parser;
mod token;

fn main() {
	let mut done = false;
	let mut files = Vec::new();
	let mut list_tokens = false;
	let mut list_ast = false;
	for arg in env::args().skip(1) {
		done = done
			|| match arg.as_str() {
				"--version" | "-v" => {
					println!("\nByte 0.0.1 - rust prototype\n");
					true
				}
				"--help" | "-h" => {
					print_usage();
					true
				}
				"--tokens" => {
					list_tokens = true;
					false
				}
				"--ast" => {
					list_ast = true;
					false
				}
				_ => {
					files.push(arg);
					false
				}
			}
	}

	if done {
		return;
	}

	if files.len() != 1 {
		print_usage();
		if files.len() != 0 {
			eprintln!("[error] specify a single file\n");
		} else {
			eprintln!("[error] no arguments given\n");
		}
		std::process::exit(1);
	}

	for file in files {
		match input::open_file(&file) {
			Ok(mut input) => {
				let mut program = Vec::new();
				loop {
					let next;
					let mut tokens = input.tokens();
					if list_tokens {
						loop {
							let (next, span, text) = (tokens.get(), tokens.span(), tokens.text());
							println!("{span}: {:10}  =  {text:?}", format!("{next:?}"));
							if next == Token::None {
								std::process::exit(0);
							}
							tokens.shift();
						}
					}

					next = parse_statement(&mut tokens);
					match next {
						ParseResult::Invalid(span, msg) => {
							eprintln!("\n[compile error] {input}:{span}: {msg}\n");
							std::process::exit(2);
						}
						ParseResult::Ok(next) => {
							program.push(next);
						}
						ParseResult::EndOfInput => {
							break;
						}
					}
				}

				if list_ast {
					for (i, it) in program.into_iter().enumerate() {
						println!("\n{i:03} = {it:#?}");
					}
					println!();
				} else {
					execute(program);
				}
			}
			Err(msg) => {
				eprintln!("\n[error] open file: {msg}\n");
				std::process::exit(1);
			}
		}
	}
}

fn print_usage() {
	println!("\nUSAGE:\n\n  byte {{FILE}}\n");
	println!("Compiles and executes the given FILE.\n");
}

fn execute(program: Vec<Statement>) {
	let mut map = HashMap::<String, ExprResult>::new();
	for st in program.into_iter() {
		execute_statement(&st, &mut map);
	}
}

fn execute_statement(st: &Statement, map: &mut HashMap<String, ExprResult>) {
	match st {
		Statement::Block(statements) => {
			for it in statements.iter() {
				execute_statement(it, map);
			}
		}

		Statement::Let(Id(id), expr) => {
			let res = execute_expr(expr, map);
			map.insert(id.clone(), res);
		}

		Statement::Print(expr_list) => {
			for (i, expr) in expr_list.iter().enumerate() {
				let res = execute_expr(expr, map);
				if i > 0 {
					print!(" ");
				}
				print!("{res}");
			}
			println!();
		}

		Statement::For(Id(id), from, to, block) => {
			let from = match execute_expr(from, map) {
				ExprResult::Integer(from) => from,
				value => panic!("for: invalid from expression {value:?}"),
			};
			let to = match execute_expr(to, map) {
				ExprResult::Integer(to) => to,
				value => panic!("for: invalid to expression {value:?}"),
			};

			for i in from..=to {
				map.insert(id.clone(), ExprResult::Integer(i));
				execute_statement(block, map);
			}
		}

		Statement::If(cond, block) => {
			let cond = execute_expr(cond, map);
			if cond.to_bool() {
				execute_statement(block, map);
			}
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ExprResult {
	Integer(i64),
	String(String),
	None,
}

impl ExprResult {
	pub fn to_bool(&self) -> bool {
		match &self {
			ExprResult::Integer(0) => false,
			ExprResult::String(x) if x == "" => false,
			ExprResult::None => false,
			_ => true,
		}
	}
}

fn execute_expr(expr: &Expr, map: &mut HashMap<String, ExprResult>) -> ExprResult {
	match expr {
		Expr::Binary(op, left, right) => {
			let left = execute_expr(&left, map);
			let right = execute_expr(&right, map);

			if let BinaryOp::Add = op {
				if let ExprResult::String(left) = left {
					return ExprResult::String(format!("{left}{right}"));
				}

				if let ExprResult::String(right) = right {
					return ExprResult::String(format!("{left}{right}"));
				}
			}

			let left = match left {
				ExprResult::Integer(value) => value,
				v => panic!("{op} with left `{v}` is invalid"),
			};
			let right = match right {
				ExprResult::Integer(value) => value,
				v => panic!("{op} with `{left}` and `{v}` is invalid"),
			};
			let result = match op {
				BinaryOp::Add => left + right,
				BinaryOp::Sub => left - right,
				BinaryOp::Mul => left * right,
				BinaryOp::Div => left / right,
				BinaryOp::Mod => left % right,
			};
			ExprResult::Integer(result)
		}

		Expr::Integer(value) => ExprResult::Integer(value.parse().unwrap()),
		Expr::Literal(value) => ExprResult::String(value.clone()),
		Expr::Neg(value) => match execute_expr(&value, map) {
			ExprResult::Integer(value) => ExprResult::Integer(-value),
			v => panic!("minus operand `{v}` is not a number"),
		},

		Expr::Var(Id(id)) => {
			if let Some(value) = map.get(id) {
				value.clone()
			} else {
				ExprResult::None
			}
		}

		Expr::TernaryConditional(cond, left, right) => {
			let cond = execute_expr(cond, map);
			if cond.to_bool() {
				execute_expr(left, map)
			} else {
				execute_expr(right, map)
			}
		}

		Expr::Equality(left, right) => {
			let left = execute_expr(left, map);
			let right = execute_expr(right, map);
			if left == right {
				ExprResult::Integer(1)
			} else {
				ExprResult::Integer(0)
			}
		}
	}
}

impl std::fmt::Display for ExprResult {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ExprResult::Integer(v) => write!(f, "{v}"),
			ExprResult::String(v) => write!(f, "{v}"),
			ExprResult::None => write!(f, "(none)"),
		}
	}
}
