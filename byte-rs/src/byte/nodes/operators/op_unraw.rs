use super::*;

pub struct OpUnraw;

impl IsNodeOperator for OpUnraw {
	fn applies(&self, node: &Node) -> bool {
		matches!(node.val(), NodeValue::Raw(list) if list.len() == 1)
	}

	fn execute(&self, ctx: &mut OperatorContext, node: &mut Node) -> Result<()> {
		let _ = ctx;
		match node.val() {
			NodeValue::Raw(list) => {
				if list.len() == 1 {
					let new_value = list[0].val();
					let new_span = list[0].span();
					node.set_value(new_value, new_span);
				}
			}
			_ => (),
		}
		Ok(())
	}
}
