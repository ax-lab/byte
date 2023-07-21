use std::collections::{btree_map::Entry, BTreeMap};

use super::*;

/// Manages a root [`Scope`] and provides access to writing to scopes
/// using a [`ScopeWriter`].
pub struct ScopeList {
	id: Id,
	root: Arc<ScopeData>,
}

impl ScopeList {
	pub fn new(program: ProgramHandle) -> Self {
		let id = id();
		let root = ScopeData::new(id, program);
		Self { id, root: root.into() }
	}

	pub fn get_root_writer(&self) -> ScopeWriter {
		self.get_writer(self.get_root())
	}

	pub fn get_root(&self) -> Scope {
		Scope {
			data: self.root.clone(),
		}
	}

	pub fn get_writer(&self, scope: Scope) -> ScopeWriter {
		assert!(scope.data.list_id == self.id);
		ScopeWriter { scope }
	}
}

/// Provides read-only access to scoped data for the language.
///
/// This includes declared variables, operators, the configured [`Matcher`],
/// and others.
///
/// Scopes can be inherited. Data from a parent scope is used unless it is
/// overridden by the children.
pub struct Scope {
	data: Arc<ScopeData>,
}

/// Provides write access to the [`Scope`] data. This can only be obtained
/// through the parent [`ScopeList`] for a scope.
pub struct ScopeWriter {
	scope: Scope,
}

/// Handle with a weak-reference to a [`Scope`].
///
/// This should always be used when storing references to a scope, as it can
/// be safely stored inside data that is owned by the scope without creating
/// cycles and leaking memory.
#[derive(Clone)]
pub struct ScopeHandle {
	data: Weak<ScopeData>,
}

impl ScopeHandle {
	pub fn get(&self) -> Scope {
		Scope {
			data: self.data.upgrade().expect("using orphaned ScopeHandle"),
		}
	}
}

impl PartialEq for ScopeHandle {
	fn eq(&self, other: &Self) -> bool {
		self.data.as_ptr() == other.data.as_ptr()
	}
}

/// Internal data for a scope.
struct ScopeData {
	list_id: Id,
	program: ProgramHandle,
	parent: Option<ScopeHandle>,
	children: RwLock<Vec<Arc<ScopeData>>>,
	matcher: Arc<RwLock<Option<Matcher>>>,
	node_evaluators: Arc<RwLock<HashMap<NodeEval, NodePrecedence>>>,
	bindings: RwLock<HashMap<Symbol, BindingList>>,
}

impl ScopeData {
	pub fn new(list_id: Id, program: ProgramHandle) -> Self {
		Self {
			list_id,
			program,
			parent: Default::default(),
			children: Default::default(),
			matcher: Default::default(),
			node_evaluators: Default::default(),
			bindings: Default::default(),
		}
	}
}

//====================================================================================================================//
// Scope read interface
//====================================================================================================================//

impl Scope {
	pub fn program(&self) -> ProgramRef {
		self.data.program.get()
	}

	pub fn parent(&self) -> Option<Scope> {
		self.data.parent.as_ref().map(|parent| parent.get())
	}

	pub fn handle(&self) -> ScopeHandle {
		let data = Arc::downgrade(&self.data);
		ScopeHandle { data }
	}

