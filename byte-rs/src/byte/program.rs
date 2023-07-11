use super::*;

/// Contains the whole program being compiled. This includes all files, text
/// segments, pre-defined symbols, libraries, etc.
pub struct Program {
	data: Arc<ProgramData>,
}

struct ProgramData {
	compiler: Compiler,
	scopes: ScopeList,
	segments: RwLock<Vec<NodeList>>,
	run_list: RwLock<Vec<NodeList>>,
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
		let data = Arc::new_cyclic(|data| {
			let handle = ProgramHandle { data: data.clone() };
			let compiler = compiler.clone();
			let scopes = ScopeList::new(handle);
			let mut root_scope = scopes.get_root_writer();
			compiler.configure_root_scope(&mut root_scope);
			ProgramData {
				compiler,
				scopes,
				segments: Default::default(),
				run_list: Default::default(),
				runtime: Default::default(),
				dump_code: Default::default(),
			}
		});
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
		let run_list = { self.data.run_list.read().unwrap().clone() };
		for it in run_list.iter() {
			value = self.run_resolved_nodes(it)?;
		}
		Ok(value)
	}

	pub fn eval<T1: Into<String>, T2: AsRef<str>>(&mut self, name: T1, text: T2) -> Result<Value> {
		let nodes = self.load_string(name, text)?;
		self.resolve()?;
		self.run_resolved_nodes(&nodes)
	}

	pub fn load_string<T1: Into<String>, T2: AsRef<str>>(&mut self, name: T1, data: T2) -> Result<NodeList> {
		let context = Context::get();
		let source = context.load_source_text(name, data.as_ref());
		self.load_span(source.span())
	}

	pub fn load_file<T: AsRef<Path>>(&mut self, path: T) -> Result<NodeList> {
		let context = Context::get();
		let source = context.load_source_file(path)?;
		let list = self.load_span(source.span())?;

		let mut run_list = self.data.run_list.write().unwrap();
		run_list.push(list.clone());

		Ok(list)
	}

	pub fn run_nodes(&mut self, nodes: &NodeList) -> Result<Value> {
		self.resolve()?;
		self.run_resolved_nodes(nodes)
	}

	fn load_span(&mut self, span: Span) -> Result<NodeList> {
		let scope = self.root_scope().new_child();
		let mut scope = self.data.scopes.get_writer(scope.get());
		let nodes = scan(&mut scope, &span)?;
		let mut segments = self.data.segments.write().unwrap();
		segments.push(nodes.clone());
		Ok(nodes)
	}

	fn run_resolved_nodes(&self, nodes: &NodeList) -> Result<Value> {
		let mut context = CodeContext::new();
		if self.dump_enabled() {
			context.dump_code();
		}

		let scope = self.data.runtime.write();
		let mut scope = match scope {
			Ok(scope) => scope,
			Err(poisoned) => poisoned.into_inner(),
		};
		let mut value = Value::from(());
		for expr in nodes.generate_code(&mut context)? {
			value = expr.execute(&mut scope)?.value();
		}
		Ok(value)
	}

	pub fn resolve(&self) -> Result<()> {
		let mut segments = self.data.segments.write().unwrap();

		let mut errors = Errors::new();
		loop {
			let mut to_process = Vec::new();
			let mut precedence = None;

			// collect the applicable operator for all segments
			for it in segments.iter_mut() {
				match it.get_next_node_operator(precedence) {
					Ok(Some(op)) => {
						let op_precedence = op.precedence();
						assert!(precedence.is_none() || Some(op_precedence) <= precedence);
						precedence = Some(op_precedence);
						to_process.push((op_precedence, op, it));
					}
					Ok(None) => {
						// do nothing
					}
					Err(segment_errors) => {
						errors.append(&segment_errors);
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
			let to_process = to_process.into_iter().filter(|(prec, ..)| *prec == precedence);

			let mut has_changes = false;

			let mut new_segments = Vec::new();
			for (_, op, nodes) in to_process {
				let mut context = EvalContext::new(nodes);
				let version = nodes.version();
				match op.apply(&mut context, nodes) {
					Ok(()) => (),
					Err(errs) => {
						errors.append(&errs);
					}
				};
				let changed = nodes.version() > version;

				context.get_new_segments(&mut new_segments);
				has_changes = has_changes || changed;

				let declares = context.get_declares();
				drop(context);

				let scope = nodes.scope_handle().get();
				let mut writer = self.data.scopes.get_writer(scope);
				for (name, offset, value) in declares {
					let result = if let Some(offset) = offset {
						writer.set_at(name, offset, value)
					} else {
						writer.set_static(name, value)
					};
					match result {
						Ok(..) => {}
						Err(errs) => errors.append(&errs),
					}
				}
			}

			if errors.len() > 0 {
				if self.dump_enabled() {
					println!("\n===== NODE DUMP =====");
					for (n, it) in segments.iter().enumerate() {
						println!("\n>>> SEGMENT {n} <<<\n");
						println!("{it}");
					}
					println!("\n=====================");
				}
				return Err(errors);
			}

			for it in new_segments {
				if !segments.contains(&it) {
					has_changes = true;
					segments.push(it);
				}
			}

			if !has_changes {
				break;
			}
		}

		if errors.len() > 0 {
			return Err(errors);
		}

		/*
			for all NodeList segments, determine the next set of operators to
			apply

				in each set, evaluate operators in groups of precedence and
				check that they can be applied

				take only the highest precedence group across all segments to
				apply

			apply the next set of operators across all segments

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

			just use bindings declared between the modules and let the operator
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
