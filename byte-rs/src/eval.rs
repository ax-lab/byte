use crate::lexer::{Stream, Token};

mod context;
pub use context::*;

mod node;
pub use node::*;

mod operator;
pub use operator::*;

mod parser;

pub use super::runtime::*;

mod macros;

#[derive(Clone, Debug)]
pub enum Result {
	None,
	Fatal(String),
	Value(Value),
}

impl<'a> Result {
	fn is_final(&self) -> bool {
		match self {
			Result::Fatal(..) => true,
			_ => false,
		}
	}
}

impl<'a> std::fmt::Display for Result {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Result::None => Ok(()),
			Result::Fatal(err) => write!(f, "[error: {err}]"),
			Result::Value(val) => write!(f, "{val}"),
		}
	}
}

pub fn run(input: Stream) -> Result {
	let mut context = Context::new(input.clone());
	let mut program = Vec::new();
	while context.has_some() && context.is_valid() {
		let mut line = context.scope_line(";");
		let expr = parser::parse_node(&mut line);
		let node = resolve_macro(&mut line, expr);
		program.push(node);

		context = line.pop_scope();
		if context.token() == Token::Symbol(";") {
			context.next();
		}
		if context.token() == Token::Break {
			context.next();
		}
	}

	let (program, errors) = context.finish(program);
	if errors.len() > 0 {
		eprintln!();
		for it in errors.into_iter() {
			let name = input.source().name();
			let span = it.span();
			eprintln!("error: at {name}:{span} -- {it}");
		}
		eprintln!();
		std::process::exit(1);
	}

	let mut runtime = Runtime::new();
	let mut result = Result::None;
	for it in program.into_iter() {
		result = execute(&mut runtime, it);
		if result.is_final() {
			break;
		}
	}

	result
}

fn execute(rt: &mut Runtime, node: Node) -> Result {
	match node.value {
		NodeValue::None => Result::None,
		NodeValue::Invalid => Result::Fatal(format!("invalid node")),
		NodeValue::Expr(expr) => Result::Value(execute_expr(rt, expr)),
	}
}

fn execute_expr<'a>(rt: &mut Runtime, expr: Expr) -> Value {
	match expr {
		Expr::Unary(op, a) => {
			let a = execute_expr(rt, *a);
			match op {
				OpUnary::Minus => a.op_minus(),
				OpUnary::Plus => a.op_plus(),
				OpUnary::Not => a.op_not(),
				OpUnary::Negate => a.op_negate(),
				OpUnary::PreIncrement => a.op_pre_increment(),
				OpUnary::PreDecrement => a.op_pre_decrement(),
				OpUnary::PosIncrement => a.op_pos_increment(),
				OpUnary::PosDecrement => a.op_pos_decrement(),
			}
		}
		Expr::Binary(op, a, b) => {
			let a = execute_expr(rt, *a);
			let b = move || execute_expr(rt, *b);
			match op {
				OpBinary::Add => a.op_add(b()),
				OpBinary::Sub => a.op_sub(b()),
				OpBinary::Mul => a.op_mul(b()),
				OpBinary::Div => a.op_div(b()),
				OpBinary::Mod => a.op_mod(b()),
				OpBinary::Equal => a.op_equal(b()),
				OpBinary::Assign => a.op_assign(b()),
				OpBinary::And => {
					if a.to_bool() {
						b().clone()
					} else {
						a.clone()
					}
				}
				OpBinary::Or => {
					if a.to_bool() {
						a.clone()
					} else {
						b().clone()
					}
				}
			}
		}
		Expr::Ternary(op, a, b, c) => match op {
			OpTernary::Conditional => {
				let a = execute_expr(rt, *a);
				if a.to_bool() {
					execute_expr(rt, *b)
				} else {
					execute_expr(rt, *c)
				}
			}
		},
		Expr::Value(atom) => match atom {
			Atom::Bool(value) => Value::Bool(value),
			Atom::Integer(value) => Value::Integer(value as i128),
			Atom::Null => Value::Null,
			Atom::String(value) => Value::String(value),
			Atom::Id(var) => rt.get(var.as_str()),
		},
	}
}

fn resolve_macro<'a>(_input: &mut Context, expr: Node<'a>) -> Node<'a> {
	expr
}
