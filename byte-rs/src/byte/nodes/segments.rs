use super::*;

pub fn parse_segments(
	scanner: &Scanner,
	stream: &mut NodeStream,
	errors: &mut Errors,
) -> Vec<Node> {
	let block = Block::parse(scanner, stream, errors, 0);
	let result = if let Some(block) = block {
		block.items.into_iter().map(Node::from).collect()
	} else {
		Vec::new()
	};
	if errors.empty() {
		if let Some(next) = stream.peek() {
			errors.add("unparsed suffix at end of input".at_node(next));
		}
	}
	result
}

//====================================================================================================================//
// Block
//====================================================================================================================//

#[derive(Eq, PartialEq)]
pub struct Block {
	items: Vec<Segment>,
}

has_traits!(Block: IsNode);

impl Block {
	pub fn new<T: IntoIterator<Item = Segment>>(segments: T) -> Self {
		Self {
			items: segments.into_iter().collect(),
		}
	}

	fn parse(
		scanner: &Scanner,
		stream: &mut NodeStream,
		errors: &mut Errors,
		level: usize,
	) -> Option<Self> {
		let mut items = Vec::new();
		while let Some(next) = Segment::parse(scanner, stream, errors, level) {
			if !next.empty() {
				items.push(next);
			}
			stream.read_if(|x| x.is_symbol(";"));
			stream.skip_comments();
			stream.read_if(|x| x.is_break());
			if !errors.empty() {
				break;
			}
		}

		if items.len() > 0 {
			Some(Block { items })
		} else {
			None
		}
	}
}

impl IsNode for Block {}

impl HasRepr for Block {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		write!(output, "{{")?;

		{
			let output = &mut output.indented();
			for it in self.items.iter() {
				write!(output, "\n")?;
				it.output_repr(&mut output.compact())?;
			}
		}

		if self.items.len() > 0 {
			write!(output, "\n")?;
		} else {
			write!(output, " ")?;
		}
		write!(output, "}}")?;

		Ok(())
	}
}

//====================================================================================================================//
// Segment
//====================================================================================================================//

#[derive(Eq, PartialEq)]
pub struct Segment {
	line: NodeList,
	block: Option<Block>,
}

has_traits!(Segment: IsNode);

impl Segment {
	pub fn new<T: IntoIterator<Item = Node>>(line: T) -> Self {
		let line = NodeList::new(line);
		Self { line, block: None }
	}

	pub fn new_with_block<T: IntoIterator<Item = Node>>(line: T, block: Block) -> Self {
		let line = NodeList::new(line);
		Self {
			line,
			block: Some(block),
		}
	}

	pub fn line(&self) -> &NodeList {
		&self.line
	}

	pub fn block(&self) -> Option<&Block> {
		self.block.as_ref()
	}

	pub fn empty(&self) -> bool {
		self.line.len() == 0 && self.block.is_none()
	}

	fn parse(
		scanner: &Scanner,
		stream: &mut NodeStream,
		errors: &mut Errors,
		level: usize,
	) -> Option<Self> {
		if let Some(next) = stream.peek() {
			if next.indent() < level || next.is_break() {
				return None;
			}
		} else {
			return None;
		}

		let mut block = None;
		let mut line = Vec::new();
		stream.skip_comments();
		while let Some(next) = stream.peek() {
			if next.is_symbol(";") {
				stream.undo();
				break;
			} else if next.is_break() {
				if let Some(next) = stream.lookahead(1) {
					let next_level = next.indent();
					if next_level > level {
						stream.read();
						block = Block::parse(scanner, stream, errors, next_level);
					}
				}
				break;
			} else if let Some(next) = Group::parse(scanner, stream, errors, level) {
				line.push(Node::from(next));
			} else {
				line.push(stream.read().unwrap());
			}

			if !errors.empty() {
				break;
			}

			stream.skip_comments();
		}

		let line = NodeList::new(line);
		Some(Segment { line, block })
	}
}

impl IsNode for Segment {}

impl HasRepr for Segment {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		let multiline = self.line.range(..).multiline();

		write!(output, "<")?;
		let mut output = if multiline {
			output.indented()
		} else {
			output.clone()
		};

