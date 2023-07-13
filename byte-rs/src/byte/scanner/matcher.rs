use super::*;

#[derive(Clone)]
pub struct Matcher {
	matchers: Arc<Vec<Arc<dyn IsMatcher>>>,
	table: Arc<SymbolTable<ScanAction>>,
	word_chars: Arc<HashSet<char>>,
}

impl Matcher {
	pub fn new() -> Self {
		Self {
			matchers: Default::default(),
			table: Default::default(),
			word_chars: Default::default(),
		}
	}

	pub fn add_matcher<T: IsMatcher + 'static>(&mut self, matcher: T) {
		let matchers = Arc::make_mut(&mut self.matchers);
		matchers.push(Arc::new(matcher));
	}

	pub fn add_symbol(&mut self, symbol: &str) {
		assert!(symbol.len() > 0);
		let table = Arc::make_mut(&mut self.table);
		let is_word = self.word_chars.contains(&symbol.chars().next().unwrap());
		table.add(symbol, ScanAction::Symbol(symbol.to_string(), is_word));
	}

	pub fn add_word_chars(&mut self, chars: &str) {
		let word_chars = Arc::make_mut(&mut self.word_chars);
		for char in chars.chars() {
			word_chars.insert(char);
		}
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

	pub fn scan(&mut self, cursor: &mut Span, errors: &mut Errors) -> Option<(Token, Span)> {
		loop {
			// skip spaces
			let line_start = cursor.is_indent();
			self.skip_blank(cursor, false);

			// check for a line break
			if let Some(size) = check_line_break(cursor.data()) {
				assert!(size > 0);
				// ignore empty or space-only lines
				let span = cursor.advance_span(size);
				if !line_start {
					// get the next line indentation
					self.skip_blank(cursor, true);
					let indent = cursor.indent();
					return Some((Token::Break(indent), span));
				} else {
					continue;
				}
			}

			// check for the end of input
			if cursor.at_end() || !errors.empty() {
				return None;
			}

			/*
				TODO: validate bad indentation scenarios

				- spaces followed by tabs
				- for any consecutive non-empty lines, one of the lines
				  indentation MUST be a prefix of the other
				  - indentation must be consistent between consecutive lines
			*/

			// apply registered matchers, those have higher priority
			let start = cursor.clone();
			for it in self.matchers.iter() {
				if let Some((token, span)) = it.try_match(cursor, errors) {
					assert!(cursor.offset() > start.offset());
					return Some((token, span));
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
					self.match_word_continuation(cursor);
					let span = cursor.span_from(&start);
					Some((Token::Word(Context::symbol(span.text())), span))
				}

				// predefined symbol
				ScanAction::Symbol(symbol, is_word) => {
					let span = cursor.span_from(&start);
					let token = if is_word && self.match_word_continuation(cursor) {
						Token::Word(Context::symbol(span.text()))
					} else {
						Token::Symbol(Context::symbol(symbol))
					};
					Some((token, span))
				}
			};
		}
	}

	fn match_word_continuation(&self, cursor: &mut Span) -> bool {
		let mut matched = false;
		while let (size, Some(ScanAction::Word | ScanAction::WordNext | ScanAction::Symbol(_, true))) =
			self.table.recognize(cursor.data())
		{
			matched = true;
			cursor.advance(size);
		}
		matched
	}

	fn skip_blank(&self, cursor: &mut Span, skip_breaks: bool) {
		let mut skipping = true;
		while skipping {
			skipping = false;

			// skip spaces
			while let Some((.., size)) = check_space(cursor.data()) {
				assert!(size > 0);
				cursor.advance(size);
				skipping = true;
			}

			// check for a line break
			if skip_breaks {
				if let Some(size) = check_line_break(cursor.data()) {
					assert!(size > 0);
					cursor.advance_span(size);
					skipping = true;
				}
			}
		}
	}
}

#[derive(Clone, Eq, PartialEq)]
pub enum ScanAction {
	None,
	Word,
	WordNext,
	Symbol(String, bool),
}
