use std::io::Write;

use crate::core::*;
use crate::lexer::*;

use super::*;

//====================================================================================================================//
// Statement
//====================================================================================================================//

#[derive(Eq, PartialEq)]
pub struct Statement {
	expr: Node,
	block: Option<Node>,
}

has_traits!(Statement: IsNode, WithSpan, WithEquality);

impl Statement {
	pub fn new(expr: Node, block: Option<Node>) -> Node {
		Node::from(Self { expr, block })
	}
}

impl IsNode for Statement {}

impl HasRepr for Statement {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		if output.is_debug() {
			write!(output, "<Statement ")?;
			self.expr.output_repr(&mut output.indented().compact())?;
			if let Some(block) = &self.block {
				let mut output = output.indented();
				write!(output, "\n")?;
				block.output_repr(&mut output)?;
			}
			write!(output, ">")?;
		} else {
			self.expr.output_repr(&mut output.compact())?;
			if let Some(block) = &self.block {
				let mut output = output.indented();
				write!(output, ":\n")?;
				block.output_repr(&mut output)?;
			}
		}
		Ok(())
	}
}

impl WithSpan for Statement {
	fn get_span(&self) -> Option<Span> {
		Span::from_range(self.expr.span(), self.block.as_ref().and_then(|x| x.span()))
	}
}

//====================================================================================================================//
// RawExpr
//====================================================================================================================//

#[derive(Eq, PartialEq)]
pub struct RawExpr {
	expr: Vec<Node>,
}

has_traits!(RawExpr: IsNode, WithSpan, WithEquality);

impl RawExpr {
	pub fn new(expr: Vec<Node>) -> Node {
		Node::from(Self { expr })
	}
}

impl IsNode for RawExpr {}

impl HasRepr for RawExpr {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		Node::output_repr_start(output, "<RawExpr", "(")?;
		Node::output_repr_list(output, &self.expr, " ")?;
		Node::output_repr_end(output, ">", ")")?;
		Ok(())
	}
}

impl WithSpan for RawExpr {
	fn get_span(&self) -> Option<Span> {
		Node::span_for_list(&self.expr)
	}
}

//====================================================================================================================//
// Block parsing
//====================================================================================================================//

pub struct BlockParser<T: TokenStream> {
	input: T,
}

impl<T: TokenStream> BlockParser<T> {
	pub fn new(input: T) -> Self {
		Self { input }
	}

	pub fn read_next(&mut self, errors: &mut Errors) -> Option<Node> {
		parse_statement(&mut self.input, errors, StopCondition::none())
	}
}

fn parse_statement<T: TokenStream>(
	input: &mut T,
	errors: &mut Errors,
	stop: StopCondition,
) -> Option<Node> {
	skip_empty(input, errors, stop);
	if let Some(next) = input.next() {
		if stop.should_stop(&next) {
			return None;
		}
	} else {
		return None;
	}

	let mut expr = Vec::new();
	while let Some(next) = input.read(errors) {
		match next.get_token() {
			Some(token) => {
				let (include, stop) = match token {
					&Token::Symbol(";") => (false, true),
					&Token::Symbol(":") => todo!(),
					Token::Break => (false, true),
					_ => (true, false),
				};
				if include {
					expr.push(next)
				}
				if stop {
					break;
				}
			}
			None => expr.push(next),
		}
	}

	let expr = RawExpr::new(expr);
	Some(Node::from(Statement { expr, block: None }))
}

fn skip_empty<T: TokenStream>(input: &mut T, errors: &mut Errors, stop: StopCondition) {
	use crate::lang::Comment;

	while let Some(next) = input.next() {
		if stop.should_stop(&next) {
			break;
		}
		if next.is_token(|token| token == &Token::Break) || next.is::<Comment>() {
			input.read(errors);
		} else {
			break;
		}
	}
}

#[derive(Copy, Clone)]
struct StopCondition {
	level: usize,
	symbol: Option<&'static str>,
}

impl StopCondition {
	pub fn none() -> Self {
		Self {
			level: 0,
			symbol: None,
		}
	}