		if self.line.len() > 0 {
			self.line.output_repr(&mut output.compact())?;
		} else {
			write!(output, "(empty line)")?;
		}

		let multiline = if let Some(ref block) = self.block {
			let output = &mut output.indented();
			write!(output, "\n")?;
			block.output_repr(output)?;
			true
		} else {
			multiline
		};

		if multiline {
			write!(output, "\n")?;
		}
		write!(output, ">")?;

		Ok(())
	}
}

//====================================================================================================================//
// Group
//====================================================================================================================//

#[derive(Eq, PartialEq)]
pub struct Group {
	sta: Node,
	end: Node,
	content: NodeList,
}

has_traits!(Group: IsNode);

impl Group {
	pub fn new(sta: Node, content: NodeList, end: Node) -> Self {
		Self { sta, end, content }
	}

	fn parse(
		scanner: &Scanner,
		stream: &mut NodeStream,
		errors: &mut Errors,
		level: usize,
	) -> Option<Self> {
		if let Some(end_symbol) = stream
			.peek()
			.and_then(|token| token.symbol())
			.and_then(|symbol| scanner.closing_bracket_for(symbol))
		{
			let sta = stream.read().unwrap();
			let mut end = None;
			let mut content = Vec::new();
			while let Some(next) = stream.peek() {
				let indent = next.indent();
				if indent < level {
					errors.add("unexpected dedent".at_node(&next));
					stream.undo();
					break;
				}

				if next.is_symbol(end_symbol) {
					end = stream.read();
					break;
				} else if let Some(group) = Group::parse(scanner, stream, errors, indent) {
					content.push(Node::from(group));
				} else {
					content.push(stream.read().unwrap());
				}
			}

			let end = if let Some(end) = end {
				end
			} else {
				errors.add(
					format!(
						"`{sta}`{} expected `{end_symbol}`",
						sta.format_location(" from ")
					)
					.maybe_at(stream.pos()),
				);
				Node::empty()
			};

			let content = NodeList::new(content);
			let group = Self { sta, end, content };
			Some(group)
		} else {
			None
		}
	}
}

impl IsNode for Group {}

impl HasRepr for Group {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		let multiline = self.content.range(..).multiline();

		write!(output, "{}", self.sta)?;
		if multiline {
			write!(output, "\n")?;
		}

		self.content.output_repr(&mut output.indented())?;

