use super::*;

/// Contains the whole program being compiled. This includes all files, text
/// segments, pre-defined symbols, libraries, etc.
pub struct Program {
	data: Arc<ProgramData>,
}

#[doc(hidden)]
pub struct ProgramData {
	compiler: CompilerRef,
	segments: RwLock<Vec<NodeList>>,
	run_list: RwLock<Vec<NodeList>>,
	root_scope: Scope,
	runtime: RwLock<RuntimeScope>,
}

impl Program {
	pub fn new(compiler: &Compiler) -> Program {
		Program::new_cyclic(|handle| {
			let mut root_scope = Scope::new(handle);
			compiler.configure_root_scope(&mut root_scope);

			let compiler = compiler.get_ref();
			ProgramData {
				compiler,
				root_scope,
				segments: Default::default(),
				run_list: Default::default(),
				runtime: Default::default(),
			}
		})
	}

	pub fn configure_runtime<P: FnOnce(&mut RuntimeScope)>(&mut self, action: P) {
		let mut runtime = self.data.runtime.write().unwrap();
		(action)(&mut runtime);
	}

	pub fn default_scanner(&self) -> Scanner {
		self.data.compiler.get().scanner().clone()
	}

	pub fn root_scope(&self) -> &Scope {
		&self.data.root_scope
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
		let nodes = self.load_string(name, text);
		self.resolve()?;
		self.run_resolved_nodes(&nodes)
	}

	pub fn load_string<T1: Into<String>, T2: AsRef<str>>(&mut self, name: T1, data: T2) -> NodeList {
		let context = Context::get();
		let source = context.load_source_text(name, data.as_ref());
		self.load_span(source.span())
	}

	pub fn load_file<T: AsRef<Path>>(&mut self, path: T) -> Result<NodeList> {
		let context = Context::get();
		let source = context.load_source_file(path)?;
		let list = self.load_span(source.span());

		let mut run_list = self.data.run_list.write().unwrap();
		run_list.push(list.clone());

		Ok(list)
	}

	pub fn run_nodes(&mut self, nodes: &NodeList) -> Result<Value> {
		self.resolve()?;
		self.run_resolved_nodes(nodes)
	}

	fn load_span(&mut self, span: Span) -> NodeList {
		let node = Node::Module(span.clone(), at(span));
		let scope = self.root_scope().new_child();
		let list = NodeList::from_single(scope, node);
		let mut segments = self.data.segments.write().unwrap();
		segments.push(list.clone());
		list
	}

	fn run_resolved_nodes(&self, nodes: &NodeList) -> Result<Value> {
		let mut context = CodeContext::new(self.data.compiler.clone());
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
				match it.get_next_operator(precedence) {
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
				let mut context = OperatorContext::new(nodes);
				op.apply(&mut context, &mut errors);
				context.get_new_segments(&mut new_segments);
				if context.has_node_changes() {
					has_changes = has_changes || true;
				}

				let declares = context.get_declares();
				drop(context);

				let mut scope = nodes.scope_mut();
				for (name, offset, value) in declares {
					let result = if let Some(offset) = offset {
						scope.set_at(name, offset, value)
					} else {
						scope.set_static(name, value)
					};
					match result {
						Ok(..) => {}
						Err(errs) => errors.append(&errs),
					}
				}
			}

			if errors.len() > 0 {
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

impl CanHandle for Program {
	type Data = ProgramData;

	fn inner_data(&self) -> &Arc<Self::Data> {
		&self.data
	}

	fn from_inner_data(data: Arc<Self::Data>) -> Self {
		Self { data }
	}
}
