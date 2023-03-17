use crate::lexer::Stream;

mod context;
pub use context::*;

mod node;
pub use node::*;

mod parser;

#[derive(Clone, Debug)]
pub enum Result {
	None,
	Fatal(String),
	Value(Value),
}

#[derive(Clone, Debug)]
pub enum Value {
	Null,
	Bool(bool),
	Integer(u64),
	String(String),
}

impl std::fmt::Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Value::Null => write!(f, "null"),
			Value::Bool(true) => write!(f, "true"),
			Value::Bool(false) => write!(f, "false"),
			Value::Integer(val) => write!(f, "{val}"),
			Value::String(val) => write!(f, "{val}"),
		}
	}
}

impl Result {
	fn is_final(&self) -> bool {
		match self {
			Result::Fatal(..) => true,
			_ => false,
		}
	}
}

impl std::fmt::Display for Result {
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
		let expr = parser::parse_node(&mut context);
		let node = resolve_macro(&mut context, expr);
		program.push(node);
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

fn execute(_rt: &mut Runtime, node: Node) -> Result {
	match node.value {
		NodeValue::None => Result::None,
		NodeValue::Invalid => Result::Fatal(format!("invalid node")),
		NodeValue::Atom(value) => Result::Value(match value {
			Atom::Bool(value) => Value::Bool(value),
			Atom::Integer(value) => Value::Integer(value),
			Atom::Null => Value::Null,
			Atom::String(value) => Value::String(value),
			Atom::Id(..) => todo!(),
		}),
	}
}

struct Runtime {}

impl Runtime {
	fn new() -> Self {
		Runtime {}
	}
}

fn resolve_macro<'a>(_input: &mut Context, expr: Node<'a>) -> Node<'a> {
	expr
}
