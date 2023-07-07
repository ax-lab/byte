use super::*;

pub struct LetOperator;

impl IsOperator for LetOperator {
	fn precedence(&self) -> Precedence {
		Precedence::Let
	}

	fn can_apply(&self, nodes: &NodeList) -> bool {
		// TODO: make a static list of symbols
		nodes.is_keyword(0, &"let".into()) && nodes.is_identifier(1) && nodes.is_symbol(2, &"=".into())
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let _ = errors;
		let mut declares = Vec::new();
		let mut new_lists = Vec::new();
		context.nodes().fold_first(
			|node| node.is_symbol(&"=".into()),
			|lhs, _, rhs| {
				let name = lhs.get_symbol(lhs.len() - 1).unwrap();
				let offset = rhs.offset();
				let value = BindingValue::NodeList(rhs.clone());
				new_lists.push(rhs.clone());
				declares.push((name.clone(), offset, value));
				let rhs_span = rhs.span();
				Bit::Let(name, offset, rhs).at(lhs.span().to(rhs_span))
			},
		);

		for (name, offset, value) in declares {
			context.declare_at(name, offset, value);
		}
		for it in new_lists {
			context.resolve_nodes(&it);
		}
	}
}
