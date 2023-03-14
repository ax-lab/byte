use std::{collections::HashMap, env};

use exec::{execute_expr, ResultValue};
use lexer::Context;
use parser::{parse_statement, Id, ParseResult, Statement};

mod input;
use input::Input;

mod error;
pub use error::*;

mod exec;
mod lexer;
mod parser;
mod source;

fn main() {
	let mut done = false;
	let mut files = Vec::new();
	let mut list_tokens = false;
	let mut list_ast = false;
	let mut list_blocks = false;
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
				"--blocks" => {
					list_blocks = true;
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
		match source::open_file(&file) {
			Ok(source) => {
				let mut context = lexer::open(&source);
				if list_tokens {
					while context.value().is_some() {
						let token = context.token();
						let span = context.span();
						let text = span.text();
						println!("{span}: {:10}  =  {token:?}", format!("{text:?}"));
						context.next();
					}
					print_errors(&context);
					std::process::exit(0);
				}

				if list_blocks {
					parser::list_blocks(&mut context);
					print_errors(&context);
					std::process::exit(0);
				}

				let mut program = Vec::new();
				while context.value().is_some() {
					let parsed = parse_statement(&mut context);
					match parsed {
						ParseResult::Error(span, msg) => {
							eprintln!("\nIn {file}:{span}:\n\n    error parsing: {msg}");
							print_errors(&context);
							eprintln!();
							if list_ast {
								break;
							}
							std::process::exit(2);
						}
						ParseResult::Ok(parsed) => {
							program.push(parsed);
						}
						ParseResult::None => {
							break;
						}
					}
				}

				print_errors(&context);

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

fn print_errors(ctx: &Context) {
	let mut has_errors = false;
	for it in ctx.errors() {
		if !has_errors {
			eprintln!("\n---- Errors ----\n");
			has_errors = true;
		}
		eprintln!("    error: {it} at {}", it.span());
	}
}

fn print_usage() {
	println!("\nUSAGE:\n\n  byte {{FILE}}\n");
	println!("Compiles and executes the given FILE.\n");
}

fn execute(program: Vec<Statement>) {
	let mut map = HashMap::<String, ResultValue>::new();
	for st in program.into_iter() {
		execute_statement(&st, &mut map);
	}
}

fn execute_statement(st: &Statement, map: &mut HashMap<String, ResultValue>) {
	match st {
		Statement::Block(statements) => {
			for it in statements.iter() {
				execute_statement(it, map);
			}
		}

		Statement::Expr(expr) => {
			execute_expr(expr, map);
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
				ResultValue::Integer(from) => from,
				value => panic!("for: invalid from expression {value:?}"),
			};
			let to = match execute_expr(to, map) {
				ResultValue::Integer(to) => to,
				value => panic!("for: invalid to expression {value:?}"),
			};

			for i in from..=to {
				map.insert(id.clone(), ResultValue::Integer(i));
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
