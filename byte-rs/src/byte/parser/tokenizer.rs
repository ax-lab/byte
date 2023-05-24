use crate::core::errors::*;

pub const TAB_WIDTH: usize = 4;

/*
	Goals:

	- tokenize the whole input into a list of tokens segments
	- segments are parsed into nodes in order of parsing priority
	- segments are processed by range, with the initial range being the whole input
	- parsed nodes may contain ranges, that they will solve recursively
	- parsed nodes may create additional ranges for themselves
	- nodes are immutable, resolving a node generates a new node
	- nodes publish a list of input/output bindings that they need to solve

	For any given range we need:

	- quickly lookup a particular symbol or token from a position
	- check for the presence of a list of symbols

*/

pub trait Scanner {
	fn try_match(&mut self, input: &[u8], errors: &mut Errors) -> (usize, Option<Token>);
}

pub trait ScannerWithOr<T: Scanner>: Scanner {
	type Output: Scanner;

	fn or(self, other: T) -> Self::Output;
}

impl<T: Scanner, S: Scanner> ScannerWithOr<T> for S {
	type Output = ScanOr<Self, T>;

	fn or(self, other: T) -> Self::Output {
		ScanOr::new(self, other)
	}
}

pub fn try_match(s: &dyn Scanner) {}

pub struct ScanOr<A: Scanner, B: Scanner> {
	a: A,
	b: B,
}

impl<A: Scanner, B: Scanner> ScanOr<A, B> {
	pub fn new(a: A, b: B) -> Self {
		Self { a, b }
	}
}

impl<A: Scanner, B: Scanner> Scanner for ScanOr<A, B> {
	fn try_match(&mut self, input: &[u8], errors: &mut Errors) -> (usize, Option<Token>) {
		let (size, output) = self.a.try_match(input, errors);
		if size == 0 {
			self.b.try_match(input, errors)
		} else {
			(size, output)
		}
	}
}

pub enum Token {
	Pragma,
	Break,
	Indent,
	Dedent,
	Comment,
	Word,
	Value,
	Symbol,
	Unknown,
}

pub struct Segment {
	pub pos: usize,
	pub len: usize,
}

pub struct SegmentData {
	input: Vec<Input>,
	tokens: Vec<Token>,
	values: Vec<usize>,
	breaks: Vec<usize>,
	indent: Vec<usize>,
	offset: Vec<(usize, usize)>,
}

impl SegmentData {}

pub struct Input {
	pub name: String,
	pub data: Vec<u8>,
}

#[derive(Default)]
pub struct Tokenizer {
	tab_width: usize,
	symbols: Vec<String>,
}

impl Tokenizer {
	pub fn segment(&self, input: &[u8], output: &mut SegmentData, errors: &mut Errors) {}

	pub fn add_symbol(&mut self, symbol: &str) -> usize {
		todo!()
	}

	pub fn add_brackets(&mut self, s: &str, e: &str) {
		todo!()
	}

	pub fn add_comment(&mut self, s: &str) {
		todo!()
	}

	pub fn add_comment_brackets(&mut self, s: u8, e: u8) {
		todo!()
	}

	pub fn skip(&mut self, char: char) {
		todo!()
	}

	pub fn set_tab_width(&mut self, width: usize) {
		self.tab_width = width;
	}

	fn next_utf8_len(input: &[u8]) -> usize {
		// skip continuation bytes after the first byte, this assumes the input
		// is not empty
		let mut size = 1;
		while size < input.len() && (input[size] & 0b1100_0000) == 0b1000_0000 {
			size += 1;
		}
		size
	}
}

//====================================================================================================================//
// Symbol table
//====================================================================================================================//

pub struct Table<T> {
	states: Vec<State<T>>,
}

impl<T> Default for Table<T> {
	fn default() -> Self {
		Self {
			states: vec![Default::default()],
		}
	}
}

