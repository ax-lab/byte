use super::*;

pub struct LetOperator;

impl IsOperator for LetOperator {
	fn precedence(&self) -> Precedence {
		Precedence::Let
	}

	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.is_keyword(0, "let") && nodes.is_identifier(1) && nodes.is_symbol(2, "=")
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let _ = errors;
		let mut declares = Vec::new();
		context.nodes().fold_first(
			|node| node.is_symbol("="),
			|lhs, _, rhs| {
				let name = lhs.get_name(lhs.len() - 1).unwrap();
				let offset = rhs.offset();
				let value = BindingValue::NodeList(rhs.clone());
				declares.push((name.clone(), offset, value));
				let node = Node::Let(name, rhs);
				node.at(lhs.span())
			},
		);

		for (name, offset, value) in declares {
			context.declare_at(name, offset, value);
		}
	}
}
