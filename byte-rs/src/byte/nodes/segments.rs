use std::collections::VecDeque;

use super::*;

use crate::lexer::SymbolTable;

//====================================================================================================================//
// Line
//====================================================================================================================//

#[derive(Eq, PartialEq)]
pub struct Line {
	indent: usize,
	items: Vec<Segment>,
}

has_traits!(Line: IsNode);

impl IsNode for Line {}

impl Line {
	pub fn new(indent: usize, items: Vec<Segment>) -> Line {
		Line { indent, items }
	}

	pub fn span(&self) -> Span {
		let start = self.items.first().unwrap().span();
		let end = self.items.last().unwrap().span();
		Span::merge(start, end)
	}
}

impl HasRepr for Line {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		write!(output, "<Line")?;
		if !output.is_compact() {
			let output = &mut output.indented();
			for it in self.items.iter() {
				write!(output, "\n")?;
				it.output_repr(output)?;
			}
			write!(output, "\n")?;
		} else {
			for it in self.items.iter() {
				write!(output, " ")?;
				it.output_repr(output)?;
			}
		}
		write!(output, ">")?;
		Ok(())
	}
}

//====================================================================================================================//
// Segment
//====================================================================================================================//

#[derive(Eq, PartialEq)]
pub struct Segment {
	span: Span,
	kind: SegmentKind,
}

has_traits!(Segment: IsNode);

impl IsNode for Segment {}

impl Segment {
	pub fn text(span: Span) -> Segment {
		Self {
			span: span.clone(),
			kind: SegmentKind::Text,
		}
	}

	pub fn comment(span: Span) -> Segment {
		Self {
			span: span.clone(),
			kind: SegmentKind::Comment,
		}
	}

	pub fn group(start: Span, span: Span, end: Span) -> Segment {
		Self {
			span: span.clone(),
			kind: SegmentKind::Group(start, end),
		}
	}

	pub fn span(&self) -> &Span {
		&self.span
	}
}

#[derive(Eq, PartialEq)]
pub enum SegmentKind {
	Text,
	Comment,
	Group(Span, Span),
}

impl HasRepr for Segment {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		let output = &mut output.compact();
		write!(output, "(")?;
		let end = match &self.kind {
			SegmentKind::Text => {
				write!(output, "Text")?;
				None
			}
			SegmentKind::Comment => {
				write!(output, "Comment")?;
				None
			}
			SegmentKind::Group(start, end) => {
				write!(output, "Group{start}")?;
				Some(end)
			}
		};

		Node::output_source(output, &self.span, " ")?;

		if let Some(end) = end {
			write!(output, "{end}")?;
		}
		write!(output, ")")?;

		Ok(())
	}
}

//====================================================================================================================//
// Parser implementation
//====================================================================================================================//

#[derive(Clone)]
pub struct SegmentParser {
	comment: char,
	multi_comment_s: char,
	multi_comment_e: char,
	brackets: SymbolTable<Bracket>,
	errors: Errors,
}

impl SegmentParser {
	pub fn new() -> Self {
		let mut result = Self {
			comment: '#',
			multi_comment_s: '(',
			multi_comment_e: ')',
			brackets: Default::default(),
			errors: Errors::new(),
		};
		result.add_brackets("(", ")");
		result.add_brackets("[", "]");
		result.add_brackets("{", "}");
		result
	}

	pub fn add_brackets(&mut self, sta: &'static str, end: &'static str) {
		self.brackets.add_symbol(sta, Bracket(sta, end, true));
		self.brackets.add_symbol(end, Bracket(sta, end, false));
	}

	pub fn parse(&mut self, cursor: &mut Cursor) -> Option<Node> {
		self.parse_line(cursor).map(|line| {
			let span = line.span();
			Node::from(line).with_span(span)
		})
	}

	pub fn has_errors(&self) -> bool {
		!self.errors.empty()
	}

	pub fn errors(&self) -> Errors {
		self.errors.clone()
	}

	fn add_error(&mut self, error: Value) {
		if self.errors.len() < MAX_ERRORS {
			self.errors.add(error);
		}
	}

