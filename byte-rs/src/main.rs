use std::{collections::HashMap, env};

use input::TokenStream;
use parser::{parse_statement, BinaryOp, Expr, Id, ParseResult, Statement};

mod input;
mod lexer;
mod parser;

fn main() {
	let mut done = false;
	let mut files = Vec::new();
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
				let mut token = input.next();
				loop {
					let next;
					(next, token) = parse_statement(&mut input, token);
					match next {
						ParseResult::Error(io_err) => {
							eprintln!("\n[error] reading {}: {}\n", file, io_err);
							std::process::exit(1);
						}
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
				execute(program);
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
		match st {
			Statement::Let(Id(id), expr) => {
				let res = execute_expr(expr, &mut map);
				map.insert(id, res);
			}

			Statement::Print(expr_list) => {
				for (i, expr) in expr_list.into_iter().enumerate() {
					let res = execute_expr(expr, &mut map);
					if i > 0 {
						print!(" ");
					}
					print!("{res}");
				}
				println!();
			}
		}
	}
}

#[derive(Clone, Debug)]
enum ExprResult {
	Integer(i64),
	String(String),
	None,
}

fn execute_expr(expr: Expr, map: &mut HashMap<String, ExprResult>) -> ExprResult {
	match expr {
		Expr::Binary(op, left, right) => {
			let left = execute_expr(*left, map);
			let right = execute_expr(*right, map);
			if let (BinaryOp::Add, ExprResult::String(left)) = (op, &left) {
				ExprResult::String(format!("{}{}", left, right))
			} else {
				let left = match left {
					ExprResult::Integer(value) => value,
					v => panic!("{op} left operator `{v}` is not a number"),
				};
				let right = match right {
					ExprResult::Integer(value) => value,
					v => panic!("{op} right operator `{v}` is not a number"),
				};
				let result = match op {
					BinaryOp::Add => left + right,
					BinaryOp::Sub => left - right,
					BinaryOp::Mul => left * right,
					BinaryOp::Div => left / right,
				};
				ExprResult::Integer(result)
			}
		}

		Expr::Integer(value) => ExprResult::Integer(value.parse().unwrap()),
		Expr::Literal(value) => ExprResult::String(value),
		Expr::Neg(value) => match execute_expr(*value, map) {
			ExprResult::Integer(value) => ExprResult::Integer(-value),
			v => panic!("minus operand `{v}` is not a number"),
		},

		Expr::Var(Id(id)) => {
			if let Some(value) = map.get(&id) {
				value.clone()
			} else {
				ExprResult::None
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
