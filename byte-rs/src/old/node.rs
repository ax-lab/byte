use crate::core::error::*;
use crate::core::input::*;
use crate::lang::operator::*;
use crate::lexer::*;

#[derive(Clone, Debug)]
pub enum NodeError {
	At(String, Box<NodeError>),
	Expected(&'static str, TokenAt),
	ExpectedExpression(TokenAt),
	ExpectedSymbol(&'static str, Span),
	ExpectedIndent(Span),
	InvalidToken(Span),
}

impl IsError for NodeError {
	fn output(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{self}")
	}
}

impl NodeError {
	pub fn span(&self) -> Span {
		match self {
			NodeError::At(_, err) => err.span(),
			NodeError::Expected(_, lex) => lex.span(),
			NodeError::ExpectedExpression(lex) => lex.span(),
			NodeError::ExpectedSymbol(_, span) => span.clone(),
			NodeError::ExpectedIndent(span) => span.clone(),
			NodeError::InvalidToken(span) => span.clone(),
		}
	}

	pub fn at<T: Into<String>>(self, context: T) -> NodeError {
		NodeError::At(context.into(), self.into())
	}
}

/// Represents a syntactic structure in the source code.
///
/// Nodes either map 1:1 to expressions in the source or are parsed through
/// syntax macros.
///
/// After parsing, a Node is evaluated and the result is the actual compiled
/// program output
#[derive(Clone, Debug)]
pub enum Node {
	None(Cursor),
	Invalid(NodeError),
	Some(NodeKind, Span),
}

impl Node {
	#[allow(unused)]
	pub fn span(&self) -> Span {
		match self {
			Node::None(cur) => Span {
				sta: cur.clone(),
				end: cur.clone(),
			},
			Node::Invalid(error) => error.span().clone(),
			Node::Some(_, span) => span.clone(),
		}
	}
}

#[derive(Clone, Debug)]
pub enum NodeKind {
	Atom(Atom),
	Unary(OpUnary, Box<NodeKind>),
	Binary(OpBinary, Box<NodeKind>, Box<NodeKind>),
	Ternary(OpTernary, Box<NodeKind>, Box<NodeKind>, Box<NodeKind>),
	Block(Vec<NodeKind>),
	Let(String, Option<Box<NodeKind>>),
	Print(Vec<NodeKind>),
	If {
		expr: Box<NodeKind>,
		block: Box<NodeKind>,
	},
	For {
		id: String,
		from: Box<NodeKind>,
		to: Box<NodeKind>,
		block: Box<NodeKind>,
	},
}

#[derive(Clone, Debug)]
pub enum Atom {
	Null,
	Bool(bool),
	String(String),
	Integer(u64),
	Id(String),
}

impl Atom {
	pub fn as_value(self) -> NodeKind {
		NodeKind::Atom(self)
	}
}

impl std::fmt::Display for NodeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			NodeError::At(context, error) => write!(f, "{context}: {error}"),
			NodeError::Expected(what, sym) => write!(f, "expected {what}, got `{sym}`"),
			NodeError::ExpectedExpression(sym) => write!(f, "expression expected, got `{sym}`"),
			NodeError::ExpectedSymbol(sym, ..) => write!(f, "expected `{sym}`"),
			NodeError::ExpectedIndent(..) => write!(f, "expected indented line"),
			NodeError::InvalidToken(..) => write!(f, "invalid token, parsing failed"),
		}
	}
}
