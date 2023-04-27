//! Defines common traits for [`Node`] values.
//!
//! Traits must be implemented by a node value type and provided dynamically
//! by implementing the [`HasTraits`] trait (e.g., using [`has_traits`]).
//!
//! At the very least, a node value must implement and provide the [`IsNode`]
//! trait. Other traits are required depending on the context a node is used.

use core::*;

use super::*;

//====================================================================================================================//
// IsNode
//====================================================================================================================//

/// Root trait implemented for any [`Node`] value.
pub trait IsNode: IsValue + HasRepr {
	fn eval(&mut self) -> NodeEval;

	fn span(&self) -> Option<Span>;
}

//====================================================================================================================//
// Macro traits
//====================================================================================================================//

/// Provides macro expansion in a [`Raw`] expression.
///
/// An [`IsMacroNode`] must be implemented by any node that can either parse as
/// a macro or resolve to a macro.
pub trait IsMacroNode {
	fn try_parse(&self, nodes: &[Node]) -> Option<(Vec<Node>, usize)>;
}

//====================================================================================================================//
// Expression traits
//====================================================================================================================//

/// Implemented by nodes which can possibly be used in an expression value
/// context.
pub trait IsExprValueNode {
	/// Returns if this node can be used as a value in an expression.
	fn is_value(&self) -> bool;
}

use crate::vm::operators::*;

/// Implemented by nodes which can resolve to an operand in an expression
/// context.
pub trait IsOperatorNode {
	/// Return the corresponding unary operator if this is a valid
	/// prefix unary operator symbol.
	fn get_unary_pre(&self) -> Option<OpUnary>;

	/// Return the corresponding unary operator if this is a valid
	/// posfix unary operator symbol.
	fn get_unary_pos(&self) -> Option<OpUnary>;

	/// Return the corresponding binary operator if this is a valid
	/// binary operator symbol.
	fn get_binary(&self) -> Option<OpBinary>;

	/// Return the corresponding ternary operator and delimiter symbol
	/// if this is a valid ternary operator symbol.
	fn get_ternary(&self) -> Option<(OpTernary, &'static str)>;
}
