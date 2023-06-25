use super::*;

pub struct Scope<'a> {
	program: &'a Program,
	data: Arc<ScopeData>,
}

#[derive(Default)]
pub(crate) struct ScopeData {
	parent: Option<Weak<ScopeData>>,
	children: RwLock<Vec<Arc<ScopeData>>>,
	scanner: Option<Scanner>,
	operators: Arc<Vec<Operator>>,
}

impl<'a> Scope<'a> {
	pub fn as_handle(&self) -> ScopeHandle<'a> {
		let data = Arc::downgrade(&self.data);
		ScopeHandle {
			program: self.program,
			data,
		}
	}

	pub fn parent(&self) -> Option<Scope<'a>> {
		let program = self.program;
		self.data.parent.as_ref().map(|parent| {
			let data = parent.upgrade().unwrap();
			Scope { program, data }
		})
	}

	pub fn new_child(&self) -> ScopeHandle<'a> {
		let program = self.program;
		let parent = Arc::downgrade(&self.data);
		let mut data = ScopeData::default();
		data.parent = Some(parent);
		let mut children = self.data.children.write().unwrap();

		let data = Arc::new(data);
		children.push(data.clone());
		Scope { program, data }.as_handle()
	}

	pub fn scanner(&self) -> Scanner {
		if let Some(ref scanner) = self.data.scanner {
			scanner.clone()
		} else if let Some(parent) = self.parent() {
			parent.scanner()
		} else {
			self.program.default_scanner()
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

impl ScopeData {
	pub fn to_scope<'a>(program: &'a Program, data: Arc<ScopeData>) -> Scope<'a> {
		Scope { program, data }
	}
}

/// Carries a [`Scope`] reference.
#[derive(Clone)]
pub struct ScopeHandle<'a> {
	program: &'a Program,
	data: Weak<ScopeData>,
}

impl<'a> ScopeHandle<'a> {
	pub fn get(&self) -> Scope {
		let data = self.data.upgrade().unwrap();
		Scope {
			program: self.program,
			data,
		}
	}
}
