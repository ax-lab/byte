use super::*;

pub fn parse_line_expr(context: &mut Context) -> Vec<ExprItem> {
	let mut out = Vec::new();
	let stop = context.limit(StopAt::Line);
	while context.next().is_some() {
		let next = context.read();
		out.push(ExprItem::Token(next));
	}
	context.release(stop);
	out
}
