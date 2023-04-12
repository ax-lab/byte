use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

use crate::core::input::*;

#[derive(Clone)]
pub enum Str {
	Empty,
	Static(&'static str),
	Shared(Arc<String>),
	FromInput(Span),
	Interned(StrId),
}

impl Str {
	pub fn intern<S: AsRef<str>>(str: S) -> Str {
		let store = Self::get_store();
		let mut store = store.lock().unwrap();
		let id = store.add(str.as_ref());
		Str::Interned(id)
	}

	pub fn interned(&self) -> Str {
		if let Str::Interned(id) = &self {
			Str::Interned(*id)
		} else {
			Str::intern(self)
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Str::Empty => "",
			Str::Static(str) => str,
			Str::Shared(str) => &str,
			Str::FromInput(span) => span.text(),
			Str::Interned(id) => {
				let store = Self::get_store();
				let str = store.lock().unwrap().get(id);
				let str = Arc::as_ref(&str);
				let str = str as *const str;
				unsafe { str.as_ref().unwrap() }
			}
		}
	}

	fn get_store() -> &'static Mutex<StrStore> {
		use once_cell::sync::OnceCell;
		static STORE: OnceCell<Mutex<StrStore>> = OnceCell::new();
		STORE.get_or_init(|| {
			let store = StrStore::default();
			Mutex::new(store)
		})
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct StrId(usize);

#[derive(Default)]
struct StrStore {
	hash: HashMap<Arc<str>, StrId>,
	entries: Vec<Arc<str>>,
}

impl StrStore {
	pub fn get(&self, id: &StrId) -> Arc<str> {
		self.entries[id.0].clone()
	}

	pub fn add(&mut self, str: &str) -> StrId {
		if let Some(id) = self.hash.get(str) {
			*id
		} else {
			let id = StrId(self.entries.len());
			let str = str.to_string().into_boxed_str();
			let str = Arc::from(str);
			self.entries.push(Arc::clone(&str));
			self.hash.insert(str, id);
			id
		}
	}
}

impl std::fmt::Display for Str {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl std::fmt::Debug for Str {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.as_str())
	}
}

impl From<&'static str> for Str {
	fn from(value: &'static str) -> Self {
		Self::Static(value)
	}
}

impl From<String> for Str {
	fn from(value: String) -> Self {
		Self::Shared(Arc::new(value))
	}
}

impl From<Span> for Str {
	fn from(value: Span) -> Self {
		Self::FromInput(value)
	}
}

impl PartialEq for Str {
	fn eq(&self, other: &Self) -> bool {
		if let Str::Interned(a) = self {
			if let Str::Interned(b) = other {
				return a == b;
			}
		}
		self.as_str() == other.as_str()
	}
}

impl PartialEq<&str> for Str {
	fn eq(&self, other: &&str) -> bool {
		self.as_str() == *other
	}
}

impl PartialEq<Str> for &str {
	fn eq(&self, other: &Str) -> bool {
		other.as_str() == *self
	}
}

impl PartialEq<String> for Str {
	fn eq(&self, other: &String) -> bool {
		self.as_str() == other
	}
}

impl PartialEq<Str> for String {
	fn eq(&self, other: &Str) -> bool {
		other.as_str() == self
	}
}

impl Eq for Str {}

impl AsRef<str> for Str {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn str_empty() {
		assert_eq!(Str::Empty, Str::Empty);
		assert_eq!(Str::Empty, Str::Static(""));
		assert_eq!(Str::Empty, Str::from(""));
		assert_eq!(Str::Empty, Str::from(String::new()));
		assert_eq!(Str::Empty, Str::intern(""));
		assert_eq!(Str::Empty, Str::intern(String::new()));
		assert_eq!(Str::Empty, "");
		assert_eq!(Str::Empty, String::new());
		assert_eq!("", Str::Empty);
		assert_eq!(String::new(), Str::Empty);
	}

	#[test]
	fn str_eq() {
		let a1 = Str::Static("123");
		let a2 = Str::from("123");
		let a3 = Str::from(String::from("123"));
		let a4 = Str::intern("123");
		let a5 = Str::intern("123");

		let input = Input::open_str("", "[123]");
		let mut sta = input.start();
		sta.read();
		let mut end = sta.clone();
		end.read();
		end.read();
		end.read();
		let span = Span { sta, end };
		let a6 = Str::from(span);

		assert_eq!(a1, "123");
		assert_eq!(a1, String::from("123"));
		assert_eq!(a1, a1);
		assert_eq!(a1, a2);
		assert_eq!(a1, a3);
		assert_eq!(a1, a4);
		assert_eq!(a1, a5);
		assert_eq!(a1, a6);

		assert_eq!(a2, "123");
		assert_eq!(a2, String::from("123"));
		assert_eq!(a2, a1);
		assert_eq!(a2, a2);
		assert_eq!(a2, a3);
		assert_eq!(a2, a4);
		assert_eq!(a2, a5);
		assert_eq!(a2, a6);

		assert_eq!(a3, "123");
		assert_eq!(a3, String::from("123"));
		assert_eq!(a3, a1);
		assert_eq!(a3, a2);
		assert_eq!(a3, a3);
		assert_eq!(a3, a4);
		assert_eq!(a3, a5);
		assert_eq!(a3, a6);

		assert_eq!(a4, "123");
		assert_eq!(a4, String::from("123"));
		assert_eq!(a4, a1);
		assert_eq!(a4, a2);
		assert_eq!(a4, a3);
		assert_eq!(a4, a4);
		assert_eq!(a4, a5);
		assert_eq!(a4, a6);

		assert_eq!(a5, "123");
		assert_eq!(a5, String::from("123"));
		assert_eq!(a5, a1);
		assert_eq!(a5, a2);
		assert_eq!(a5, a3);
		assert_eq!(a5, a4);
		assert_eq!(a5, a5);
		assert_eq!(a5, a6);

		assert_eq!(a6, "123");
		assert_eq!(a6, String::from("123"));
		assert_eq!(a6, a1);
		assert_eq!(a6, a2);
		assert_eq!(a6, a3);
		assert_eq!(a6, a4);
		assert_eq!(a6, a5);
		assert_eq!(a6, a6);

		let s = "123";
		assert_eq!(s, a1);
		assert_eq!(s, a2);
		assert_eq!(s, a3);
		assert_eq!(s, a4);
		assert_eq!(s, a5);
		assert_eq!(s, a6);

		let s = String::from("123");
		assert_eq!(s, a1);
		assert_eq!(s, a2);
		assert_eq!(s, a3);
		assert_eq!(s, a4);
		assert_eq!(s, a5);
		assert_eq!(s, a6);

		assert_eq!(a1, a1.clone());
		assert_eq!(a2, a2.clone());
		assert_eq!(a3, a3.clone());
		assert_eq!(a4, a4.clone());
		assert_eq!(a5, a5.clone());
		assert_eq!(a6, a6.clone());
	}
}
