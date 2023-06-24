use super::*;

pub trait NodeOperator: Cell {
	fn evaluate(&self, context: &mut ResolveContext);
}

/// Encapsulates the compilation context for [`Node`] resolution.
///
/// The context holds everything that is available at a given location in the
/// source code and a given "point in time" in the compilation.
#[derive(Clone)]
pub struct Context {
	compiler: CompilerRef,
	parent: Option<ContextHandle>,
	data: Arc<RwLock<ContextData>>,
}

#[derive(Default)]
struct ContextData {
	scanner: Option<Scanner>,
	operators: Arc<Vec<(Precedence, Arc<dyn NodeOperator>)>>,
	_bindings: Arc<HashMap<Name, ContextBinding>>,
}

impl Context {
	pub fn new(compiler: &Compiler) -> Self {
		let compiler = compiler.get_ref();
		Self {
			compiler,
			parent: None,
			data: Default::default(),
		}
	}

	pub fn as_handle(&self) -> ContextHandle {
		ContextHandle {
			compiler: self.compiler.clone(),
			parent: self.parent.as_ref().map(|x| Box::new(x.clone())),
			data: Arc::downgrade(&self.data),
		}
	}

	pub fn compiler(&self) -> Compiler {
		self.compiler.get()
	}

	pub fn scanner(&self) -> Scanner {
		self.get(|x| x.scanner.clone(), |c| c.scanner().clone())
	}

	pub fn set_scanner(&mut self, scanner: Scanner) {
		self.write(|x| x.scanner = Some(scanner))
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Operators
	//----------------------------------------------------------------------------------------------------------------//

	pub fn declare_operator<T: NodeOperator>(&mut self, precedence: Precedence, operator: T) {
		let operator = Arc::new(operator);
		self.write(|data| {
			let operators = Arc::make_mut(&mut data.operators);
			operators.push((precedence, operator));
			operators.sort_by_key(|it| it.0);
		});
	}

	pub fn get_operators(&self) -> Vec<(Precedence, Arc<dyn NodeOperator>)> {
		self.read(|data| data.operators.to_vec())
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Internals
	//----------------------------------------------------------------------------------------------------------------//

	fn get<T, P: Fn(&ContextData) -> Option<T>, F: FnOnce(&Compiler) -> T>(&self, predicate: P, fallback: F) -> T {
		{
			let data = self.data.read().unwrap();
			predicate(&data)
		}
		.unwrap_or_else(|| {
			let result = self.parent.as_ref().and_then(|x| x.read_data(predicate));
			if let Some(result) = result {
				result
			} else {
				let compiler = self.compiler.get();
				fallback(&compiler)
			}
		})
	}

	fn read<T, P: FnOnce(&ContextData) -> T>(&self, predicate: P) -> T {
		let data = self.data.read().unwrap_or_else(|err| err.into_inner());
		predicate(&data)
	}

	fn write<T, P: FnOnce(&mut ContextData) -> T>(&mut self, predicate: P) -> T {
		let mut data = self.data.write().unwrap_or_else(|err| err.into_inner());
		predicate(&mut data)
	}
}

impl Debug for Context {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let ptr = Arc::as_ptr(&self.data);
		write!(f, "<Context {ptr:?}>")
	}
}

impl PartialEq for Context {
	fn eq(&self, other: &Self) -> bool {
		Arc::as_ptr(&self.data) == Arc::as_ptr(&other.data)
	}
}

impl Eq for Context {}

//====================================================================================================================//
// ContextHandle
//====================================================================================================================//

/// Provides a weak reference to a context that can be stored by its children.
#[derive(Clone)]
pub struct ContextHandle {
	compiler: CompilerRef,
	parent: Option<Box<ContextHandle>>,
	data: Weak<RwLock<ContextData>>,
}

impl ContextHandle {
	pub fn read<T, P: FnOnce(&Context) -> T>(&self, predicate: P) -> T {
		let context = self.upgrade();
		predicate(&context)
	}

	fn read_data<T, P: FnOnce(&ContextData) -> T>(&self, predicate: P) -> T {
		self.read(|ctx| {
			let data = ctx.data.read().unwrap();
			predicate(&data)
		})
	}

	fn upgrade(&self) -> Context {
		Context {
			compiler: self.compiler.clone(),
			parent: self.parent.as_ref().map(|x| *x.clone()),
			data: self.data.upgrade().expect("using orphaned ContextHandle"),
		}
	}
}

//====================================================================================================================//
// Helpers
//====================================================================================================================//

struct ContextBinding {
	_bindings: Vec<(Visibility, Node)>,
}

#[derive(Eq, PartialEq)]
pub enum Visibility {
	Static,
	From(usize),
	Range { start: usize, len: usize },
}

impl Visibility {
	pub fn cmp_with_pos(&self, pos: usize) -> std::cmp::Ordering {
		use std::cmp::Ordering;
		match self {
			Visibility::Static => Ordering::Greater,
			Visibility::From(start) => pos.cmp(start),
			Visibility::Range { start, len } => {
				if pos < *start {
					Ordering::Less
				} else if pos - start <= *len {
					Ordering::Equal
				} else {
					Ordering::Greater
				}
			}
		}
	}
}

impl Ord for Visibility {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		let priority = |value: &Visibility| match value {
			Visibility::Static => unreachable!(),
			Visibility::From(start) => (*start, usize::MAX),
			Visibility::Range { start, len } => (*start, *len),
		};

		if let Visibility::Static = self {
			return if let Visibility::Static = other {
				std::cmp::Ordering::Equal
			} else {
				std::cmp::Ordering::Less
			};
		}

		let a = priority(self);
		let b = priority(other);
		a.cmp(&b)
	}
}

impl PartialOrd for Visibility {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
