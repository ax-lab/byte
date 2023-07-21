use super::*;

/// Contains the whole program being compiled. This includes all files, text
/// segments, pre-defined symbols, libraries, etc.
pub struct Program {
	data: Arc<ProgramData>,
}

struct ProgramData {
	compiler: Compiler,
	scopes: ScopeList,
	nodes: RwLock<Vec<Node>>,
	run_list: RwLock<Vec<Node>>,
	runtime: RwLock<RuntimeScope>,
	dump_code: RwLock<bool>,
}

//====================================================================================================================//
// Handle and reference
//====================================================================================================================//

/// Handle with a weak reference to a [`Program`].
///
/// This allows storing references to a program from data owned by the program
/// without creating dependency cycles.
#[derive(Clone)]
pub struct ProgramHandle {
	data: Weak<ProgramData>,
}

impl ProgramHandle {
	pub fn get(&self) -> ProgramRef {
		let data = self.data.upgrade().expect("using orphaned program handle");
		let program = Program { data };
		ProgramRef { program }
	}
}

impl PartialEq for ProgramHandle {
	fn eq(&self, other: &Self) -> bool {
		self.data.as_ptr() == other.data.as_ptr()
	}
}

impl Eq for ProgramHandle {}

/// Reference to a [`Program`] obtained through a [`ProgramHandle`].
pub struct ProgramRef {
	program: Program,
}

impl Deref for ProgramRef {
	type Target = Program;

	fn deref(&self) -> &Self::Target {
		&self.program
	}
}

//====================================================================================================================//
// Program methods
//====================================================================================================================//

impl Program {
	pub fn new(compiler: &Compiler) -> Program {
		let runtime = RuntimeScope::new();
		let data = Arc::new_cyclic(|data| {
			let handle = ProgramHandle { data: data.clone() };
			let compiler = compiler.clone();
			let scopes = ScopeList::new(handle);
			ProgramData {
				compiler,
				scopes,
				nodes: Default::default(),
				run_list: Default::default(),
				runtime: runtime.into(),
				dump_code: Default::default(),
			}
		});

		let mut root_scope = data.scopes.get_root_writer();
		compiler.configure_root_scope(&mut root_scope);

		Program { data }
	}

	pub fn compiler(&self) -> &Compiler {
		&self.data.compiler
	}

	pub fn dump_code(&mut self) {
		*self.data.dump_code.write().unwrap() = true;
	}

	pub fn dump_enabled(&self) -> bool {
		*self.data.dump_code.read().unwrap()
	}

	pub fn configure_runtime<P: FnOnce(&mut RuntimeScope)>(&mut self, action: P) {
		let mut runtime = self.data.runtime.write().unwrap();
		(action)(&mut runtime);
	}

	pub fn root_scope(&self) -> Scope {
		self.data.scopes.get_root()
	}

	pub fn run(&self) -> Result<Value> {
		self.resolve()?;
		let mut value = Value::from(());
		let mut run_list = { self.data.run_list.read().unwrap().clone() };
		for it in run_list.iter_mut() {
			value = self.run_resolved(it)?;
		}
		Ok(value)
	}

	pub fn eval<T1: Into<String>, T2: AsRef<str>>(&mut self, name: T1, text: T2) -> Result<Value> {
		let mut node = self.load_string(name, text)?;
		self.resolve()?;
		self.run_resolved(&mut node)
	}

	pub fn load_string<T1: Into<String>, T2: AsRef<str>>(&mut self, name: T1, data: T2) -> Result<Node> {
		let context = Context::get();
		let source = context.load_source_text(name, data.as_ref());
		self.load_span(source.span())
	}

	pub fn load_file<T: AsRef<Path>>(&mut self, path: T) -> Result<Node> {
		let context = Context::get();
		let source = context.load_source_file(path)?;
		let list = self.load_span(source.span())?;

		let mut run_list = self.data.run_list.write().unwrap();
		run_list.push(list.clone());

		Ok(list)
	}

	pub fn run_node(&mut self, node: &mut Node) -> Result<Value> {
		self.resolve()?;
		self.run_resolved(node)
	}

	fn load_span(&mut self, span: Span) -> Result<Node> {
		let scope = self.root_scope().new_child();
		let mut scope = self.data.scopes.get_writer(scope.get());
		let node = scan(&mut scope, &span)?;
		let mut segments = self.data.nodes.write().unwrap();
		segments.push(node.clone());
		Ok(node)
	}

