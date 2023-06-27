use super::*;

/// Default tab width across the compiler.
///
/// This affects reported column numbers and the computed indentation values.
pub const DEFAULT_TAB_WIDTH: usize = 4;

/// Contains the whole program being compiled. This includes all files, text
/// segments, pre-defined symbols, libraries, etc.
pub struct Program {
	data: Arc<ProgramData>,
}

#[doc(hidden)]
pub struct ProgramData {
	compiler: CompilerRef,
	tab_width: usize,
	segments: RwLock<Vec<NodeList>>,
	run_list: RwLock<Vec<NodeList>>,
	root_scope: Scope,
	sources: SourceList,
}

impl Program {
	pub fn new(compiler: &Compiler) -> Program {
		let base_path = compiler.base_path();
		let compiler = compiler.get_ref();
		Program::new_cyclic(|handle| {
			let root_scope = Scope::new(handle);
			ProgramData {
				compiler,
				root_scope,
				tab_width: DEFAULT_TAB_WIDTH,
				segments: Default::default(),
				run_list: Default::default(),
				sources: SourceList::new(base_path).unwrap(),
			}
		})
	}

	pub fn default_scanner(&self) -> Scanner {
		self.data.compiler.get().scanner().clone()
	}

	pub fn root_scope(&self) -> &Scope {
		&self.data.root_scope
	}

	pub fn tab_width(&self) -> usize {
		self.data.tab_width
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
		let span = self.data.sources.add_text(name, data.as_ref());
		self.load_span(span)
	}

	pub fn load_file<T: AsRef<Path>>(&mut self, path: T) -> Result<NodeList> {
		let span = self.data.sources.add_file(path)?;
		let list = self.load_span(span);

		let mut run_list = self.data.run_list.write().unwrap();
		run_list.push(list.clone());

		Ok(list)
	}

	fn load_span(&mut self, span: Span) -> NodeList {
		let node = Node::Module(span.clone()).at(span);
		let scope = self.root_scope().new_child();
		let list = NodeList::from_single(scope, node);
		let mut segments = self.data.segments.write().unwrap();
		segments.push(list.clone());
		list
	}

	fn run_resolved_nodes(&self, nodes: &NodeList) -> Result<Value> {
		let compiler = self.data.compiler.get();
		let mut scope = RuntimeScope::new();
		let mut value = Value::from(());
		for expr in nodes.generate_code(&compiler)? {
			value = expr.execute(&mut scope)?;
		}
		Ok(value)
	}

	pub fn resolve(&self) -> Result<()> {
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
		todo!()
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
