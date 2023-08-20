use std::{
	collections::HashSet,
	fmt::{Debug, Display, Formatter},
	hash::Hash,
	sync::RwLock,
};

/// Symbols are interned strings managed by a [`SymbolSet`].
#[derive(Copy, Clone)]
pub struct Symbol<'a>(&'a str);

/// Manages a set of [`Symbol`] and provides methods for interning and
/// retrieving strings.
#[derive(Default)]
pub struct SymbolSet {
	entries: RwLock<HashSet<String>>,
}

impl SymbolSet {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get<T: AsRef<str>>(&self, str: T) -> Symbol {
		let mut entries = self.entries.write().unwrap();
		let str = str.as_ref();
		let str = if let Some(entry) = entries.get(str) {
			entry.as_str()
		} else {
			entries.insert(str.to_string());
			entries.get(str).unwrap().as_str()
		};
		let str = str as *const _;
		Symbol(unsafe { &*str })
	}
}

impl<'a> Symbol<'a> {
	pub fn as_str(&self) -> &str {
		self.0
	}

	pub fn as_ptr(&self) -> *const u8 {
		self.as_str().as_ptr()
	}
}

impl<'a> PartialEq for Symbol<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.as_ptr() == other.as_ptr()
	}
}

impl<'a> Eq for Symbol<'a> {}

impl<'a> Hash for Symbol<'a> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_ptr().hash(state);
	}
}

impl<'a> Display for Symbol<'a> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl<'a> Debug for Symbol<'a> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{self}")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn symbols() {
		let symbols = SymbolSet::new();
		let a = symbols.get("");
		let b = symbols.get("1234567890");
		let c = symbols.get("abc");
		let d = symbols.get("abc");

		assert_eq!(a.as_str(), "");
		assert_eq!(b.as_str(), "1234567890");
		assert_eq!(c.as_str(), "abc");
		assert_eq!(d.as_str(), "abc");

		assert!(c == c);
		assert!(c == d);
		assert!(c != a);
		assert!(c != b);

		assert_eq!(c, symbols.get("abc"));
		assert_eq!(c.as_ptr(), symbols.get("abc").as_ptr());
	}
}