	fn parse_line(&mut self, cursor: &mut Cursor) -> Option<Line> {
		// helper to push a text segment to the list
		let push_text = |items: &mut Vec<Segment>, start: &Cursor, end: &Cursor| {
			if end.offset() > start.offset() {
				let span = Span::from(&start, &end);
				items.push(Segment {
					kind: SegmentKind::Text,
					span,
				});
			}
		};

		// skip blank line and spaces
		cursor.skip_while(|c| c == '\n' || is_space(c));

		// save the line indentation
		let indent = cursor.location().indent();

		let mut items = Vec::new();
		let mut start = cursor.clone();
		let mut end = cursor.clone();

		loop {
			if self.has_errors() {
				break;
			}

			if let Some(segment) = self.parse_non_text_segment(cursor) {
				// push any preceding text segment
				push_text(&mut items, &start, &end);

				// add parsed segment
				items.push(segment);

				// reset text segment
				cursor.skip_spaces();
				start = cursor.clone();
				end = cursor.clone();
			} else {
				// read the next char in the text segment
				match cursor.read() {
					Some('\n') | None => break,
					Some(..) => {
						end = cursor.clone();
					}
				}
			}
		}

		if self.has_errors() {
			cursor.skip_while(|c| c != '\n');
		}

		// push any trailing text segment
		push_text(&mut items, &start, &end);

		if items.len() > 0 {
			Some(Line { indent, items })
		} else {
			None
		}
	}

	fn parse_non_text_segment(&mut self, cursor: &mut Cursor) -> Option<Segment> {
		cursor.skip_spaces();

		let start = cursor.clone();
		if self.read_comment(cursor) {
			Some(Segment {
				kind: SegmentKind::Comment,
				span: Span::from(&start, cursor),
			})
		} else if let Some(segment) = self.parse_group(cursor) {
			Some(segment)
		} else {
			None
		}
	}

	fn parse_group(&mut self, cursor: &mut Cursor) -> Option<Segment> {
		let mut stack = if let Some((bracket, span)) = self.brackets.parse_with_span(cursor) {
			if !bracket.is_opening() {
				self.add_error(format!("unexpected unmatched `{bracket}`").at(span));
				return None;
			}
			let mut stack = VecDeque::new();
			stack.push_back((bracket, span));
			stack
		} else {
			return None;
		};

		let start = cursor.clone();
		loop {
			self.skip_blanks(cursor);

			let end = cursor.clone();
			if let Some((bracket, span)) = self.brackets.parse_with_span(cursor) {
				let (open, ..) = stack.back().unwrap();
				if bracket.closes(open) {
					let (_, open_span) = stack.pop_back().unwrap();
					if stack.is_empty() {
						let content = Span::from(&start, &end);
						let segment = Segment::group(open_span, content, span);
						return Some(segment);
					}
				} else if bracket.is_opening() {
					stack.push_back((bracket, span));
				} else {
					self.add_error(format!("unmatched closing `{bracket}`").at(span));
				}
			} else if cursor.read().is_none() {
				break;
			}
		}

		let (bracket, span) = stack.pop_back().unwrap();
		self.add_error(
			format!(
				"the opening `{bracket}` expected a `{}`{}, but found nothing",
				bracket.closing(),
				cursor.location().format(" at "),
			)
			.at(span),
		);
		None
	}

	fn skip_blanks(&mut self, cursor: &mut Cursor) {
		let skip = |cursor: &mut Cursor| cursor.skip_while(|c| c == '\n' || is_space(c));
		skip(cursor);
		while self.read_comment(cursor) {
			skip(cursor)
		}
	}

	fn read_comment(&mut self, cursor: &mut Cursor) -> bool {
		let next = if let Some(next) = cursor.peek() {
			next
		} else {
			return false;
		};

		if next != self.comment {
			false
		} else {
			cursor.read();
			let (multi, mut level) = if cursor.try_read(self.multi_comment_s) {
				(true, 1)
			} else {
				(false, 0)
			};

			let mut end = cursor.clone();
			loop {
				match cursor.read() {
					Some('\n') if !multi => break,
					Some(c) => {
						end = cursor.clone();
						if multi {
							if c == self.multi_comment_s {
								level += 1;
							} else if c == self.multi_comment_e {
								level -= 1;
								if level == 0 {
									break;
								}
							}
						}
						cursor.skip_spaces();
					}
					None => break,
				}
			}

			*cursor = end;
			true
		}
	}
}

