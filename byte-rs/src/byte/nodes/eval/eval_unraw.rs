use super::*;

pub struct EvalUnraw;

impl IsNodeEval for EvalUnraw {
	fn applies(&self, node: &Node) -> bool {
		matches!(node.expr(), Expr::Raw(list) if list.len() == 1)
	}

	fn execute(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()> {
		let _ = ctx;
		match node.expr() {
			Expr::Raw(list) => {
				if list.len() == 1 {
					let new_value = list[0].expr();
					let new_span = list[0].span();
					node.set_value(new_value, new_span);
				}
			}
			_ => (),
		}
		Ok(())
	}
}
