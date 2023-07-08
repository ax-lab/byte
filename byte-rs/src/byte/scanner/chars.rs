use once_cell::sync::OnceCell;

use super::*;

#[inline(always)]
pub fn char_size(input: &[u8]) -> usize {
	match input.len() {
		0 => 0,
		1 => 1,
		n => {
			for i in 1..n {
				if input[i] & 0b1100_0000 != 0b1000_0000 {
					return i;
				}
			}
			n
		}
	}
}

pub fn check_space(input: &[u8]) -> Option<(char, usize)> {
	static TABLE: OnceCell<SymbolTable<char>> = OnceCell::new();

	let table = TABLE.get_or_init(|| {
		let mut table = SymbolTable::default();
		table.add(" ", ' ');
		table.add("\t", '\t');
		table
	});

	if let (size, Some(char)) = table.recognize(input) {
		Some((*char, size))
	} else {
		None
	}
}

#[inline(always)]
pub fn check_line_break(input: &[u8]) -> Option<usize> {
	const CR: u8 = '\r' as u8;
	const LF: u8 = '\n' as u8;
	match input.len() {
		0 => None,
		1 => {
			if matches!(input[0], CR | LF) {
				Some(1)
			} else {
				None
			}
		}
		_ => match input[0] {
			LF => Some(1),
			CR => Some(if input[1] == LF { 2 } else { 1 }),
			_ => None,
		},
	}
}

#[inline(always)]
pub fn digit_value(n: char) -> u128 {
	(n as u128) - ('0' as u128)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn utf8_char_size() {
		assert_eq!(char_size("".as_bytes()), 0);
		assert_eq!(char_size("a".as_bytes()), 1);
		assert_eq!(char_size("ab".as_bytes()), 1);
		assert_eq!(char_size("äb".as_bytes()), 2);
		assert_eq!(char_size("気!".as_bytes()), 3);
		assert_eq!(char_size("𝄑!".as_bytes()), 4);

		assert_eq!(
			char_size(&[
				0b1100_0000,
				0b1010_0000,
				0b1011_0000,
				0b1011_1100,
				0b1011_1111,
				0b1001_0000,
				0b1000_1111,
				0b1000_0000,
				0b0011_1111,
			]),
			8
		);
	}
}
