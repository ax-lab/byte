use super::*;

static COMMON_SYMBOLS: &[&'static str] = &["(", ")", "[", "]", "{", "}", ";", ":", ",", "."];

const ALPHA: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";
const DIGIT: &'static str = "0123456789";

/// Trait for a matcher that can be used by the [`Scanner`].
pub trait Matcher {
	fn try_match(&self, cursor: &mut Cursor, errors: &mut Errors) -> Option<Node>;
}

#[derive(Clone)]
pub struct Scanner {
	compiler: CompilerRef,
	matchers: Arc<Vec<Arc<dyn Matcher>>>,
	table: Arc<SymbolTable<ScanAction>>,
}

impl Scanner {
	pub fn new(compiler: CompilerRef) -> Self {
		Self {
			compiler,
			matchers: Default::default(),
			table: Default::default(),
		}
	}

	pub fn register_common_symbols(&mut self) {
		for it in COMMON_SYMBOLS.iter() {
			self.add_symbol(it);
		}
		self.add_word_chars(ALPHA);
		self.add_word_next_chars(DIGIT);
	}

	pub fn add_matcher<T: Matcher + 'static>(&mut self, matcher: T) {
		let matchers = Arc::make_mut(&mut self.matchers);
		matchers.push(Arc::new(matcher));
	}

	pub fn add_symbol(&mut self, name: &str) {
		let table = Arc::make_mut(&mut self.table);
		table.add(name, ScanAction::Symbol(name.to_string()));
	}

	pub fn add_word_chars(&mut self, chars: &str) {
		self.add_action_chars(chars, ScanAction::Word);
	}

	pub fn add_word_next_chars(&mut self, chars: &str) {
		self.add_action_chars(chars, ScanAction::WordNext);
	}

	fn add_action_chars(&mut self, chars: &str, kind: ScanAction) {
		let table = Arc::make_mut(&mut self.table);
		let mut buffer: [u8; 4] = [0; 4];
		for char in chars.chars() {
			let str = char::encode_utf8(char, &mut buffer);
			table.add(str, kind.clone());
		}
	}

	pub fn scan(&self, cursor: &mut Cursor, errors: &mut Errors) -> Option<Node> {
		let compiler = &self.compiler.get();
		loop {
			// skip spaces
			let start = cursor.clone();
			let line_start = cursor.is_indent();
			let mut has_indent = false;
			while let Some((.., skip_len)) = check_space(cursor.data()) {
				assert!(skip_len > 0);
				cursor.advance(skip_len);
				has_indent = line_start;
			}

			// check for a line break
			if let Some(size) = check_line_break(cursor.data()) {
				assert!(size > 0);
				let span = cursor.advance_span(size);

				// ignore empty or space-only lines
				if !line_start {
					return Some(Node::from(Token::Break, Some(span)));
				} else {
					continue;
				}
			}

			// check for the end of input
			if cursor.at_end() || !errors.empty() {
				return None;
			}

			// generate a meaningful indent token
			if has_indent {
				/*
					TODO: validate bad indentation scenarios

					- spaces followed by tabs
					- for any consecutive non-empty lines, one of the lines
					  indentation MUST be a prefix of the other
					  - indentation must be consistent between consecutive lines
				*/
				let span = cursor.span_from(&start);
				return Some(Node::from(Token::Indent(cursor.indent()), Some(span)));
			}

			// apply registered matchers, those have higher priority
			let start = cursor.clone();
			for it in self.matchers.iter() {
				if let Some(node) = it.try_match(cursor, errors) {
					assert!(cursor.offset() > start.offset());
					return Some(node);
				} else if !errors.empty() {
					return None;
				} else {
					*cursor = start.clone();
				}
			}

			// match using the symbol table
			let (size, action) = self.table.recognize(cursor.data());
			let (size, action) = if let Some(action) = action {
				assert!(size > 0);
				(size, action.clone())
			} else {
				let size = char_size(cursor.data());
				(size, ScanAction::None)
			};
			cursor.advance(size);

			break match action {
				// no match or explicitly invalid match
				ScanAction::None | ScanAction::WordNext => {
					let span = cursor.span_from(&start);
					errors.add_at("invalid symbol", Some(span));
					None
				}

				// first character in an identifier
				ScanAction::Word => {
					// continue matching the entire identifier
					while let (size, Some(ScanAction::Word | ScanAction::WordNext)) =
						self.table.recognize(cursor.data())
					{
						cursor.advance(size);
					}

					// generate a Name token
					let name = cursor.data_from(&start);
					let name = String::from_utf8(name.to_vec()).unwrap();
					let span = cursor.span_from(&start);
					Some(Node::from(Token::Word(compiler.get_name(name)), Some(span)))
				}

				// predefined symbol
				ScanAction::Symbol(name) => {
					let span = cursor.span_from(&start);
					Some(Node::from(Token::Symbol(compiler.get_name(name)), Some(span)))
				}
			};
		}
	}
}

#[derive(Clone, Eq, PartialEq)]
pub enum ScanAction {
	None,
	Word,
	WordNext,
	Symbol(String),
}
