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

#[derive(Default)]
struct ModuleState {
	nodes: NodeList,
	resolved: bool,
	errors: Errors,
}

impl Module {
	pub fn new(compiler: &Compiler, input: Input) -> Self {
		let compiler = compiler.get_ref();

		let mut state = ModuleState::default();

		let span = input.start().span();
		let source = Node::from(RawText(input.clone()), Some(span));
		state.nodes = NodeList::single(source);

		let state = Arc::new(RwLock::new(state));
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
			let compiler = self.compiler();
			state.resolved = true;

			let mut errors = Errors::new();
			while let Some(nodes) = compiler.resolve_next(&state.nodes, &mut errors) {
				state.nodes = nodes;
				if errors.len() > 0 {
					break;
				}
			}

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
				errors.add_at(
					format!("resulting node is not compilable -- {it:?}"),
					it.span().cloned(),
				);
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
		let mut scope = Scope::new();
		for it in code {
			value = it.execute(&mut scope)?;
		}

		Ok(value)
	}
}
