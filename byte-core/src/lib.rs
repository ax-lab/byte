pub fn name() -> &'static str {
	"Byte Language"
}

pub fn version() -> &'static str {
	"0.1.0"
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		assert!(name().contains("Byte"));
		assert!(version() != "");
	}
}
