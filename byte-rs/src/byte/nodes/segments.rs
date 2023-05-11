use super::*;

pub struct Segment {
	span: Span,
	kind: SegmentKind,
}

has_traits!(Segment: IsNode);

impl IsNode for Segment {}

impl PartialEq for Segment {
	fn eq(&self, other: &Self) -> bool {
		self.span == other.span && self.kind == other.kind
	}
}

impl Segment {
	pub fn text(span: Span) -> Node {
		Node::from(Self {
			span,
			kind: SegmentKind::Text,
		})
	}

	pub fn comment(span: Span) -> Node {
		Node::from(Self {
			span,
			kind: SegmentKind::Comment,
		})
	}

	pub fn group(start: Span, span: Span, end: Span) -> Node {
		Node::from(Self {
			span,
			kind: SegmentKind::Group(start, end),
		})
	}

	pub fn parse(input: &mut Cursor) -> Option<Node> {
		let start = input.clone();
		let mut end = input.clone();
		while let Some(next) = input.read() {
			if next == '\n' {
				break;
			} else {
				end = input.clone();
			}
		}

		if end.offset() > start.offset() {
			let span = Span::from(&start, &end);
			let text = Self::text(span);
			Some(text)
		} else {
			None
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
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		write!(output, "<")?;
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

		Node::output_source(output, &self.span, ": ")?;

		if let Some(end) = end {
			write!(output, "{end}")?;
		}
		write!(output, ">")?;

		Ok(())
	}
}

//====================================================================================================================//
// Node extensions
//====================================================================================================================//

impl Node {
	pub fn output_source(
		output: &mut Repr<'_>,
		span: &Span,
		separator: &str,
	) -> std::io::Result<()> {
		let output = &mut output.indented();
		let lines = Self::output_source_and_location(output, span, separator)?;
		if lines > 1 {
			write!(output, "\n")?;
		}
		Ok(())
	}

	pub fn output_source_and_location(
		output: &mut Repr<'_>,
		span: &Span,
		separator: &str,
	) -> std::io::Result<usize> {
		let from = span.location();
		let name = span.input().name();
		let text = span.text().split('\n').collect::<Vec<_>>();
		let line = span.location().line().unwrap_or_default();
		let has_pos = line > 0 || name.is_some();

		let lines = if text.len() <= 1 {
			write!(output, "{separator}")?;
			Self::output_source_text(output, span)?;
			if has_pos {
				from.output_location(output, name, " # ")?;
			}
			1
		} else {
			if let Some(name) = name {
				write!(output, "# {name}")?;
			}
			write!(output, "\n")?;
			Self::output_source_text(output, span)?;
			text.len()
		};
		Ok(lines)
	}

	pub fn output_source_text(output: &mut Repr<'_>, span: &Span) -> std::io::Result<()> {
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
		check(input, vec![text("line 1"), text("line 2"), text("line 3")])
	}

	fn check(input: Vec<&'static str>, expected: Vec<Node>) {
		let actual = parse(input);
		assert_eq!(actual, expected);
	}

	fn parse(input: Vec<&'static str>) -> Vec<Node> {
		let text = input.join("\n");
		let text = Input::new("test.in", text);
		let mut nodes = Vec::new();

		let mut cursor = text.cursor();
		while let Some(node) = Segment::parse(&mut cursor) {
			nodes.push(node);
		}
		nodes
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Helpers
	//----------------------------------------------------------------------------------------------------------------//

	fn text(text: &'static str) -> Node {
		Segment::text(span(text))
	}

	fn _comment(text: &'static str) -> Node {
		Segment::comment(span(text))
	}

	fn _group(start: &'static str, text: &'static str, end: &'static str) -> Node {
		Segment::group(span(start), span(text), span(end))
	}

	fn span(text: &'static str) -> Span {
		Input::from(text).span().without_line()
	}
}
