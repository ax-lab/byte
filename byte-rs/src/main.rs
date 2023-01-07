use std::env;

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
		match std::fs::read_to_string(&file) {
			Ok(content) => {
				execute(&file, &content);
			}
			Err(msg) => {
				eprintln!("\n[error] reading {}: {}\n", file, msg);
				std::process::exit(1);
			}
		}
	}
}

fn print_usage() {
	println!("\nUSAGE:\n\n  byte {{FILE}}\n");
	println!("Compiles and executes the given FILE.\n");
}

fn execute(name: &str, input: &str) {
	for (n, line) in input.lines().enumerate() {
		let n = n + 1;
		let line = line.trim();
		if line == "" {
			continue;
		}

		let exit_with_err = |msg: &str| {
			eprintln!("\n[compile error] {name}:{n}: {msg}\n\n    |{n:03}| {line}\n");
			std::process::exit(2);
		};

		let text = match line.strip_prefix("print ") {
			Some(text) => text,
			_ => exit_with_err("invalid command"),
		};

		let literal = text
			.trim()
			.strip_prefix("'")
			.map(|x| (x, '\''))
			.or_else(|| text.trim().strip_prefix("\"").map(|x| (x, '"')));

		let literal = literal
			.and_then(|(s, delim)| s.strip_suffix(delim))
			.unwrap_or_else(|| exit_with_err("invalid string literal"));

		println!("{}", literal);
	}
}
