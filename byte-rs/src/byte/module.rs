use super::*;

#[derive(Clone)]
pub struct Module {
	data: Arc<ModuleData>,
}

#[derive(Clone)]
struct ModuleData {
	compiler: CompilerRef,
	input: Input,
	state: Arc<RwLock<ModuleState>>,
}

struct ModuleState {
	nodes: NodeList,
	context: Context,
	resolved: bool,
	errors: Errors,
}

impl Module {
	pub fn new(compiler: &Compiler, input: Input) -> Self {
		let source = NodeValue::from(RawText(input.clone()));

		let state = ModuleState {
			nodes: NodeList::single(source),
			context: compiler.new_context(),
			resolved: false,
			errors: Errors::new(),
		};

		let state = Arc::new(RwLock::new(state));
		let compiler = compiler.get_ref();
		let data = ModuleData { compiler, input, state };
		Self { data: Arc::new(data) }
	}

	pub fn compiler(&self) -> Compiler {
		self.data.compiler.get()
	}

	pub fn input(&self) -> &Input {
		&self.data.input
	}

	pub fn resolve(&self) -> Result<()> {
		let mut state = self.data.state.write().unwrap();
		if !state.resolved {
			state.resolved = true;

			let mut errors = Errors::new();
			let (context, nodes) = state.context.resolve(&state.nodes, &mut errors);
			state.context = context;
			state.nodes = nodes;
			state.errors = errors;
		}

		if state.errors.len() > 0 {
			Err(state.errors.clone())
		} else {
			Ok(())
		}
	}

	pub fn eval(&self) -> Result<Value> {
		self.resolve()?;

		let compiler = self.compiler();
		let state = self.data.state.read().unwrap();

		let mut code = Vec::new();
		let mut errors = Errors::new();
		for it in state.nodes.iter() {
			if let Some(node) = it.as_compilable() {
				if let Some(expr) = node.compile(it, &compiler, &mut errors) {
					code.push(expr);
				}
			} else {
				errors.add(format!("resulting node is not compilable -- {it:?}"));
			}

			if errors.len() > MAX_ERRORS {
				break;
			}
		}

		if errors.len() > 0 {
			println!("\n---- NODE DUMP ----\n{:?}\n-------------------", state.nodes);
			return Err(errors);
		}

		let mut value = Value::from(());
		let mut scope = RuntimeScope::new();
		for it in code {
			value = it.execute(&mut scope)?;
		}

		Ok(value)
	}
}
