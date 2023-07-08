use super::*;

/// Manages a root [`Scope`] and provides access to writing to scopes
/// using a [`ScopeWriter`].
pub struct ScopeList {
	id: Id,
	root: Arc<ScopeData>,
}

impl ScopeList {
	pub fn new(program: Handle<Program>) -> Self {
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

/// Internal data for a scope.
struct ScopeData {
	list_id: Id,
	program: Handle<Program>,
	parent: Option<ScopeHandle>,
	children: RwLock<Vec<Arc<ScopeData>>>,
	matcher: Arc<RwLock<Option<Matcher>>>,
	operators: Arc<RwLock<HashSet<Operator>>>,
	bindings: RwLock<HashMap<Symbol, BindingList>>,
}

impl ScopeData {
	pub fn new(list_id: Id, program: Handle<Program>) -> Self {
		Self {
			list_id,
			program,
			parent: Default::default(),
			children: Default::default(),
			matcher: Default::default(),
			operators: Default::default(),
			bindings: Default::default(),
		}
	}
}

//====================================================================================================================//
// Scope read interface
//====================================================================================================================//

impl Scope {
	pub fn program(&self) -> HandleRef<Program> {
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
	// Operators
	//----------------------------------------------------------------------------------------------------------------//

	pub fn get_operators(&self) -> Vec<Operator> {
		let mut set = HashSet::new();
		if let Some(parent) = self.parent() {
			parent.get_operator_set(&mut set);
		}
		self.get_operator_set(&mut set);

		let mut output = set.into_iter().collect::<Vec<_>>();
		output.sort_by_key(|x| x.precedence());
		output
	}

	fn get_operator_set(&self, set: &mut HashSet<Operator>) {
		let operators = self.data.operators.read().unwrap();
		for it in operators.iter() {
			set.insert(it.clone());
		}
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Bindings
	//----------------------------------------------------------------------------------------------------------------//

	pub fn lookup(&self, name: &Symbol, offset: Option<usize>) -> Option<Option<usize>> {
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

	pub fn get_static(&self, name: Symbol) -> Option<BindingValue> {
		let value = {
			let bindings = self.data.bindings.read().unwrap();
			bindings.get(&name).and_then(|x| x.get_static().cloned())
		};

		value.or_else(|| {
			if let Some(parent) = self.parent() {
				parent.get_static(name)
			} else {
				None
			}
		})
	}

	pub fn get_at(&self, name: Symbol, offset: usize) -> Option<BindingValue> {
		let value = {
			let bindings = self.data.bindings.read().unwrap();
			bindings.get(&name).and_then(|x| x.get_at(offset).cloned())
		};

		value.or_else(|| {
			if let Some(parent) = self.parent() {
				parent.get_at(name, offset)
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

	pub fn add_operator(&mut self, op: Operator) {
		let mut operators = self.data().operators.write().unwrap();
		operators.insert(op);
	}

	pub fn set_static(&mut self, name: Symbol, value: BindingValue) -> Result<()> {
		let mut bindings = self.data().bindings.write().unwrap();
		let binding = bindings.entry(name.clone()).or_insert(Default::default());
		let span = value.span();
		if binding.set_static(value) {
			Ok(())
		} else {
			let error = format!("static `{name}` already defined");
			let error = Errors::from(error, span);
			Err(error)
		}
	}

	pub fn set_at(&mut self, name: Symbol, offset: usize, value: BindingValue) -> Result<()> {
		let mut bindings = self.data().bindings.write().unwrap();
		let binding = bindings.entry(name.clone()).or_insert(Default::default());
		let span = value.span();
		if binding.set_at(offset, value) {
			Ok(())
		} else {
			let error = format!("`{name}` already defined for the given offset");
			let error = Errors::from(error, span);
			Err(error)
		}
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
	value_static: Option<BindingValue>,
	value_from: Vec<(usize, BindingValue)>,
}

#[derive(Clone)]
pub enum BindingValue {
	NodeList(NodeList),
	Node(Node),
}

impl BindingValue {
	pub fn span(&self) -> Span {
		match self {
			BindingValue::NodeList(list) => list.span(),
			BindingValue::Node(node) => node.span().clone(),
		}
	}
}

impl BindingList {
	pub fn get_static(&self) -> Option<&BindingValue> {
		self.value_static.as_ref()
	}

	pub fn set_static(&mut self, value: BindingValue) -> bool {
		if self.value_static.is_some() {
			false
		} else {
			self.value_static = Some(value);
			true
		}
	}

	pub fn set_at(&mut self, offset: usize, value: BindingValue) -> bool {
		let index = self.value_from.binary_search_by_key(&offset, |x| x.0);
		match index {
			Ok(..) => false, // offset already exists
			Err(index) => {
				self.value_from.insert(index, (offset, value));
				true
			}
		}
	}

	pub fn lookup_index(&self, offset: Option<usize>) -> Option<Option<usize>> {
		let static_value = || if self.value_static.is_some() { Some(None) } else { None };
		if let Some(offset) = offset {
			let index = self.value_from.binary_search_by_key(&offset, |x| x.0);
			let index = match index {
				Ok(index) => index,
				Err(index) => {
					if index > 0 {
						index - 1
					} else {
						return (static_value)();
					}
				}
			};
			Some(Some(self.value_from[index].0))
		} else {
			(static_value)()
		}
	}

	pub fn get_at(&self, offset: usize) -> Option<&BindingValue> {
		let index = self.value_from.binary_search_by_key(&offset, |x| x.0);

		// return the nearest definition visible at the requested offset
		let index = match index {
			Ok(index) => index,
			Err(index) => {
				if index > 0 {
					index - 1
				} else {
					return self.get_static();
				}
			}
		};
		Some(&self.value_from[index].1)
	}
}
