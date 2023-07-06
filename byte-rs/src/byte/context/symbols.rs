use super::*;

#[derive(Clone, Ord, PartialOrd)]
pub struct Symbol(Arc<String>);

impl Context {
	pub fn symbol<T: AsRef<str>>(symbol: T) -> Symbol {
		Context::get().read(|data| {
			let str = symbol.as_ref();
			let key = &SymbolCell::Ref(str as *const str);
			let value = {
				let set = data.symbols.set.read().unwrap();
				set.get(key).cloned()
			};
			if let Some(value) = value {
				// fast path
				value.get_symbol()
			} else {
				// insert a new symbol...
				let mut set = data.symbols.set.write().unwrap();
				if let Some(value) = set.get(key) {
					value.get_symbol()
				} else {
					let str = Arc::new(str.to_string());
					set.insert(SymbolCell::Owned(str.clone()));
					Symbol(str)
				}
			}
		})
	}
}

impl Symbol {
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}

	pub fn as_ptr(&self) -> *const u8 {
		self.0.as_ptr()
	}
}

impl Default for Symbol {
	fn default() -> Self {
		Self::from("")
	}
}

impl Hash for Symbol {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_ptr().hash(state)
	}
}

impl<T: AsRef<str>> From<T> for Symbol {
	fn from(value: T) -> Self {
		Context::symbol(value)
	}
}

impl Eq for Symbol {}

impl PartialEq for Symbol {
	fn eq(&self, other: &Self) -> bool {
		self.0.as_ptr() == other.0.as_ptr()
	}
}

impl<T: AsRef<str> + ?Sized> PartialEq<T> for Symbol {
	fn eq(&self, other: &T) -> bool {
		let other = other.as_ref();
		self.as_ptr() == other.as_ptr() || self.as_str() == other
	}
}

impl Display for Symbol {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl Debug for Symbol {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "<{:?}>", self.as_str())
	}
}

//====================================================================================================================//
// Internals
//====================================================================================================================//

#[derive(Default, Clone)]
pub(super) struct ContextSymbols {
	set: Arc<RwLock<HashSet<SymbolCell>>>,
}

#[derive(Clone)]
enum SymbolCell {
	Owned(Arc<String>),
	Ref(*const str),
}

impl SymbolCell {
	pub fn get_symbol(&self) -> Symbol {
		match self {
			SymbolCell::Owned(data) => Symbol(data.clone()),
			SymbolCell::Ref(_) => unreachable!("ref cannot be used as data"),
		}
	}
	pub fn as_str(&self) -> &str {
		match self {
			SymbolCell::Owned(str) => str.as_str(),
			SymbolCell::Ref(ptr) => unsafe { &**ptr },
		}
	}

	pub fn as_ptr(&self) -> *const u8 {
		self.as_str().as_ptr()
	}
}

impl PartialEq for SymbolCell {
	fn eq(&self, other: &Self) -> bool {
		self.as_ptr() == other.as_ptr() || self.as_str() == other.as_str()
	}
}

impl Eq for SymbolCell {}

impl Hash for SymbolCell {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

unsafe impl Send for SymbolCell {}
unsafe impl Sync for SymbolCell {}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn basic_symbols() {
		let a1 = Context::symbol("a");
		let a2 = Context::symbol("a");
		let b1 = Context::symbol("b");
		let b2 = Context::symbol("b".to_string());
		assert_eq!(a1, a2);
		assert_eq!(b1, b2);

		assert_eq!(a1.as_str(), "a");
		assert_eq!(a2.as_str(), "a");
		assert_eq!(b1.as_str(), "b");
		assert_eq!(b2.as_str(), "b");

		assert_eq!(a1.to_string(), "a");
		assert_eq!(format!("{a1:?}"), "<\"a\">");

		assert!(a1.as_ptr() == a2.as_ptr());
		assert!(b1.as_ptr() == b2.as_ptr());

		assert!(a1 == "a");
		assert!(a1 == "a".to_string());

		assert!(Symbol::default() == Context::symbol(""));
	}

	#[test]
	fn symbols_as_keys() {
		let a = Context::symbol("key-A");
		let b = Context::symbol("key-B");
		let c = Context::symbol("key-C");

		let mut map = HashMap::new();
		map.insert(a, 1);
		map.insert(b, 2);
		map.insert(c, 3);

		assert_eq!(map.get(&"key-A".into()), Some(&1));
		assert_eq!(map.get(&"key-A".to_string().into()), Some(&1));
		assert_eq!(map.get(&Context::symbol("key-A")), Some(&1));

		assert_eq!(map.get(&"key-B".into()), Some(&2));
		assert_eq!(map.get(&"key-C".into()), Some(&3));
	}
}
