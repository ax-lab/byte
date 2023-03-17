use crate::lexer::Stream;

mod error;
pub use error::*;

mod node;
pub use node::*;

mod parser;

#[derive(Clone, Debug)]
pub enum Result {
	None,
	Fatal(Error),
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

pub fn run(mut input: Stream) -> Result {
	let mut state = State::new();
	let mut program = Vec::new();
	while input.value().is_some() && state.is_valid() {
		let expr = parser::parse_node(&mut input, &mut state);
		let node = resolve_macro(&mut state, expr);
		program.push(node);
	}

	let (program, errors) = state.finish(program);
	if errors.len() > 0 {
		eprintln!();
		for it in errors.into_iter() {
			eprintln!("error: {it}");
		}
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
	match node.val {
		NodeValue::None => Result::None,
		NodeValue::Invalid => Result::Fatal(Error::InvalidNode),
		NodeValue::Atom(value) => Result::Value(match value {
			Atom::Bool(value) => Value::Bool(value),
			Atom::Integer(value) => Value::Integer(value),
			Atom::Null => Value::Null,
			Atom::String(value) => Value::String(value),
			Atom::Id(..) => todo!(),
		}),
	}
}

pub struct State {}

impl State {
	fn new() -> Self {
		State {}
	}

	fn finish(self, program: Vec<Node>) -> (Vec<Node>, Vec<Error>) {
		(program, Vec::new())
	}

	fn is_valid(&self) -> bool {
		true
	}
}

struct Runtime {}

impl Runtime {
	fn new() -> Self {
		Runtime {}
	}
}

fn resolve_macro(_input: &mut State, expr: Node) -> Node {
	expr
}
