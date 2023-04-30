use std::env;

mod core;
mod eval;
mod lang;
mod lexer;
mod nodes;
mod parser;
mod vm;

fn main() {
	println!("{}", byte::hello());

	let mut done = false;
	let mut files = Vec::new();
	let mut list_tokens = false;
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

	for _it in eval_list.into_iter() {
		todo!()
	}

	for file in files {
		match core::input::Input::open_file(&file) {
			Ok(source) => {
				if is_blocks {
					parser::parse(source.clone());
				}

				let mut _lexer = lexer::open(source);
				if list_tokens {
					todo!();
				}
			}
			Err(msg) => {
				eprintln!("\n[error] open file: {msg}\n");
				std::process::exit(1);
			}
		}
	}
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
