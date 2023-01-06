use std::env;

fn main() {
	println!();
	println!("Hello, world from Byte! (rs)\n");

	let args = env::args().map(|x| format!("`{}`", x)).collect::<Vec<_>>();
	let args = args.join("  ");
	println!("ARGS: {}", args);

	let cwd = env::current_dir().unwrap();
	let cwd = cwd.to_str().unwrap();
	println!("WDIR: {}", cwd);

	println!();
}
