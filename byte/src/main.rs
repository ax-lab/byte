fn main() {
	for file in std::env::args().skip(1) {
		let input = std::fs::read_to_string(&file).unwrap();
		let result = byte::exec(input);
		println!("{}", result.stdout());
	}
}
