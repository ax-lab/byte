pub fn hello() -> &'static str {
	"hello from byte"
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn says_hello() {
		assert!(hello().contains("hello"));
	}
}
