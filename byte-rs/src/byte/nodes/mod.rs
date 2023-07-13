use super::*;

pub mod eval;
pub mod list;
pub mod operators;

pub use eval::*;
pub use list::*;
pub use operators::*;

const SHOW_INDENT: bool = false;

/// Enumeration of all available language elements.
///
/// Nodes relate to the source code, representing language constructs of all
/// levels, from files, raw text, and tokens, all the way to fully fledged
/// definitions.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Bit {
	Token(Token),
	Null,
	Boolean(bool),
	//----[ Structural ]------------------------------------------------------//
	Line(NodeList),
	Sequence(Vec<NodeList>),
	RawText(Span),
	Group(NodeList),
	Block(NodeList, NodeList),
	//----[ Logic ]-----------------------------------------------------------//
	If {
		condition: NodeList,
		when_true: NodeList,
		when_false: Option<NodeList>,
	},
	//----[ AST ]-------------------------------------------------------------//
	Let(Symbol, usize, NodeList),
	UnaryOp(UnaryOp, NodeList),
	BinaryOp(BinaryOp, NodeList, NodeList),
	Variable(Symbol, Option<usize>),
	Print(NodeList, &'static str),
	Conditional(NodeList, NodeList, NodeList),
}

impl Bit {
	pub fn at(self, span: Span) -> Node {
		Node(self, at(span))
	}

	pub fn symbol(&self) -> Option<Symbol> {
		let symbol = match self {
			Bit::Token(Token::Word(symbol)) => symbol,
			Bit::Token(Token::Symbol(symbol)) => symbol,
			_ => return None,
		};
		Some(symbol.clone())
	}

	pub fn get_dependencies<P: FnMut(&NodeList)>(&self, mut output: P) {
		match self {
			Bit::Token(..) => (),
			Bit::Null => (),
			Bit::Boolean(..) => (),
			Bit::Line(expr) => output(expr),
			Bit::Sequence(list) => {
				for expr in list.iter() {
					output(expr)
				}
			}
			Bit::RawText(..) => (),
			Bit::Group(expr) => output(expr),
			Bit::Block(head, body) => {
				output(head);
				output(body);
			}
			Bit::Let(.., expr) => output(expr),
			Bit::UnaryOp(.., expr) => output(expr),
			Bit::BinaryOp(.., lhs, rhs) => {
				output(lhs);
				output(rhs);
			}
			Bit::Variable(..) => (),
			Bit::Print(expr, ..) => output(expr),
			Bit::Conditional(a, b, c) => {
				output(a);
				output(b);
				output(c);
			}
			Bit::If {
				condition,
				when_true,
				when_false,
			} => {
				output(condition);
				output(when_true);
				if let Some(when_false) = when_false {
					output(when_false);
				}
			}
		}
	}
}

impl Display for Bit {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			Bit::Line(nodes) => {
				write!(f, "Line::{nodes:?}")
			}
			_ => write!(f, "{self:?}"),
		}
	}
}

#[derive(Clone)]
pub struct Node(Bit, Id);

impl Node {
	pub fn id(&self) -> Id {
		self.1.clone()
	}

	pub fn bit(&self) -> &Bit {
		&self.0
	}

	pub fn span(&self) -> Span {
		self.id().span()
	}

	pub fn offset(&self) -> usize {
		self.span().offset()
	}

	pub fn indent(&self) -> usize {
		self.span().indent()
	}

	pub fn get_dependencies<P: FnMut(&NodeList)>(&self, output: P) {
		self.bit().get_dependencies(output)
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Parsing helpers
	//----------------------------------------------------------------------------------------------------------------//

	pub fn is_symbol(&self, expected: &Symbol) -> bool {
		match self.bit() {
			Bit::Token(Token::Symbol(symbol) | Token::Word(symbol)) => symbol == expected,
			_ => false,
		}
	}

	pub fn is_keyword(&self, expected: &Symbol) -> bool {
		match self.bit() {
			Bit::Token(Token::Word(symbol)) => symbol == expected,
			_ => false,
		}
	}

	pub fn has_symbol(&self, symbol: &Symbol) -> bool {
		match self.bit() {
			Bit::Token(Token::Symbol(s) | Token::Word(s)) => s == symbol,
			_ => false,
		}
	}

	pub fn symbol(&self) -> Option<Symbol> {
		self.bit().symbol()
	}
}

impl Debug for Node {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self.bit())?;

		let ctx = Context::get();
		let format = ctx.format().with_mode(Mode::Minimal).with_separator(" @");
		ctx.with_format(format, || write!(f, "{}", self.span()))?;
		if SHOW_INDENT {
			write!(f, "~{}", self.indent())?;
		}
		Ok(())
	}
}

impl Display for Node {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.bit())?;

		let ctx = Context::get();
		let format = ctx.format().with_mode(Mode::Minimal).with_separator(" @");
		ctx.with_format(format, || write!(f, "{:#}", self.span()))?;
		if SHOW_INDENT {
			write!(f, "~{}", self.indent())?;
		}
		Ok(())
	}
}