	pub fn _level(level: usize) -> Self {
		Self {
			level,
			symbol: None,
		}
	}

	pub fn should_stop(&self, next: &Node) -> bool {
		if let Some(span) = next.span() {
			if span.start().indent() < self.level {
				return true;
			}
		}
		if let Some(symbol) = self.symbol {
			if next.get_token() == Some(&Token::Symbol(symbol)) {
				return true;
			}
		}
		false
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::lang::*;

	#[test]
	fn empty() {
		let blocks = read("");
		assert!(blocks.len() == 0);

		let blocks = read("    ");
		assert!(blocks.len() == 0);

		let blocks = read("\n\n\n");
		assert!(blocks.len() == 0);

		let blocks = read("\n# some comment\n\n# another comment\n");
		assert!(blocks.len() == 0);
	}

	#[test]
	fn lines() {
		let blocks = read("1\n2 3\n# comment\n4 5\n6\n");
		let a = RawExpr::new(vec![n(1)]);
		let b = RawExpr::new(vec![n(2), n(3)]);
		let c = RawExpr::new(vec![n(4), n(5)]);
		let d = RawExpr::new(vec![n(6)]);
		check(
			blocks,
			vec![
				Statement::new(a, None),
				Statement::new(b, None),
				Statement::new(c, None),
				Statement::new(d, None),
			],
		)
	}

	fn check(actual: Vec<Node>, expected: Vec<Node>) {
		if actual != expected {
			let mut output = std::io::stderr().lock();
			let mut output = Repr::new(&mut output, ReprMode::Debug, ReprFormat::Full);

			let _ = write!(output, "\nActual:\n");
			{
				let mut output = output.indented();
				for (i, it) in actual.iter().enumerate() {
					let _ = write!(output, "\n[{i}] ");
					let _ = it.output_repr(&mut output);
				}
				let _ = write!(output, "\n");
			}

			let _ = write!(output, "\nExpected:\n");
			{
				let mut output = output.indented();
				for (i, it) in expected.iter().enumerate() {
					let _ = write!(output, "\n[{i}] ");
					let _ = it.output_repr(&mut output);
				}
				let _ = write!(output, "\n");
			}
			let _ = write!(output, "\n");
		}
		assert_eq!(actual, expected);
	}

	fn n(value: u128) -> Node {
		Node::from(Integer(value))
	}

	fn _s(symbol: &'static str) -> Node {
		Node::from(Token::Symbol(symbol))
	}

	fn read(input: &str) -> Vec<Node> {
		let mut blocks = open(input);
		let mut output = Vec::new();
		let mut errors = Errors::new();
		while let Some(node) = blocks.read_next(&mut errors) {
			output.push(node);
		}
		assert!(errors.empty(), "parser generated errors:\n{errors}");
		output
	}

	fn open(input: &str) -> BlockParser<InputTokenStream> {
		let mut scanner = Scanner::new();
		scanner.add_matcher(IntegerMatcher);
		scanner.add_matcher(CommentMatcher);
		scanner.add_symbol("+", Token::Symbol("+"));
		scanner.add_symbol("-", Token::Symbol("-"));
		scanner.add_symbol("*", Token::Symbol("*"));
		scanner.add_symbol("/", Token::Symbol("/"));
		scanner.add_symbol(":", Token::Symbol(":"));
		scanner.add_symbol(";", Token::Symbol(";"));
		scanner.add_symbol(",", Token::Symbol(","));
		scanner.add_symbol("(", Token::Symbol("("));
		scanner.add_symbol(")", Token::Symbol(")"));
		scanner.add_symbol("[", Token::Symbol("["));
		scanner.add_symbol("]", Token::Symbol("]"));
		scanner.add_symbol("{", Token::Symbol("{"));
		scanner.add_symbol("}", Token::Symbol("}"));

		let input = Input::from(input);
		let input = InputTokenStream::new(input.start(), scanner);
		let block = BlockParser::new(input);
		block
	}
}