	fn run_resolved(&self, node: &mut Node) -> Result<Value> {
		// TODO: unify scope and runtime
		let runtime_scope = self.data.runtime.write();
		let mut runtime_scope = match runtime_scope {
			Ok(scope) => scope,
			Err(poisoned) => poisoned.into_inner(),
		};
		let expr = node.generate_code()?;
		let value = expr.execute(&mut runtime_scope)?.into_value();
		Ok(value)
	}

	pub fn resolve(&self) -> Result<()> {
		let mut nodes_to_process = self.data.nodes.write().unwrap();

		let mut errors = Errors::new();
		let mut pc = 0;
		loop {
			let mut to_process = Vec::new();
			let mut precedence = None;

			if DEBUG_PROCESSING {
				println!("\n=> processing {} node(s)", nodes_to_process.len());
			}

			// collect the applicable operations for all segments
			for it in nodes_to_process.iter_mut() {
				match it.get_node_operations(precedence) {
					Ok(Some((changes, prec))) => {
						assert!(precedence.is_none() || Some(prec) <= precedence);
						precedence = Some(prec);
						to_process.push((changes, prec));
					}
					Ok(None) => {
						// do nothing
					}
					Err(node_errors) => {
						errors.append(&node_errors);
					}
				}
			}

			if errors.len() > 0 || to_process.len() == 0 {
				break;
			}

			// precedence will contain the highest precedence level from all
			// segments
			let precedence = precedence.unwrap();

			// only process segments that are at the highest precedence level
			let to_process = to_process.into_iter().filter(|(_, prec)| *prec == precedence);

			let mut has_changes = false;

			let to_process = to_process.flat_map(|(ops, _)| ops.into_iter());

			for change in to_process {
				let node = change.node();
				let op = change.evaluator();
				if DEBUG_PROCESSING {
					println!("\n-> #{pc}: apply {op:?} to {}", node.short_repr());
					pc += 1;
				}

				let mut context = EvalContext::new(node);
				let version = node.version();
				let mut node = node.clone(); // TODO: maybe have a node writer?
				match change.evaluator_impl().execute(&mut context, &mut node) {
					Ok(()) => (),
					Err(errs) => {
						errors.append(&errs);
					}
				};
				let changed = node.version() > version;
				has_changes = has_changes || changed;

				let declares = context.get_declares();
				drop(context);

				if DEBUG_PROCESSING_DETAIL {
					let pc = pc - 1;
					println!("\n-> RESULT #{pc} = {node}");
				}

				let scope = node.scope_handle().get();
				let mut writer = self.data.scopes.get_writer(scope);
				for (name, offset, value) in declares {
					match writer.set(name, offset, value) {
						Ok(..) => {}
						Err(errs) => errors.append(&errs),
					}
				}
			}

			if errors.len() > 0 {
				if self.dump_enabled() {
					println!("\n===== NODE DUMP =====");
					for (n, it) in nodes_to_process.iter().enumerate() {
						println!("\n>>> NODE {n} <<<\n");
						println!("{it}");
					}
					println!("\n=====================");
				}
				return Err(errors);
			}

			if !has_changes {
				break;
			}
		}

		if DEBUG_PROCESSING {
			println!("\n>>> Node resolution complete! <<<\n");
		}

		if errors.len() > 0 {
			return Err(errors);
		}

		/*
			for all Node segments, determine the next set of evaluators to
			apply

				in each set, evaluate evaluators in groups of precedence and
				check that they can be applied

				take only the highest precedence group across all segments to
				apply

			apply the next set of evaluators across all segments

			merge and apply changes; repeat until there are no changes to
			apply


			recursive macro application?
			===========================

			keep a stack depth number for each node, and generate an error
			if it gets too high; track maximum number of nodes and limit;
			limit number of generations; track explosive growth


			tracing spans
			=============

			keep track of input spans used for each node; a node has a source
			span, which traces back directly to the origin of that node; and
			a dependency graph span, which relates all the nodes that were used
			to generate the given node


			inter-module dependencies
			=========================

			just use bindings declared between the modules and let the evaluator
			precedence take care of resolution
		*/
		Ok(())
	}
}

impl PartialEq for Program {
	fn eq(&self, other: &Self) -> bool {
		self as *const Program == other as *const Program
	}
}

impl Eq for Program {}