#[derive(Copy, Clone)]
struct Bracket(&'static str, &'static str, bool);

impl Bracket {
	pub fn is_opening(&self) -> bool {
		self.2
	}

	pub fn symbol(&self) -> &'static str {
		if self.is_opening() {
			self.0
		} else {
			self.1
		}
	}

	pub fn closing(&self) -> &'static str {
		self.1
	}

	pub fn closes(&self, start: &Bracket) -> bool {
		!self.is_opening() && start.1 == self.1
	}
}

impl std::fmt::Display for Bracket {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.symbol())
	}
}

//====================================================================================================================//
// Node extensions
//====================================================================================================================//

impl Node {
	pub fn output_source(output: &mut Repr, span: &Span, separator: &str) -> std::io::Result<()> {
		let output = &mut output.indented();
		let lines = Self::output_source_and_location(output, span, separator)?;
		if lines > 1 {
			write!(output, "\n")?;
		}
		Ok(())
	}

	pub fn output_source_and_location(
		output: &mut Repr,
		span: &Span,
		separator: &str,
	) -> std::io::Result<usize> {
		let from = span.location();
		let name = if output.is_full() {
			span.input().name()
		} else {
			None
		};
		let text = span.text().split('\n').collect::<Vec<_>>();
		let line = span.location().line().unwrap_or_default();
		let has_pos = line > 0 || name.is_some();

		let lines = if text.len() <= 1 {
			write!(output, "{separator}`")?;
			Self::output_source_text(output, span)?;
			write!(output, "`")?;
			if has_pos {
				from.output_location(output, name, " @")?;
			}
			1
		} else {
			if let Some(name) = name {
				write!(output, "#{name}")?;
			}
			write!(output, "\n")?;
			Self::output_source_text(output, span)?;
			text.len()
		};
		Ok(lines)
	}

