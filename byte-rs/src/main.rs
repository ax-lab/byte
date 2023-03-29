use std::env;

use lexer::{LexStream, Stream};

mod input;
use input::Input;

mod error;
pub use error::*;

mod eval;
mod lexer;
mod parser;
mod runtime;
mod source;

fn main() {
	let mut done = false;
	let mut files = Vec::new();
	let mut list_tokens = false;
	let mut list_ast = false;
	let mut list_blocks = false;
	let mut eval_list = Vec::new();
	let mut next_is_eval = false;
	for arg in env::args().skip(1) {
		if next_is_eval {
			next_is_eval = false;
			eval_list.push(arg);
			continue;
		}
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
				"--eval" => {
					next_is_eval = true;
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

	if files.len() != 1 && eval_list.len() == 0 {
		print_usage();
		if files.len() != 0 {
			eprintln!("[error] specify a single file\n");
		} else {
			eprintln!("[error] no arguments given\n");
		}
		std::process::exit(1);
	}

	for it in eval_list.into_iter() {
		let text = it.as_str();
		let context = lexer::open(&text);
		let result = eval::run(context, false);
		println!("{result}");
	}

	for file in files {
		match source::open_file(&file) {
			Ok(source) => {
				let mut context = lexer::open(&source);
				if list_tokens {
					while context.next().is_some() {
						let token = context.token();
						let span = context.span();
						let text = span.text();
						println!("{span}: {:10}  =  {token:?}", format!("{text:?}"));
						context.advance();
					}
					print_errors(&context);
					std::process::exit(0);
				}

				if list_blocks {
					parser::list_blocks(&mut context);
					print_errors(&context);
					std::process::exit(0);
				}
				eval::run(context, list_ast);
			}
			Err(msg) => {
				eprintln!("\n[error] open file: {msg}\n");
				std::process::exit(1);
			}
		}
	}
}

fn print_errors(ctx: &Stream) {
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
