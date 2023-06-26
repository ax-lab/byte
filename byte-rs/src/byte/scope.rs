use super::*;

pub struct Scope {
	data: Arc<ScopeData>,
}

#[doc(hidden)]
pub struct ScopeData {
	program: Handle<Program>,
	parent: Option<Handle<Scope>>,
	children: RwLock<Vec<Arc<ScopeData>>>,
	scanner: Option<Scanner>,
	operators: Arc<Vec<Operator>>,
}

impl ScopeData {
	pub fn new(program: Handle<Program>) -> Self {
		Self {
			program,
			parent: Default::default(),
			children: Default::default(),
			scanner: Default::default(),
			operators: Default::default(),
		}
	}
}

impl Scope {
	pub fn new(program: Handle<Program>) -> Self {
		let data = ScopeData::new(program);
		Self { data: data.into() }
	}

	pub fn parent(&self) -> Option<HandleRef<Scope>> {
		self.data.parent.as_ref().map(|parent| parent.get())
	}

	pub fn new_child(&self) -> Handle<Scope> {
		let mut data = ScopeData::new(self.data.program.clone());
		data.parent = Some(self.handle());
		let mut children = self.data.children.write().unwrap();

		let data = Arc::new(data);
		children.push(data.clone());
		Scope { data }.handle()
	}

	pub fn scanner(&self) -> Scanner {
		if let Some(ref scanner) = self.data.scanner {
			scanner.clone()
		} else if let Some(parent) = self.parent() {
			parent.scanner()
		} else {
			self.data.program.read(|x| x.default_scanner().clone())
		}
	}

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
		for it in self.data.operators.iter() {
			set.insert(it.clone());
		}
	}
}

impl CanHandle for Scope {
	type Data = ScopeData;

	fn inner_data(&self) -> &Arc<Self::Data> {
		&self.data
	}

	fn from_inner_data(data: Arc<Self::Data>) -> Self {
		Scope { data }
	}
}
