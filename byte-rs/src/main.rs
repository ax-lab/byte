use std::env;

use byte::core::*;

fn main() {
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
		match Input::open(&file) {
			Ok(_source) => {
				if is_blocks {
					todo!()
				}

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

fn print_usage() {
	println!("\nUSAGE:\n\n  byte {{FILE}}\n");
	println!("Compiles and executes the given FILE.\n");
}
