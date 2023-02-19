use std::{collections::HashMap, env};

use exec::{execute_expr, Result};
use parser::{parse_statement, Id, ParseResult, Statement};

use crate::token::Token;

mod input;
mod lexer;
mod parser;
mod token;

mod exec;

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

				loop {
					let next;
					next = parse_statement(&mut tokens);
					match next {
						ParseResult::Invalid(span, msg) => {
							eprintln!("\n[compile error] {input}:{span}: {msg}\n");
							std::process::exit(2);
						}
						ParseResult::Ok(next) => {
							program.push(next);
						}
						ParseResult::None => {
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
	let mut map = HashMap::<String, Result>::new();
	for st in program.into_iter() {
		execute_statement(&st, &mut map);
	}
}

fn execute_statement(st: &Statement, map: &mut HashMap<String, Result>) {
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
				Result::Integer(from) => from,
				value => panic!("for: invalid from expression {value:?}"),
			};
			let to = match execute_expr(to, map) {
				Result::Integer(to) => to,
				value => panic!("for: invalid to expression {value:?}"),
			};

			for i in from..=to {
				map.insert(id.clone(), Result::Integer(i));
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
