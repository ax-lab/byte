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

struct ProgramData {
	compiler: CompilerRef,
	tab_width: usize,
	segments: Vec<NodeList>,
	root_scope: Arc<ScopeData>,
	sources: SourceList,
}

impl Program {
	pub fn new(compiler: &Compiler) -> Program {
		let base_path = compiler.base_path();
		let compiler = compiler.get_ref();
		let data = ProgramData {
			compiler,
			tab_width: DEFAULT_TAB_WIDTH,
			segments: Default::default(),
			root_scope: Default::default(),
			sources: SourceList::new(base_path).unwrap(),
		};
		Self { data: data.into() }
	}

	pub fn default_scanner(&self) -> Scanner {
		self.data.compiler.get().scanner().clone()
	}

	pub fn root_scope(&self) -> Scope {
		ScopeData::to_scope(self, self.data.root_scope.clone())
	}

	pub fn run(&self) -> Result<Value> {
		todo!()
	}

	pub fn eval<T1: AsRef<str>, T2: AsRef<str>>(&self, _name: T1, _input: T2) -> Result<Value> {
		todo!()
	}

	pub fn load_string<T: AsRef<str>>(&self, _input: T) {
		todo!()
	}

	pub fn load_file<T: AsRef<Path>>(&self, _path: T) {
		todo!()
	}

	pub fn resolve(&self) -> Result<()> {
		todo!()
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
	}
}

impl PartialEq for Program {
	fn eq(&self, other: &Self) -> bool {
		self as *const Program == other as *const Program
	}
}

impl Eq for Program {}
