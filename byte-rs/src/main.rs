use std::env;

fn main() {
	let mut done = false;
	let mut files = Vec::new();
	let mut show_blocks = false;
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
				"--blocks" => {
					show_blocks = true;
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

	if files.len() == 0 && eval_list.len() == 0 {
		print_usage();
		if files.len() != 0 {
			eprintln!("[error] specify a single file\n");
		} else {
			eprintln!("[error] no arguments given\n");
		}
		std::process::exit(1);
	}

	let mut compiler = byte::new();
	if show_blocks {
		compiler.enable_trace_blocks();
	}

	let mut errors = byte::Errors::new();
	for file in files {
		if let Err(err) = compiler.load_file(file) {
			errors.append(&err);
		}
	}

	if !errors.empty() {
		eprintln!("\n{errors}\n");
		std::process::exit(1);
	}

	compiler.resolve_all();

	let errors = compiler.errors();
	if !errors.empty() {
		eprintln!("");
		eprintln!("{errors}");
		std::process::exit(1);
	}

	for _it in eval_list.into_iter() {
		todo!()
	}
}

fn print_usage() {
	println!("\nUSAGE:\n\n  byte {{FILE}}\n");
	println!("Compiles and executes the given FILE.\n");
}
