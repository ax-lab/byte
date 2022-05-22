#[test]
fn hello() {
	let output = tux::run_bin("byte", &["tests/scripts/hello.by"]);
	let expected = include_str!("scripts/hello.out");
	assert_eq!(output.trim(), expected.trim());
}