	pub fn new_child(&self) -> ScopeHandle {
		let mut data = ScopeData::new(self.data.list_id, self.data.program.clone());
		data.parent = Some(self.handle());
		let mut children = self.data.children.write().unwrap();

		let data = Arc::new(data);
		children.push(data.clone());
		Scope { data }.handle()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Matcher
	//----------------------------------------------------------------------------------------------------------------//

	pub fn matcher(&self) -> Matcher {
		let matcher = self.data.matcher.read().unwrap().clone();
		if let Some(matcher) = matcher {
			matcher
		} else if let Some(parent) = self.parent() {
			parent.matcher()
		} else {
			let program = self.program();
			program.compiler().new_matcher()
		}
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Node evaluators
	//----------------------------------------------------------------------------------------------------------------//

	pub fn get_node_evaluators(&self) -> Vec<(NodeEval, NodePrecedence)> {
		let mut map = HashMap::new();
		if let Some(parent) = self.parent() {
			parent.get_node_evaluator_map(&mut map);
		}
		self.get_node_evaluator_map(&mut map);

		let mut output = map.into_iter().collect::<Vec<_>>();
		output.sort_by_key(|x| x.1);
		output
	}

	fn get_node_evaluator_map(&self, map: &mut HashMap<NodeEval, NodePrecedence>) {
		let evaluators = self.data.node_evaluators.read().unwrap();
		for (key, val) in evaluators.iter() {
			map.insert(key.clone(), val.clone());
		}
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Bindings
	//----------------------------------------------------------------------------------------------------------------//

	pub fn lookup(&self, name: &Symbol, offset: &CodeOffset) -> Option<CodeOffset> {
		let value = {
			let bindings = self.data.bindings.read().unwrap();
			if let Some(value) = bindings.get(&name) {
				value.lookup_index(offset)
			} else {
				None
			}
		};

		value.or_else(|| {
			if let Some(parent) = self.parent() {
				parent.lookup(name, offset)
			} else {
				None
			}
		})
	}

	pub fn get(&self, name: Symbol, offset: &CodeOffset) -> Option<Expr> {
		let value = {
			let bindings = self.data.bindings.read().unwrap();
			bindings.get(&name).and_then(|x| x.get(offset).cloned())
		};

		value.or_else(|| {
			if let Some(parent) = self.parent() {
				parent.get(name, offset)
			} else {
				None
			}
		})
	}
}

//====================================================================================================================//
// Scope write interface
//====================================================================================================================//

impl ScopeWriter {
	pub fn set_matcher(&mut self, new_matcher: Matcher) {
		let mut matcher = self.data().matcher.write().unwrap();
		*matcher = Some(new_matcher);
	}

	pub fn add_node_eval(&mut self, op: NodeEval, prec: NodePrecedence) {
		let mut evaluators = self.data().node_evaluators.write().unwrap();
		evaluators.insert(op, prec);
	}

	pub fn set(&mut self, name: Symbol, offset: CodeOffset, value: Expr) -> Result<()> {
		// TODO: setting the expression value here does not have much of a meaning right now
		let mut bindings = self.data().bindings.write().unwrap();
		let binding = bindings
			.entry(name.clone())
			.or_insert_with(|| BindingList::new(name.clone()));
		binding.set(offset, value)
	}

	fn data(&mut self) -> &ScopeData {
		&self.scope.data
	}
}

impl Deref for ScopeWriter {
	type Target = Scope;

	fn deref(&self) -> &Self::Target {
		&self.scope
	}
}

//====================================================================================================================//
// Internals
//====================================================================================================================//

#[derive(Default)]
struct BindingList {
	name: Symbol,
	values: BTreeMap<CodeOffset, Expr>,
}

impl BindingList {
	pub fn new(name: Symbol) -> Self {
		Self {
			name,
			values: Default::default(),
		}
	}

	pub fn get(&self, offset: &CodeOffset) -> Option<&Expr> {
		let offset = self.lookup_index(offset);
		offset.and_then(|offset| self.values.get(&offset))
	}

	pub fn set(&mut self, offset: CodeOffset, value: Expr) -> Result<()> {
		match self.values.entry(offset) {
			Entry::Vacant(entry) => {
				entry.insert(value);
				Ok(())
			}
			Entry::Occupied(..) => {
				let name = &self.name;
				let error = format!("`{name}` already bound at {offset}");
				Err(Errors::from(error, value.span().clone()))
			}
		}
	}

	pub fn lookup_index(&self, offset: &CodeOffset) -> Option<CodeOffset> {
		let static_offset = || {
			if self.values.contains_key(&CodeOffset::Static) {
				Some(CodeOffset::Static)
			} else {
				None
			}
		};
		match offset {
			CodeOffset::Static => static_offset(),
			CodeOffset::At(offset) => {
				let keys = self.values.keys().collect::<Vec<_>>();
				let index = keys.partition_point(|x| {
					if let CodeOffset::At(bind_offset) = x {
						offset < bind_offset
					} else {
						true
					}
				});
				if let Some(offset) = keys.get(index) {
					Some(**offset)
				} else {
					static_offset()
				}
			}
		}
	}
}
