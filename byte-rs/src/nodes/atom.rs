use std::io::Write;

use crate::core::repr::HasRepr;
use crate::lang::*;
use crate::lexer::*;
use crate::vm::operators::*;

use super::*;

#[derive(Clone)]
pub struct Atom(TokenAt);

impl From<TokenAt> for Atom {
	fn from(value: TokenAt) -> Self {
		Atom(value)
	}
}

impl Atom {
	pub fn symbol(&self) -> Option<&str> {
		let Atom(value) = self;
		value.symbol()
	}
}

has_traits!(Atom: IsNode, HasRepr, IsExprValueNode, IsOperatorNode);

impl IsNode for Atom {
	fn eval(&mut self, _scope: &mut Scope) -> NodeEval {
		NodeEval::Complete
	}

	fn span(&self) -> Option<Span> {
		Some(self.0.span())
	}
}

impl HasRepr for Atom {
	fn output_repr(&self, output: &mut repr::Repr) -> std::io::Result<()> {
		let token = &self.0;
		if output.is_debug() {
			write!(output, "Atom(")?;
			token.output_repr(&mut output.minimal())?;
			write!(output, ")")?;
		} else {
			write!(output, "{token}")?;
		}
		Ok(())
	}
}

impl IsExprValueNode for Atom {
	fn is_value(&self) -> Option<bool> {
		let Atom(value) = self;
		match value.token() {
			Token::Identifier => Some(true),
			token @ Token::Other(..) => Some(token.is::<Integer>() || token.is::<Literal>()),
			_ => Some(false),
		}
	}
}

impl IsOperatorNode for Atom {
	fn get_unary_pre(&self) -> Option<OpUnary> {
		if let Some(symbol) = self.symbol() {
			OpUnary::get_prefix(symbol)
		} else {
			None
		}
	}

	fn get_unary_pos(&self) -> Option<OpUnary> {
		if let Some(symbol) = self.symbol() {
			OpUnary::get_posfix(symbol)
		} else {
			None
		}
	}

	fn get_binary(&self) -> Option<OpBinary> {
		if let Some(symbol) = self.symbol() {
			OpBinary::get(symbol)
		} else {
			None
		}
	}

	fn get_ternary(&self) -> Option<(OpTernary, &'static str)> {
		if let Some(symbol) = self.symbol() {
			OpTernary::get(symbol)
		} else {
			None
		}
	}
}
