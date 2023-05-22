use std::sync::{Arc, RwLock};

use crate::lexer::*;

use super::*;

/// Represents an isolated module of code.
#[derive(Clone)]
pub struct Module {
	data: Arc<ModuleData>,
}

struct ModuleData {
	context: Context,
	input: Input,
	nodes: Vec<Node>,
	code: RwLock<Option<Code>>,
}

impl Module {
	/// Create a new module using the given context and parsing code from the
	/// given input.
	pub fn new(mut context: Context, input: Input) -> Self {
		// Tokenize the file and parse the raw  code segments to be resolved.
		let mut scanner = context.scanner();

		let mut errors = Errors::new();
		let tokens = NodeList::tokenize(input.clone(), &mut scanner, &mut errors);

		let mut stream = tokens.into_iter();
		let nodes = if errors.empty() {
			parse_segments(&scanner, &mut stream, &mut errors)
		} else {
			Vec::new()
		};

		// Raise any lexical or parsing errors in the context.
		context.raise_errors(&errors);

		// Update the scanner in the context with any additional rules and
		// definitions from the lexical analysis.
		context.update_scanner(scanner);

		// Add the parsed nodes to be resolved by the context.
		context.queue_nodes(nodes.iter().cloned());

		Self {
			data: Arc::new(ModuleData {
				context,
				input,
				nodes,
				code: RwLock::new(None),
			}),
		}
	}

	pub fn input(&self) -> &Input {
		&self.data.input
	}

	pub fn context(&self) -> &Context {
		&self.data.context
	}

	pub fn nodes(&self) -> impl Iterator<Item = &Node> + '_ {
		self.data.nodes.iter()
	}

	pub fn errors(&self) -> Errors {
		self.data.context.errors()
	}

	pub fn has_errors(&self) -> bool {
		self.data.context.has_errors()
	}

	pub fn code(&self) -> Code {
		{
			let code = match self.data.code.read() {
				Ok(code) => code,
				Err(err) => err.into_inner(),
			};

			if let Some(code) = code.as_ref() {
				return code.clone();
			}
		}

		self.compile_code();
		self.code()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Module compilation & resolution
	//----------------------------------------------------------------------------------------------------------------//

	/// Compile the module code. This should only be called after all nodes
	/// have been resolved by the context.
	pub fn compile_code(&self) {
		let mut code = self.data.code.write().unwrap();
		if code.is_none() {
			*code = Some(self.recompile());
		}
	}

	fn recompile(&self) -> Code {
		let context = self.context();

		let mut code = Code::default();
		for it in self.nodes() {
			let expr = get_trait!(it, IsCompilable);
			let expr = expr.and_then(|x| x.compile(context));
			if let Some(expr) = expr {
				code.append(expr);
			} else {
				context.raise_error(format!("node cannot be compiled: {it:?}").at_node(it))
			}
		}

		code
	}
}
