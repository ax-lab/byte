use std::env;

use byte::*;

fn main() {
	Context::get()
		.write(|context| {
			// main context configuration
			let _ = context;
		})
		.with(|context| {
			let mut done = false;
			let mut files = Vec::new();
			let mut eval_list = Vec::new();
			let mut next_is_eval = false;
			let mut dump_code = false;
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
						"--dump" => {
							dump_code = true;
							false
						}
						"--show-config" => {
							println!("\nGlobal configuration:\n");
							println!("- Tab width: {}", context.tab_width());
							println!("- Base path: {}", context.base_path().to_string_lossy());
							println!("\n");
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

			if files.len() == 0 && eval_list.len() == 0 {
				print_usage();
				eprintln!("No arguments given, nothing to do, exiting...\n");
				std::process::exit(0);
			}

			let compiler = Compiler::new();
			let mut program = compiler.new_program();
			if dump_code {
				program.enable_dump();
			}

			if let Err(errors) = execute(&mut program, files, eval_list) {
				eprintln!("\n{errors}");
				std::process::exit(1);
			}
		});
}

fn execute(program: &mut Program, files: Vec<String>, eval: Vec<String>) -> Result<()> {
	for file in files.into_iter() {
		program.load_file(file)?;
	}

	program.resolve()?;
	program.run()?;

	for (n, expr) in eval.into_iter().enumerate() {
		let name = format!("eval[{n}]");
		let result = program.eval(name, expr)?;
		println!("#{n:02} => {result} ({})", result.get_type().name());
	}

	Ok(())
}

fn print_usage() {
	println!("\nUSAGE:\n\n  byte {{FILE}}\n");
	println!("Compiles and executes the given FILE.\n");
}
