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

	let mut context = byte::new();
	if show_blocks {
		context.enable_trace_blocks();
	}

	for file in files {
		context.load_file(file);
	}

	context.wait_resolve();

	let errors = context.errors();
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
