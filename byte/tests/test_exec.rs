#[test]
fn hello_world() {
	let result = byte::exec("print 'hello world'");
	assert!(result.success());
	assert_eq!(result.stdout(), "hello world");

	let result = byte::exec("print 'hello test'");
	assert!(result.success());
	assert_eq!(result.stdout(), "hello test");
}
