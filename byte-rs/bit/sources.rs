use std::{
	collections::HashMap,
	fmt::{Debug, Display, Formatter},
	hash::Hash,
	sync::RwLock,
};

use super::{Arena, Result};

/// Loads and manages a collection of [`Source`] files.
#[derive(Default)]
pub struct SourceMap {
	arena: Arena<SourceData>,
	sources: RwLock<Vec<*const SourceData>>,
	by_path: RwLock<HashMap<String, Result<*const SourceData>>>,
}

/// Trait used by [`SourceMap`] to load source files.
pub trait SourceLoader {
	fn load(&self, path: &str) -> Result<(String, String)>;
}

struct SourceData {
	index: usize,
	name: String,
	text: String,
}

impl SourceMap {
	pub fn new() -> Self {
		let output = Self::default();
		// reserve index zero as an empty source
		output.add_source("(empty)".into(), "".into());
		output
	}

	/// Load a source file from a path if it has not been loaded before.
	///
	/// The source file is loaded using the given [`SourceLoader`] and the
	/// result saved for the given path.
	pub fn load_with<T: AsRef<str>, U: SourceLoader>(&self, path: T, loader: U) -> Result<Source> {
		let path = path.as_ref();
		let mut by_path = self.by_path.write().unwrap();
		let result = if let Some(result) = by_path.get(path) {
			result.clone()
		} else {
			let result = match loader.load(path) {
				Ok((name, text)) => {
					let data = self.add_source(name, text);
					Ok(data)
				}
				Err(err) => Err(err),
			};
			by_path.insert(path.to_string(), result.clone());
			result
		};
		match result {
			Ok(data) => Ok(Source(unsafe { &*data })),
			Err(err) => Err(err),
		}
	}

	pub fn by_index(&self, index: usize) -> Option<Source> {
		let data = self.sources.read().unwrap();
		data.get(index).map(|x| unsafe { Source(&**x) })
	}

	pub fn add_with_name<T: Into<String>, U: Into<String>>(&self, name: T, text: U) -> Source {
		let data = self.add_source(name.into(), text.into());
		Source(unsafe { &*data })
	}

	fn add_source(&self, name: String, text: String) -> *const SourceData {
		let mut sources = self.sources.write().unwrap();
		let index = sources.len();
		let data = SourceData { index, name, text };
		let data = self.arena.alloc(data);
		sources.push(data);
		data
	}
}

//====================================================================================================================//
// Source
//====================================================================================================================//

/// Source file from a [`SourceMap`].
#[derive(Copy, Clone)]
pub struct Source<'a>(&'a SourceData);

impl<'a> Source<'a> {
	/// Length of the source text in bytes.
	pub fn len(&self) -> usize {
		self.0.text.len()
	}

	/// Index of this source in the [`SourceMap`].
	pub fn index(&self) -> usize {
		self.0.index
	}

	/// Name for this source to be used in user facing messages.
	///
	/// This is NOT necessarily unique. The purpose of the name is only
	/// informational.
	///
	/// A source name is often derived from a file name. But that is not always
	/// the case (e.g. eval strings, user interactive line input).
	pub fn name(&self) -> &'a str {
		self.0.name.as_str()
	}

	/// Full source text.
	pub fn text(&self) -> &'a str {
		self.0.text.as_str()
	}

	fn as_ptr(&self) -> *const () {
		self.0 as *const _ as *const ()
	}
}

impl<'a> PartialEq for Source<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.as_ptr() == other.as_ptr()
	}
}

impl<'a> Eq for Source<'a> {}

impl<'a> Hash for Source<'a> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_ptr().hash(state)
	}
}

impl<'a> Display for Source<'a> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.name())
	}
}

impl<'a> Debug for Source<'a> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let len = self.len();
		write!(f, "<src:{}, len={len}>", self.name())
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	pub fn simple_source() {
		let map = SourceMap::new();
		let a = map.add_with_name("test1.txt", "test 1");
		let b = map.add_with_name("test2.txt", "test 2");
		assert_eq!(a.name(), "test1.txt");
		assert_eq!(b.name(), "test2.txt");
		assert_eq!(a.text(), "test 1");
		assert_eq!(b.text(), "test 2");
		assert_eq!(a.len(), 6);
		assert_eq!(b.len(), 6);

		let c = map.add_with_name("test3.txt", "test 3");
		assert_eq!(a.index(), 1);
		assert_eq!(b.index(), 2);
		assert_eq!(c.index(), 3);
	}

	#[test]
	pub fn source_loading() -> Result<()> {
		let map = SourceMap::new();
		let a = map.load_with("test1", TestLoader)?;
		assert_eq!(a.name(), "test1");
		assert_eq!(a.text(), "loaded from test1");

		let b1 = map.load_with("test2", TestLoader)?;
		let b2 = map.load_with("test2", TestLoader)?;
		assert_eq!(b1, b2);
		assert_eq!(b1.name(), "test2");
		assert_eq!(b1.text(), "loaded from test2");

		let err = map.load_with("some path", ErrLoader);
		assert_eq!(err, Err(String::from("not found: some path")));

		let still_an_err = map.load_with("some path", TestLoader);
		assert_eq!(err, still_an_err);

		Ok(())
	}

	#[test]
	pub fn source_equality() {
		let map = SourceMap::new();
		let a = map.add_with_name("a", "1");
		let b = map.add_with_name("a", "1");
		assert_eq!(a, a);
		assert_eq!(b, b);
		assert!(a != b);
	}

	struct TestLoader;

	impl SourceLoader for TestLoader {
		fn load(&self, path: &str) -> Result<(String, String)> {
			let name = path.to_string();
			let text = format!("loaded from {path}");
			Ok((name, text))
		}
	}

	struct ErrLoader;

	impl SourceLoader for ErrLoader {
		fn load(&self, path: &str) -> Result<(String, String)> {
			Err(format!("not found: {path}"))
		}
	}
}
