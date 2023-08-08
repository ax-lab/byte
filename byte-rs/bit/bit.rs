pub fn some_bit() -> usize {
	42
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn has_bit() {
		assert_eq!(some_bit(), 42);
	}
}
