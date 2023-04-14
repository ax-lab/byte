use super::{node::*, parser::*, runtime::*, Context};
use crate::core::error::*;
use crate::lang::operator::*;
use crate::lexer::*;

use super::stream::*;

#[derive(Clone, Debug)]
#[allow(unused)]
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

pub fn run(input: Lexer, list_ast: bool) -> Result {
	let mut context = Context::new(input.clone());
	let mut program = Vec::new();
	while context.has_some() && context.is_valid() {
		let node = parse_line(&mut context);
		let node = match node {
			Node::Invalid(error) => {
				let span = error.span();
				context.add_error(Error::new(error).at(span));
				break;
			}
			Node::None(pos) => {
				panic!("unsupported expression at {pos}");
			}
			Node::Some(value, ..) => value,
		};
		program.push(node);

		if context.token() == Token::Symbol(";") {
			context.advance();
		}
		if context.token() == Token::Break {
			context.advance();
		}
	}

	if list_ast {
		let len = program.len();
		for (i, it) in program.into_iter().enumerate() {
			print!("\n[{}/{len}] = ", i + 1);
			println!("{:#?}", it);
		}
		std::process::exit(0);
	}

	let (program, errors) = context.finish(program);
	if errors.len() > 0 {
		eprintln!();
		for it in errors.into_iter() {
			let src = input.pos();
			let src = src.src();
			let name = src.name();
			let span = it.span();
			let span = if let Some(span) = span {
				format!(":{}", span)
			} else {
				format!("")
			};
			eprintln!("error: at {name}{span} -- {it}");
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

fn execute(rt: &mut Runtime, node: NodeKind) -> Result {
	Result::Value(execute_expr(rt, node))
}

fn execute_expr<'a>(rt: &mut Runtime, expr: NodeKind) -> Value {
	match expr {
		NodeKind::Let(id, expr) => {
			let value = if let Some(expr) = expr {
				execute_expr(rt, *expr)
			} else {
				Value::Null
			};
			rt.set(id.as_str(), value.clone());
			value
		}
		NodeKind::Print(list) => {
			let mut has_output = false;
			for expr in list.into_iter() {
				let res = execute_expr(rt, expr);
				if let Value::None = res {
					continue;
				}
				if has_output {
					print!(" ");
				}
				print!("{res}");
				has_output = true;
			}
			println!();
			Value::None
		}
		NodeKind::Block(list) => {
			let mut res = Value::Null;
			for expr in list.into_iter() {
				res = execute_expr(rt, expr);
			}
			res
		}
		NodeKind::If { expr, block } => {
			let value = execute_expr(rt, *expr);
			if value.to_bool() {
				execute_expr(rt, *block)
			} else {
				Value::None
			}
		}
		NodeKind::For {
			id,
			from,
			to,
			block,
		} => {
			let from = execute_expr(rt, *from).as_value();
			let to = execute_expr(rt, *to).as_value();

			let from = match from {
				Value::Integer(from) => from,
				value => panic!("for: invalid from expression {value:?}"),
			};
			let to = match to {
				Value::Integer(to) => to,
				value => panic!("for: invalid to expression {value:?}"),
			};

			let mut value = Value::None;
			for i in from..=to {
				rt.set(&id, Value::Integer(i));
				value = execute_expr(rt, (&*block).clone());
			}
			value
		}
		NodeKind::Unary(op, a) => {
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
		NodeKind::Binary(op, a, b) => {
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
		NodeKind::Ternary(op, a, b, c) => match op {
			OpTernary::Conditional => {
				let a = execute_expr(rt, *a);
				if a.to_bool() {
					execute_expr(rt, *b)
				} else {
					execute_expr(rt, *c)
				}
			}
		},
		NodeKind::Atom(atom) => match atom {
			Atom::Bool(value) => Value::Bool(value),
			Atom::Integer(value) => Value::Integer(value as i128),
			Atom::Null => Value::Null,
			Atom::String(value) => Value::String(value),
			Atom::Id(var) => rt.get(var.as_str()),
		},
	}
}
