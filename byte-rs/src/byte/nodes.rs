use super::*;

pub mod list;

pub use list::*;

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
			Bit::Let(symbol, ..) => symbol,
			_ => return None,
		};
		Some(symbol.clone())
	}

	pub fn get_dependencies<P: FnMut(&NodeList)>(&self, mut output: P) {
		// TODO: use this to detect dependencies for new nodes.
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

	//----------------------------------------------------------------------------------------------------------------//
	// Parsing helpers
	//----------------------------------------------------------------------------------------------------------------//

	pub fn is_symbol(&self, expected: &Symbol) -> bool {
		match self.bit() {
			Bit::Token(Token::Symbol(symbol)) => symbol == expected,
			_ => false,
		}
	}

	pub fn is_word(&self, expected: &Symbol) -> bool {
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
		ctx.with_format(format, || write!(f, "{}", self.span()))
	}
}

impl Display for Node {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.bit())?;

		let ctx = Context::get();
		let format = ctx.format().with_mode(Mode::Minimal).with_separator(" @");
		ctx.with_format(format, || write!(f, "{:#}", self.span()))
	}
}
