use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;

use crate::core::error::*;
use crate::core::input::*;

mod indent;
mod stream;
mod symbol;
mod token;

pub use stream::*;
pub use token::*;

use indent::*;
use symbol::*;

/// Holds all the lexer state and provides lexing for tokens.
#[derive(Clone)]
pub struct Lexer {
	scanner: Rc<Scanner>,
	state: State,
	next: RefCell<Option<(TokenAt, State)>>,
}

#[derive(Clone)]
struct State {
	input: Cursor,
	indent: Indent,
	errors: ErrorList,
}

impl Lexer {
	pub fn new(input: Cursor, scanner: Scanner) -> Self {
		Lexer {
			scanner: Rc::new(scanner),
			state: State {
				input,
				indent: Indent::new(),
				errors: ErrorList::new(),
			},
			next: RefCell::new(None),
		}
	}

	pub fn config<F: FnOnce(&mut Scanner)>(&mut self, config: F) {
		self.next.replace(None);
		let scanner = Rc::make_mut(&mut self.scanner);
		config(scanner)
	}

	pub fn errors(&self) -> ErrorList {
		self.state.errors.clone()
	}

	pub fn next(&self) -> TokenAt {
		let token = {
			let next = self.next.borrow();
			if let Some((token, ..)) = &*next {
				Some(token.clone())
			} else {
				None
			}
		};
		if let Some(token) = token {
			token
		} else {
			let mut clone = self.clone();
			let next = clone.read();
			self.next.replace(Some((next.clone(), clone.state)));
			next
		}
	}

	pub fn read(&mut self) -> TokenAt {
		if let Some((token, state)) = self.next.take() {
			self.state = state;
			return token;
		}

		let state = &mut self.state;
		self.scanner.skip(&mut state.input);

		let start = state.input.clone();
		let token = if let Some(token) = state.indent.check_indent(&state.input, &mut state.errors)
		{
			token
		} else {
			self.scanner.read(&mut state.input, &mut state.errors)
		};

		let span = Span {
			sta: start.clone(),
			end: state.input.clone(),
		};
		TokenAt(span, token)
	}
}

impl Stream for Lexer {
	fn pos(&self) -> Cursor {
		self.state.input.clone()
	}

	fn copy(&self) -> Box<dyn Stream> {
		Box::new(self.clone())
	}

	fn next(&self) -> TokenAt {
		Lexer::next(self)
	}

	fn read(&mut self) -> TokenAt {
		Lexer::read(self)
	}

	fn errors_ref(&self) -> &ErrorList {
		&self.state.errors
	}

	fn errors_mut(&mut self) -> &mut ErrorList {
		&mut self.state.errors
	}
}

pub trait Matcher {
	fn try_match(&self, next: char, input: &mut Cursor, errors: &mut ErrorList) -> Option<Token>;
}

#[derive(Clone, Debug)]
pub struct TokenAt(Span, Token);

impl TokenAt {
	pub fn span(&self) -> Span {
		self.0.clone()
	}

	pub fn token(&self) -> Token {
		self.1.clone()
	}

	pub fn symbol(&self) -> Option<&str> {
		let str = match &self.1 {
			Token::Symbol(symbol) => *symbol,
			Token::Identifier => self.0.text(),
			_ => return None,
		};
		Some(str)
	}
}

impl std::fmt::Display for TokenAt {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.1 {
			Token::None => {
				write!(f, "end of input")
			}
			Token::Invalid => {
				write!(f, "invalid token")
			}
			Token::Break => {
				write!(f, "line break")
			}
			Token::Indent => {
				write!(f, "indent")
			}
			Token::Dedent => {
				write!(f, "dedent")
			}
			Token::Symbol(sym) => write!(f, "`{sym}`"),
			Token::Identifier => {
				write!(f, "`{}`", self.span().text())
			}
			Token::Value(value) => write!(f, "`{value}`"),
		}
	}
}

/// Provides low-level token scanning using a configurable list of matchers
/// and symbol table.
#[derive(Clone)]
pub struct Scanner {
	scanners: Vec<Rc<dyn Matcher>>,
	symbols: SymbolTable,
}

impl Scanner {
	pub fn new() -> Scanner {
		Scanner {
			scanners: Vec::new(),
			symbols: SymbolTable::default(),
		}
	}

