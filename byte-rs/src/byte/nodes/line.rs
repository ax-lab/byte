use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Line(pub NodeList);

has_traits!(Line: IsNode);

impl IsNode for Line {}

/*

	Node resolution
	===============

	Loop of expand and apply.


	Context
	=======

	Maintains a list of definitions, usually in the form of `symbol = operator`
	that is used to solve nodes.

	Each node is semantically bound to a context. Some nodes generate their own
	context, which can inherit from a parent context, which is applied to child
	nodes.


	List of operators
	=================

	Operators are maintained on a context and applied according to precedence.

	- Line operator:
		Splits the input NodeList in groups by line.

	- Indent operator:
		Splits a NodeList based on indent.

	- Bracket operator:
		Groups nodes in a NodeList based on brackets.

	- BinaryOp:
		Split a NodeList into LHS and RHS based on an operator symbol and
		groups them under an OpNode.

	- Const operator:
		Parses a `const` declaration declaring a new bind operator in the
		top-level context with the `const` name.

	- Let operator:
		Parses a `let` operator generating a new context containing a bind
		operator with the `let` name.

	- Bind operator:
		Searches for the given `name` and applies the bound node depending on
		its type. Precedence of this operator depends on the bound node.

	- Macros:
		Apply their NodeList to the location where the operator is invoked.


	Operator application
	====================

	Operators are applied across a NodeList and context. Nodes are responsible
	for applying operators to their children nodes.

	The result of an operator is a new NodeList and a list of context changes
	which have to processed before the next operator apply.

	Operators have a precedence which defines their order. Operators with the
	same precedence are applied in parallel and their result verified for
	conflicts, which result in an ambiguity error.

	Child node resolution
	=====================

	Nodes are responsible for applying resolution to their own child and
	bubbling up any changes to the parent context.

	When generating new nodes, the parent node can apply all operators up to
	the current precedence level, which allows the child nodes to properly
	respect the global operator precedence.

	The apply step must content with the possibility of new nodes in the tree,
	so it can apply previously applied operators to new ranges and respect
	any context changes derived from those operators.

*/
