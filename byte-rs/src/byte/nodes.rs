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
	//----[ Tokens ]----------------------------------------------------------//
	Break,
	Indent(usize),
	Comment,
	Word(Symbol),
	Symbol(Symbol),
	Literal(String),
	Integer(u128),
	Boolean(bool),
	Null,
	//----[ Structural ]------------------------------------------------------//
	Module(Span),
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
			Bit::Word(symbol) => symbol,
			Bit::Symbol(symbol) => symbol,
			Bit::Let(symbol, ..) => symbol,
			_ => return None,
		};
		Some(symbol.clone())
	}
}

impl Display for Bit {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{self:?}")
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
			Bit::Symbol(symbol) => symbol == expected,
			_ => false,
		}
	}

	pub fn is_word(&self, expected: &Symbol) -> bool {
		match self.bit() {
			Bit::Word(symbol) => symbol == expected,
			_ => false,
		}
	}

	pub fn has_symbol(&self, symbol: &Symbol) -> bool {
		match self.bit() {
			Bit::Symbol(s) | Bit::Word(s) => s == symbol,
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

		let format = Format::new(Mode::Minimal).with_separator(" @");
		Context::get().with_format(format, || write!(f, "{}", self.span()))
	}
}

impl Display for Node {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.bit())?;
		let format = Format::new(Mode::Normal).with_separator(" at ");
		Context::get().with_format(format, || write!(f, "{}", self.span()))
	}
}