	pub fn add_scanner<T: Matcher + 'static>(&mut self, scanner: T) {
		self.scanners.push(Rc::new(scanner));
	}

	pub fn add_symbol(&mut self, symbol: &'static str, value: Token) {
		self.symbols.add_symbol(symbol, value);
	}

	/// Skip spaces and empty lines from the input cursor.
	pub fn skip(&self, input: &mut Cursor) {
		let is_start = input.col() == 0;
		let mut saved = input.clone();
		while let Some(next) = input.read() {
			if is_space(next) || (is_start && next == '\n') {
				saved = input.clone();
			} else {
				break;
			}
		}
		*input = saved;
	}

	fn read(&self, input: &mut Cursor, errors: &mut ErrorList) -> Token {
		let mut start = input.clone();
		if let Some(next) = input.read() {
			if next == '\n' {
				return Token::Break;
			}
			let saved = (input.clone(), errors.clone());
			for scanner in self.scanners.iter() {
				if let Some(token) = scanner.try_match(next, input, errors) {
					return token;
				} else {
					(*input, *errors) = saved.clone();
				}
			}

			// if none of the scanners matched, try the symbols
			if let Some(token) = self.symbols.try_match(next, input, errors) {
				token
			} else {
				(*input, *errors) = saved;
				errors.at(
					Span {
						sta: start,
						end: input.clone(),
					},
					LexerError::InvalidSymbol,
				);
				Token::Invalid
			}
		} else {
			Token::None
		}
	}
}

#[derive(Debug)]
pub enum LexerError {
	InvalidSymbol,
	InvalidDedentInRegion,
	InvalidDedentIndent,
	ExpectedEnd(TokenAt),
}

impl ErrorInfo for LexerError {
	fn output(&self, f: &mut std::fmt::Formatter<'_>, span: &Span) -> std::fmt::Result {
		match self {
			LexerError::InvalidSymbol => write!(f, "invalid symbol"),
			LexerError::InvalidDedentInRegion => {
				write!(f, "indentation level is less than the enclosing expression")
			}
			LexerError::InvalidDedentIndent => {
				write!(
					f,
					"cannot dedent and indent in a single line, return to previous level first"
				)
			}
			LexerError::ExpectedEnd(got) => {
				write!(f, "expected end, got `{got}`")
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn s(symbol: &'static str) -> Token {
		Token::Symbol(symbol)
	}

	#[test]
	fn empty() {
		test("", &vec![]);
	}

	#[test]
	fn simple() {
		test("+-", &vec![s("+"), s("-")])
	}

	#[test]
	fn skip_spaces() {
		let expected = &vec![s("+"), s("-")];
		test("+ -", expected);
		test("+\t-", expected);
		test("+ \t \t -", expected);
		test("\n+ -", expected);
		test("  \n\n+ -", expected);
		test("\r+ -", expected);
		test("\r\n+ -", expected);
	}

	#[test]
	fn line_breaks() {
		let expected = &vec![s("+"), Token::Break, s("-")];
		test("+\n-", expected);
		test("+\r-", expected);
		test("+\r\n-", expected);
		test("+\r\n\r\n-", expected);

		let mut expected = expected.clone();
		expected.push(Token::Break);
		test("+\n\n-\n", &expected);
	}

	#[test]
	fn indent_simple() {
		// plain indent then dedent
		test(
			"+\n\t-\n+",
			&vec![
				s("+"),
				Token::Break,
				Token::Indent,
				s("-"),
				Token::Break,
				Token::Dedent,
				s("+"),
			],
		);

		// dedent at the end of file
		test(
			"+\n\t-\n\t+",
			&vec![
				s("+"),
				Token::Break,
				Token::Indent,
				s("-"),
				Token::Break,
				s("+"),
				Token::Dedent,
			],
		);

		// multiple indents
		test(
			"+\n\t-\n\t+\n\t\t*\n\t\t/\n\t-",
			&vec![
				s("+"),
				Token::Break,
				Token::Indent,
				s("-"),
				Token::Break,
				s("+"),
				Token::Break,
				Token::Indent,
				s("*"),
				Token::Break,
				s("/"),
				Token::Break,
				Token::Dedent,
				s("-"),
				Token::Dedent,
			],
		);
	}

	fn test(input: &str, expected: &Vec<Token>) {
		let input = Input::open_str("test", input);

		let mut scanner = Scanner::new();
		scanner.add_symbol("+", s("+"));
		scanner.add_symbol("-", s("-"));
		scanner.add_symbol("*", s("*"));
		scanner.add_symbol("/", s("/"));

		let mut output = Vec::new();
		let mut lexer = Lexer::new(input.start(), scanner);
		let mut clone = lexer.clone();
		loop {
			let TokenAt(span, token) = lexer.read();
			if token == Token::None {
				break;
			}

			// sanity check clone
			let TokenAt(clone_span, clone_token) = clone.read();
			assert_eq!(clone_span, span);
			assert_eq!(clone_token, token);

			output.push(token);
		}

		let errors = lexer.errors().list();
		if errors.len() > 0 {
			for it in errors.into_iter() {
				eprintln!("\n{it}");
			}
			eprintln!();
			panic!("lexer generated errors");
		}

		assert_eq!(&output, expected);
	}
}