		if multiline {
			write!(output, "\n")?;
		}
		write!(output, "{}", self.end)?;

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
				line(vec![id("line"), n(1)]),
				line(vec![id("line"), n(2)]),
				line(vec![id("line"), n(3)]),
			],
		);

		let input = vec!["", "line 1", "", "line 2", "    ", "\t", "line 3", ""];
		check(
			input,
			vec![
				line(vec![id("line"), n(1)]),
				line(vec![id("line"), n(2)]),
				line(vec![id("line"), n(3)]),
			],
		);
	}

	#[test]
	fn indented_lines() {
		let input = vec!["line 1", "  line 2", "  line 3", "    line 4", "line 5"];
		check(
			input,
			vec![
				Node::from(segment_block(
					vec![id("line"), n(1)],
					block(vec![
						segment(vec![id("line"), n(2)]),
						segment_block(
							vec![id("line"), n(3)],
							block(vec![segment(vec![id("line"), n(4)])]),
						),
					]),
				)),
				line(vec![id("line"), n(5)]),
			],
		)
	}

	#[test]
	fn comments() {
		let input = vec!["# comment 1", "line 1"];
		check(input, vec![line(vec![id("line"), n(1)])]);

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
			vec![line(vec![id("line"), n(1)]), line(vec![id("line"), n(2)])],
		);

		let input = vec!["# comment 1", "#(c1)A#(c2)B #((c3)) C D # comment 2 "];
		check(input, vec![line(vec![id("A"), id("B"), id("C"), id("D")])]);
	}

	#[test]
	fn groups() {
		let input = vec!["(1)"];
		check(input, vec![line(vec![group("(", vec![n(1)], ")")])]);

		let input = vec!["(a)", "[ 1 2 ]", "{", "    content", "}"];
		check(
			input,
			vec![
				line(vec![group("(", vec![id("a")], ")")]),
				line(vec![group("[", vec![n(1), n(2)], "]")]),
				line(vec![group("{", vec![brk(), id("content"), brk()], "}")]),
			],
		);

		let input = vec!["1 + (2 + 3) * 4"];
		check(
			input,
			vec![line(vec![
				n(1),
				s("+"),
				group("(", vec![n(2), s("+"), n(3)], ")"),
				s("*"),
				n(4),
			])],
		);

		let input = vec!["(([{1}]))"];
		check(
			input,
			vec![line(vec![group(
				"(",
				vec![group(
					"(",
					vec![group("[", vec![group("{", vec![n(1)], "}")], "]")],
					")",
				)],
				")",
			)])],
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

		let input = format!("x + ({content}) + y # end");
		let content_nodes = vec![
			comment("# comment 1"),
			brk(),
			n(1),
			brk(),
			comment("#((\n      comment 2\n    ))"),
			brk(),
			group("[", vec![n(2), s(","), n(3)], "]"),
			brk(),
			comment("# A) comment"),
			brk(),
			n(4),
			brk(),
		];

		let input = vec![input.as_str()];
		check(
			input,
			vec![line(vec![
				id("x"),
				s("+"),
				group("(", content_nodes, ")"),
				s("+"),
				id("y"),
			])],
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

		let mut scanner = Scanner::new();
		scanner.add_bracket_pair("(", ")");
		scanner.add_bracket_pair("[", "]");
		scanner.add_bracket_pair("{", "}");
		scanner.add_symbol(",", Token::Symbol(","));
		scanner.add_symbol(";", Token::Symbol(";"));
		scanner.add_symbol(":", Token::Symbol(":"));
		scanner.add_symbol("(", Token::Symbol("("));
		scanner.add_symbol("[", Token::Symbol("["));
		scanner.add_symbol("{", Token::Symbol("{"));
		scanner.add_symbol("}", Token::Symbol("}"));
		scanner.add_symbol("]", Token::Symbol("]"));
		scanner.add_symbol(")", Token::Symbol(")"));
		scanner.add_symbol("+", Token::Symbol("+"));
		scanner.add_symbol("-", Token::Symbol("-"));
		scanner.add_symbol("*", Token::Symbol("*"));
		scanner.add_symbol("/", Token::Symbol("/"));
		scanner.add_matcher(IntegerMatcher);
		scanner.add_matcher(IdentifierMatcher);
		scanner.add_matcher(CommentMatcher);

		let mut errors = Errors::new();
		let nodes = NodeList::tokenize(text, &mut scanner, &mut errors);

		let nodes = if errors.empty() {
			let mut stream = nodes.into_iter();
			parse_segments(&scanner, &mut stream, &mut errors)
		} else {
			Vec::new()
		};

		if errors.len() > 0 {
			let mut output = test_output();
			let mut output = Repr::new(&mut output, ReprMode::Display, ReprFormat::Full);
			let _ = errors.output_repr(&mut output);
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

	fn line<T: IntoIterator<Item = Node>>(line: T) -> Node {
		segment(line).into()
	}

	fn segment<T: IntoIterator<Item = Node>>(line: T) -> Segment {
		Segment::new(line)
	}

	fn segment_block<T: IntoIterator<Item = Node>>(line: T, block: Block) -> Segment {
		Segment::new_with_block(line, block)
	}

	fn block<T: IntoIterator<Item = Segment>>(segments: T) -> Block {
		Block::new(segments)
	}

	fn id(text: &str) -> Node {
		Token::Word(span(text)).into()
	}

	fn brk() -> Node {
		Token::Break.into()
	}

	fn n(value: usize) -> Node {
		Integer(value as u128).into()
	}

	fn s(symbol: &'static str) -> Node {
		Token::Symbol(symbol).into()
	}

	fn comment(text: &str) -> Node {
		Comment(span(text)).into()
	}

	fn group<T: IntoIterator<Item = Node>>(s: &'static str, nodes: T, e: &'static str) -> Node {
		let s = Node::from(Token::Symbol(s));
		let e = Node::from(Token::Symbol(e));
		Group::new(s, NodeList::new(nodes), e).into()
	}

	fn span(text: &str) -> Span {
		Input::from(text.to_string()).span().without_line()
	}
}
