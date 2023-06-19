use std::env;

use byte::*;

fn main() {
	let mut done = false;
	let mut files = Vec::new();
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
		eprintln!("No arguments given, nothing to do, exiting...\n");
		std::process::exit(0);
	}

	if let Err(errors) = execute(files, eval_list) {
		println!("\n{errors}");
		std::process::exit(1);
	}
}

fn execute(files: Vec<String>, eval: Vec<String>) -> Result<()> {
	let compiler = Compiler::new();

	let mut modules = Vec::new();
	for file in files.into_iter() {
		let module = compiler.load_file(file)?;
		modules.push(module);
	}

	for it in modules.iter() {
		it.resolve()?;
	}

	if eval.len() == 0 {
		if let Some(module) = modules.first() {
			module.eval()?;
		}
	}

	for (n, expr) in eval.into_iter().enumerate() {
		let name = format!("{{eval #{n}}}");
		let input = Input::new(name, expr.as_bytes().to_vec());
		let module = compiler.load_input(input);
		let result = module.eval()?;

		println!("#{n:02} => {result} ({})", result.type_name());
	}

	Ok(())
}

fn print_usage() {
	println!("\nUSAGE:\n\n  byte {{FILE}}\n");
	println!("Compiles and executes the given FILE.\n");
}
