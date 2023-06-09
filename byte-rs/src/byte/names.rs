use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};

use once_cell::sync::OnceCell;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Name(usize);

impl Name {
	pub fn from<T: AsRef<str>>(name: T) -> Self {
		let (_, index) = Self::intern(name.as_ref());
		Name(index)
	}

	pub fn from_u8<T: AsRef<[u8]>>(name: T) -> Self {
		let name = name.as_ref();
		let name = std::str::from_utf8(name).unwrap();
		Self::from(name)
	}

	pub fn as_str(&self) -> &'static str {
		Self::get_name(self.0)
	}

	fn get_name(index: usize) -> &'static str {
		let names = Self::names().read().unwrap();
		let name = names[index].as_str();
		unsafe { &*(name as *const str) }
	}

	fn intern(name: &str) -> (&'static str, usize) {
		let name_map = Self::name_map();
		{
			let name_map = name_map.read().unwrap();
			if let Some(value) = name_map.get(&name) {
				return *value;
			}
		}

		let mut name_map = name_map.write().unwrap();
		if let Some(value) = name_map.get(&name) {
			return *value;
		}

		let mut names = Self::names().write().unwrap();
		let index = names.len();
		names.push(name.to_string());

		// SAFETY: the `names` vec is static and the string itself never mutated
		let name = names[index].as_str();
		let name: &'static str = unsafe { &*(name as *const str) };

		name_map.insert(name, (name, index));

		(name, index)
	}

	fn names() -> &'static Arc<RwLock<Vec<String>>> {
		static NAMES: OnceCell<Arc<RwLock<Vec<String>>>> = OnceCell::new();
		NAMES.get_or_init(|| Default::default())
	}

	fn name_map() -> &'static Arc<RwLock<HashMap<&'static str, (&'static str, usize)>>> {
		static NAME_MAP: OnceCell<Arc<RwLock<HashMap<&'static str, (&'static str, usize)>>>> = OnceCell::new();
		NAME_MAP.get_or_init(|| Default::default())
	}
}

impl<T: AsRef<str>> From<T> for Name {
	fn from(value: T) -> Self {
		Name::from(value)
	}
}

impl std::fmt::Debug for Name {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let name = Name::get_name(self.0);
		write!(f, "Name({name:?})")
	}
}

impl std::fmt::Display for Name {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let name = Name::get_name(self.0);
		write!(f, "{name}")
	}
}

impl PartialEq<&str> for Name {
	fn eq(&self, other: &&str) -> bool {
		self.as_str() == *other
	}
}

impl PartialEq<String> for Name {
	fn eq(&self, other: &String) -> bool {
		self.as_str() == other
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn names() {
		let a = Name::from("a");
		let b = Name::from("banana");
		let c = Name::from("cargo build");

		let ax = a;

		assert_eq!(a, a.clone());
		assert_eq!(a, ax);
		assert_eq!(a, Name::from("a"));
		assert_eq!(a, "a");
		assert_eq!(b, String::from("banana"));
		assert_eq!(c.to_string(), String::from("cargo build"));

		assert!(a != b);
		assert!(a != c);
		assert!(b != c);

		assert_eq!(format!("{b}"), "banana");
		assert_eq!(format!("{b:?}"), "Name(\"banana\")");

		let x: Name = "abc".into();
		assert_eq!(x, Name::from("abc"));
	}
}
