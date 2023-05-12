use super::*;

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

	pub fn parse(input: &mut Cursor) -> Option<Segment> {
		input.skip_spaces();
		match input.peek() {
			Some('\n') | None => None,
			Some(next) => {
				let start = input.clone();
				let mut end;

				let kind = if next == '#' {
					//--------------------------------------------------------//
					// Comment
					//--------------------------------------------------------//

					input.read();
					let (multi, mut level) = if input.try_read('(') {
						(true, 1)
					} else {
						(false, 0)
					};

					end = input.clone();
					loop {
						match input.read() {
							Some('\n') if !multi => break,
							Some(c) => {
								end = input.clone();
								if multi {
									if c == '(' {
										level += 1;
									} else if c == ')' {
										level -= 1;
										if level == 0 {
											break;
										}
									}
								}
								input.skip_spaces();
							}
							None => break,
						}
					}
					SegmentKind::Comment
				} else {
					//--------------------------------------------------------//
					// Text
					//--------------------------------------------------------//

					end = input.clone();
					while let Some(next) = input.read() {
						if next == '\n' || next == '#' {
							break;
						} else {
							end = input.clone();
							input.skip_spaces();
						}
					}
					SegmentKind::Text
				};

				if end.offset() > start.offset() {
					let span = Span::from(&start, &end);
					*input = end;
					Some(Self { kind, span })
				} else {
					None
				}
			}
		}
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

	pub fn parse(input: &mut Cursor) -> Option<Line> {
		// skip blank line and spaces
		input.skip_while(|c| c == '\n' || is_space(c));

		let indent = input.location().indent();

		let mut items = Vec::new();
		while let Some(segment) = Segment::parse(input) {
			items.push(segment);
			if input.try_read('\n') {
				break;
			}
		}

		if items.len() > 0 {
			Some(Line { indent, items })
		} else {
			None
		}
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

	//================================================================================================================//
	// Test harness
	//================================================================================================================//

	fn check(input: Vec<&'static str>, expected: Vec<Node>) {
		let actual = parse(input);
		if actual != expected {
			let mut output = std::io::stderr().lock();
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

	fn parse(input: Vec<&'static str>) -> Vec<Node> {
		let text = input.join("\n");
		let text = Input::new("test.in", text);
		let mut nodes = Vec::new();

		let mut cursor = text.cursor();
		while let Some(node) = Line::parse(&mut cursor) {
			nodes.push(Node::from(node));
		}
		nodes
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Helpers
	//----------------------------------------------------------------------------------------------------------------//

	fn line(line: &'static str) -> Line {
		Line::new(0, vec![text(line)])
	}

	fn text(text: &'static str) -> Segment {
		Segment::text(span(text))
	}

	fn comment(text: &'static str) -> Segment {
		Segment::comment(span(text))
	}

	fn _group(start: &'static str, text: &'static str, end: &'static str) -> Segment {
		Segment::group(span(start), span(text), span(end))
	}

	fn span(text: &'static str) -> Span {
		Input::from(text).span().without_line()
	}
}
