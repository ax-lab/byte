use super::*;

/// Trait for a matcher that can be used by the [`Scanner`].
pub trait Matcher: Cell {
	fn try_match(&self, cursor: &mut Span, errors: &mut Errors) -> Option<Node>;
}

#[derive(Clone)]
pub struct Scanner {
	matchers: Arc<Vec<Arc<dyn Matcher>>>,
	table: Arc<SymbolTable<ScanAction>>,
}

impl Scanner {
	pub fn new() -> Self {
		Self {
			matchers: Default::default(),
			table: Default::default(),
		}
	}

	pub fn add_matcher<T: Matcher + 'static>(&mut self, matcher: T) {
		let matchers = Arc::make_mut(&mut self.matchers);
		matchers.push(Arc::new(matcher));
	}

	pub fn add_symbol(&mut self, symbol: &str) {
		let table = Arc::make_mut(&mut self.table);
		table.add(symbol, ScanAction::Symbol(symbol.to_string()));
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

	pub fn scan(&self, cursor: &mut Span, errors: &mut Errors) -> Option<Node> {
		loop {
			// skip spaces
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
				// ignore empty or space-only lines
				let span = cursor.advance_span(size);
				if !line_start {
					return Some(Bit::Break.at(span));
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
				return Some(Bit::Indent(cursor.indent()).at(cursor.clone()));
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
			let span = cursor.advance_span(size);

			break match action {
				// no match or explicitly invalid match
				ScanAction::None | ScanAction::WordNext => {
					errors.add("invalid symbol", span);
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

					// generate a Word token
					let span = cursor.span_from(&start);
					let symbol = span.text().to_string();
					Some(Bit::Word(Context::symbol(symbol)).at(span))
				}

				// predefined symbol
				ScanAction::Symbol(symbol) => Some(Bit::Symbol(Context::symbol(symbol)).at(span)),
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