	pub fn output_source_text(output: &mut Repr, span: &Span) -> std::io::Result<()> {
		let text = span.text().split('\n').collect::<Vec<_>>();
		let line = span.location().line().unwrap_or_default();
		if text.len() <= 1 {
			let text = text.first().cloned().unwrap_or("");
			if text.len() == 0 {
				write!(output, "(empty)")?;
			} else {
				write!(output, "{text}")?;
			}
		} else {
			for (i, it) in text.iter().enumerate() {
				if i > 0 {
					write!(output, "\n")?;
				}
				if line > 0 {
					write!(output, "{:03}: ", line + i)?;
				}
				write!(output, "{it}")?;
			}
		}

		Ok(())
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty() {
		let input = vec![];
		check(input, vec![]);
	}

	#[test]
	fn lines() {
		let input = vec!["line 1", "line 2", "line 3"];
		check(
			input,
			vec![
				line("line 1").into(),
				line("line 2").into(),
				line("line 3").into(),
			],
		);

		let input = vec!["", "line 1", "", "line 2", "    ", "\t", "line 3", ""];
		check(
			input,
			vec![
				line("line 1").into(),
				line("line 2").into(),
				line("line 3").into(),
			],
		);
	}

	#[test]
	fn indented_lines() {
		let input = vec!["line 1", "  line 2", "  line 3", "    line 4", "line 5"];
		check(
			input,
			vec![
				Line::new(0, vec![text("line 1")]).into(),
				Line::new(2, vec![text("line 2")]).into(),
				Line::new(2, vec![text("line 3")]).into(),
				Line::new(4, vec![text("line 4")]).into(),
				Line::new(0, vec![text("line 5")]).into(),
			],
		)
	}

	#[test]
	fn comments() {
		let input = vec!["# comment 1", "line 1"];
		check(
			input,
			vec![
				Line::new(0, vec![comment("# comment 1")]).into(),
				line("line 1").into(),
			],
		);

		let input = vec![
			"# comment 1",
			"line 1",
			"",
			"#(",
			"comment 2",
			")",
			"line 2",
			"",
		];
		check(
			input,
			vec![
				Line::new(0, vec![comment("# comment 1")]).into(),
				line("line 1").into(),
				Line::new(0, vec![comment("#(\ncomment 2\n)")]).into(),
				line("line 2").into(),
			],
		);

		let input = vec!["# comment 1", "#(c1)A#(c2)B #((c3)) C D # comment 2 "];
		check(
			input,
			vec![
				Line::new(0, vec![comment("# comment 1")]).into(),
				Line::new(
					0,
					vec![
						comment("#(c1)"),
						text("A"),
						comment("#(c2)"),
						text("B"),
						comment("#((c3))"),
						text("C D"),
						comment("# comment 2"),
					],
				)
				.into(),
			],
		);
	}

	#[test]
	fn groups() {
		let input = vec!["(1)"];
		check(input, vec![Line::new(0, vec![group("(", "1", ")")]).into()]);

		let input = vec!["(a)", "[ 1 2 ]", "{", "    content", "}"];
		check(
			input,
			vec![
				Line::new(0, vec![group("(", "a", ")")]).into(),
				Line::new(0, vec![group("[", " 1 2 ", "]")]).into(),
				Line::new(0, vec![group("{", "\n    content\n", "}")]).into(),
			],
		);

		let input = vec!["1 + (2 + 3) * 4"];
		check(
			input,
			vec![Line::new(0, vec![text("1 +"), group("(", "2 + 3", ")"), text("* 4")]).into()],
		);

		let input = vec!["(([{1}]))"];
		check(
			input,
			vec![Line::new(0, vec![group("(", "([{1}])", ")")]).into()],
		);

		let content = vec![
			" # comment 1",
			"    1",
			"    #((",
			"      comment 2",
			"    ))",
			"    [ 2, 3 ]",
			"   # A) comment",
			"  4",
			"  ",
		]
		.join("\n");
		let content = content.as_str();

		let input = format!("x + ({content}) + y # end");
		let input = vec![input.as_str()];
		check(
			input,
			vec![Line::new(
				0,
				vec![
					text("x +"),
					group("(", content, ")"),
					text("+ y"),
					comment("# end"),
				],
			)
			.into()],
		);
	}

	//================================================================================================================//
	// Test harness
	//================================================================================================================//

	fn check(input: Vec<&str>, expected: Vec<Node>) {
		let actual = parse(input);
		if actual != expected {
			let mut output = test_output();
			let mut output = Repr::new(&mut output, ReprMode::Debug, ReprFormat::Full);

			let _ = write!(output, "\nExpected:\n");
			{
				let output = &mut output.indented();
				for (i, it) in expected.iter().enumerate() {
					let _ = write!(output, "\n[{i}] ");

					let mut output = output.indented().compact();
					let _ = it.output_repr(&mut output);
				}
				let _ = write!(output, "\n");
			}

			let _ = write!(output, "\nActual:\n");
			{
				let output = &mut output.indented();
				for (i, it) in actual.iter().enumerate() {
					let _ = write!(output, "\n[{i}] ");

					let mut output = output.indented().compact();
					let _ = it.output_repr(&mut output);
				}
				let _ = write!(output, "\n");
			}
			let _ = write!(output, "\n");
		}
		assert_eq!(actual, expected);
	}

	fn parse(input: Vec<&str>) -> Vec<Node> {
		let text = input.join("\n");
		let text = Input::new("test.in", text);
		let mut nodes = Vec::new();

		let mut parser = SegmentParser::new();
		let mut cursor = text.cursor();
		while let Some(node) = parser.parse(&mut cursor) {
			nodes.push(node);
		}

		if parser.has_errors() {
			let mut output = test_output();
			let mut output = Repr::new(&mut output, ReprMode::Display, ReprFormat::Full);
			let _ = parser.errors().output_repr(&mut output);
			let _ = write!(output, "\n");
			panic!("Segment parsing generated errors");
		}

		nodes
	}

	fn test_output() -> impl std::io::Write {
		std::io::stderr().lock()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Helpers
	//----------------------------------------------------------------------------------------------------------------//

	fn line(line: &str) -> Line {
		Line::new(0, vec![text(line)])
	}

	fn text(text: &str) -> Segment {
		Segment::text(span(text))
	}

	fn comment(text: &str) -> Segment {
		Segment::comment(span(text))
	}

	fn group(start: &str, text: &str, end: &str) -> Segment {
		Segment::group(span(start), span(text), span(end))
	}

	fn span(text: &str) -> Span {
		Input::from(text.to_string()).span().without_line()
	}
}
