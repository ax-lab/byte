use std::env;

mod core;
mod eval;
mod lang;
mod lexer;
mod nodes;
mod parser;

#[allow(unused)]
mod vm;

mod old;

fn main() {
	use old::stream::Stream;

	let mut done = false;
	let mut files = Vec::new();
	let mut list_tokens = false;
	let mut list_ast = false;
	let mut eval_list = Vec::new();
	let mut next_is_eval = false;
	let mut is_blocks = false;
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
				"--eval" => {
					next_is_eval = true;
					false
				}
				"--blocks" => {
					is_blocks = true;
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
		let context = lexer::open(core::input::Input::open_str("eval", &it));
		let result = old::eval::run(context, false);
		println!("{result}");
	}

	for file in files {
		match core::input::Input::open_file(&file) {
			Ok(source) => {
				if is_blocks {
					parser::parse(source.clone());
				}

				let mut lexer = lexer::open(source);
				if list_tokens {
					let mut n = 1;
					while lexer.next().is_some() {
						let next = lexer.next();
						let token = next.token();
						let span = next.span();
						let text = span.text();
						println!(
							"\n>>> {n:03}. {span}:\n    {:10}  =  {token:?}",
							format!("{text:?}")
						);
						lexer.advance();
						n += 1;
					}
					print_errors(&lexer);
					std::process::exit(0);
				}
				old::eval::run(lexer, list_ast);
			}
			Err(msg) => {
				eprintln!("\n[error] open file: {msg}\n");
				std::process::exit(1);
			}
		}
	}
}

fn print_errors(ctx: &lexer::Lexer) {
	print_error_list(ctx.errors());
}

fn print_error_list(errors: crate::core::error::ErrorList) {
	if !errors.empty() {
		let mut has_errors = false;
		for (i, it) in errors.list().into_iter().enumerate() {
			if !has_errors {
				eprintln!("\n---- Errors ----\n");
				has_errors = true;
			}
			eprint!("[Error {}] {it}", i + 1);
			if let Some(span) = it.span() {
				eprint!(" at {}", span);
			}
			eprintln!();
		}
		eprintln!();
	}
}

fn print_usage() {
	println!("\nUSAGE:\n\n  byte {{FILE}}\n");
	println!("Compiles and executes the given FILE.\n");
}