impl<T> Table<T> {
	pub fn add(&mut self, input: &str, result: T) -> Option<T> {
		let mut state = 0;
		let input = input.as_bytes();
		for byte in input {
			let byte = *byte;
			state = if let Some(next) = self.get_next(state, byte) {
				next
			} else {
				let next = self.states.len();
				self.states.push(Default::default());

				let current = &mut self.states[state];
				current.next.push(StateNext { byte, state: next });
				current.next.sort_by_key(|x| x.byte);
				next
			};
		}

		let state = &mut self.states[state];
		state.done.replace(result)
	}

	pub fn default(&self) -> Option<&T> {
		self.states[0].done.as_ref()
	}

	pub fn set_default(&mut self, value: T) -> Option<T> {
		self.states[0].done.replace(value)
	}

	pub fn recognize(&self, input: &[u8]) -> (usize, Option<&T>) {
		let mut valid_state = self.states[0].done.as_ref();
		let mut valid_index = 0;
		let mut state = 0;
		let mut index = 0;
		while let Some(next) = input.get(index).and_then(|x| self.get_next(state, *x)) {
			state = next;
			index += 1;
			if let Some(ref done) = self.states[state].done {
				valid_state = Some(done);
				valid_index = index;
			}
		}
		(valid_index, valid_state)
	}

	fn get_next(&self, state: usize, byte: u8) -> Option<usize> {
		let state = &self.states[state];
		if let Ok(index) = state.next.binary_search_by_key(&byte, |x| x.byte) {
			Some(state.next[index].state)
		} else {
			None
		}
	}
}

struct State<T> {
	done: Option<T>,
	next: Vec<StateNext>,
}

struct StateNext {
	byte: u8,
	state: usize,
}

impl<T> Default for State<T> {
	fn default() -> Self {
		Self {
			done: None,
			next: Default::default(),
		}
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	pub fn table() {
		let table: &mut Table<i32> = &mut Default::default();

		check(table, "", 0, None);
		check(table, "abc", 0, None);

		table.set_default(-1);
		check(table, "", 0, Some(-1));
		check(table, "abc", 0, Some(-1));

		table.add("zero", 0);
		table.add("one", 1);
		table.add("two", 2);
		table.add("three", 3);

		table.add("0", 0);
		table.add("00", 0);
		table.add("01", 1);

		table.add("ten", 10);
		table.add("ten+one", 11);

		table.add("0000", 4);
		table.add("000000", 6);

		check(table, "abc", 0, Some(-1));
		check(table, "zer", 0, Some(-1));

		check_ok(table, "zero", 0);
		check_ok(table, "one", 1);
		check_ok(table, "two", 2);
		check_ok(table, "three", 3);

		check_ok(table, "0", 0);
		check_ok(table, "00", 0);
		check_ok(table, "01", 1);

		check_ok(table, "ten", 10);
		check_ok(table, "ten+one", 11);

		check_ok(table, "0000", 4);
		check_ok(table, "000000", 6);

		check(table, "0", 1, Some(0));
		check(table, "00", 2, Some(0));
		check(table, "000", 2, Some(0));
		check(table, "0000", 4, Some(4));
		check(table, "00000", 4, Some(4));
		check(table, "000000", 6, Some(6));
		check(table, "0000000", 6, Some(6));

		check(table, "ten", 3, Some(10));
		check(table, "ten+", 3, Some(10));
		check(table, "ten+o", 3, Some(10));
		check(table, "ten+on", 3, Some(10));
		check(table, "ten+one", 7, Some(11));
		check(table, "ten+one!", 7, Some(11));

		assert_eq!(table.add("0000", -4), Some(4));
		check(table, "00000", 4, Some(-4));

		fn check(table: &Table<i32>, str: &str, size: usize, value: Option<i32>) {
			let actual = table.recognize(str.as_bytes());
			let expected = (size, value.as_ref());
			assert_eq!(actual, expected);
		}

		fn check_ok(table: &Table<i32>, str: &str, value: i32) {
			check(table, str, str.len(), Some(value));

			let str_with_suffix = format!("{str}XXX");
			check(table, &str_with_suffix, str.len(), Some(value));
		}
	}
}
