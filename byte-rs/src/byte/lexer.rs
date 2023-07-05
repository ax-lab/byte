use super::*;

pub mod chars;
pub mod comment;
pub mod literal;
pub mod number;
pub mod scanner;
pub mod symbols;

pub use chars::*;
pub use comment::*;
pub use literal::*;
pub use number::*;
pub use scanner::*;
pub use symbols::*;

pub type Name = CompilerHandle<str>;

impl Compiler {
	pub fn get_name<T: AsRef<str>>(&self, name: T) -> Name {
		self.intern(name)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty() {
		let (actual, _) = tokenize("");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("    ");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("\n");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("\r");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("\r\n");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("\n    ");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("    \n");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("\t\n\n\r\n    \r    \n");
		assert_eq!(actual, Vec::new());
	}

	#[test]
	fn simple_scanning() {
		let (actual, compiler) = tokenize("a, b (\n\tsome_name123\n)");
		assert_eq!(
			actual,
			vec![
				word(&compiler, "a"),
				sym(&compiler, ","),
				word(&compiler, "b"),
				sym(&compiler, "("),
				eol(),
				indent(4),
				word(&compiler, "some_name123"),
				eol(),
				sym(&compiler, ")")
			]
		);
	}

	#[test]
	fn with_matchers() {
		let input = vec![
			"# some single line comment",
			"print 'hello world!'",
			"",
			"#(",
			"    this is a multiline comment (with nested parenthesis)",
			")",
			"",
			"print 1, 2, 3",
		];
		let (actual, compiler) = tokenize(input.join("\n").as_str());
		assert_eq!(
			actual,
			vec![
				comment(),
				eol(),
				word(&compiler, "print"),
				literal("hello world!"),
				eol(),
				comment(),
				eol(),
				word(&compiler, "print"),
				int(1),
				sym(&compiler, ","),
				int(2),
				sym(&compiler, ","),
				int(3),
			]
		)
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Helpers
	//----------------------------------------------------------------------------------------------------------------//

	fn tokenize(input: &str) -> (Vec<Node>, Compiler) {
		let compiler = Compiler::default();
		let mut scanner = Scanner::new(compiler.get_ref());
		scanner.register_common_symbols();
		scanner.add_matcher(CommentMatcher);
		scanner.add_matcher(LiteralMatcher);
		scanner.add_matcher(IntegerMatcher);

		let context = Context::get();
		let input = context.load_source_text("test", input);
		let mut cursor = input.span();
		let mut errors = Errors::new();
		let mut output = Vec::new();
		while let Some(node) = scanner.scan(&mut cursor, &mut errors) {
			output.push(node.to_inner());
		}

		if errors.len() > 0 {
			eprintln!("\n{errors}");
			panic!("Scanning generated errors");
		}

		assert!(cursor.at_end());
		(output, compiler)
	}

	fn word(compiler: &Compiler, name: &str) -> Node {
		Node::Word(compiler.get_name(name))
	}

	fn sym(compiler: &Compiler, name: &str) -> Node {
		Node::Symbol(compiler.get_name(name))
	}

	fn eol() -> Node {
		Node::Break
	}

	fn indent(width: usize) -> Node {
		Node::Indent(width)
	}

	fn comment() -> Node {
		Node::Comment
	}

	fn literal(str: &str) -> Node {
		Node::Literal(str.to_string())
	}

	fn int(value: u128) -> Node {
		Node::Integer(value)
	}
}
