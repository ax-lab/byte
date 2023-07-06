use super::*;

pub mod list;

pub use list::*;

/// Enumeration of all available language nodes.
///
/// Nodes relate to the source code, representing language constructs of all
/// levels, from files, raw text, and tokens, all the way to fully fledged
/// definitions.
#[derive(Clone, Debug)]
pub enum Node {
	//----[ Tokens ]----------------------------------------------------------//
	Break(Id),
	Indent(usize, Id),
	Comment(Id),
	Word(Symbol, Id),
	Symbol(Symbol, Id),
	Literal(String, Id),
	Integer(u128, Id),
	Boolean(bool, Id),
	Null(Id),
	//----[ Structural ]------------------------------------------------------//
	Module(Span, Id),
	Line(NodeList, Id),
	Sequence(Vec<NodeList>, Id),
	RawText(Span, Id),
	Group(NodeList, Id),
	//----[ AST ]-------------------------------------------------------------//
	Let(Symbol, usize, NodeList, Id),
	UnaryOp(UnaryOp, NodeList, Id),
	BinaryOp(BinaryOp, NodeList, NodeList, Id),
	Variable(Symbol, Option<usize>, Id),
	Print(NodeList, &'static str, Id),
	Conditional(NodeList, NodeList, NodeList, Id),
}

impl Node {
	pub fn at(self, span: Span) -> Node {
		let id = self.id();
		id.set_span(span);
		self
	}

	pub fn symbol(&self) -> Option<Symbol> {
		let symbol = match self {
			Node::Word(symbol, ..) => symbol,
			Node::Symbol(symbol, ..) => symbol,
			Node::Let(symbol, ..) => symbol,
			_ => return None,
		};
		Some(symbol.clone())
	}
}

impl Node {
	pub fn id(&self) -> Id {
		let id = match self {
			Node::Break(id) => id,
			Node::Indent(.., id) => id,
			Node::Comment(id) => id,
			Node::Word(.., id) => id,
			Node::Symbol(.., id) => id,
			Node::Literal(.., id) => id,
			Node::Integer(.., id) => id,
			Node::Boolean(.., id) => id,
			Node::Null(id) => id,
			Node::Module(.., id) => id,
			Node::Line(.., id) => id,
			Node::Sequence(.., id) => id,
			Node::RawText(.., id) => id,
			Node::Group(.., id) => id,
			Node::Let(.., id) => id,
			Node::UnaryOp(.., id) => id,
			Node::BinaryOp(.., id) => id,
			Node::Variable(.., id) => id,
			Node::Print(.., id) => id,
			Node::Conditional(.., id) => id,
		};
		id.clone()
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
		match self {
			Node::Symbol(symbol, ..) => symbol == expected,
			_ => false,
		}
	}

	pub fn is_word(&self, expected: &Symbol) -> bool {
		match self {
			Node::Word(symbol, ..) => symbol == expected,
			_ => false,
		}
	}

	pub fn has_symbol(&self, symbol: &Symbol) -> bool {
		match self {
			Node::Symbol(s, ..) | Node::Word(s, ..) => s == symbol,
			_ => false,
		}
	}
}

impl Display for Node {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}
