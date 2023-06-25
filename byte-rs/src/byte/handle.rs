use super::*;

/// Provides a weakly referenced handle to a shared value.
pub struct Handle<T: ?Sized> {
	data: Weak<T>,
}

impl<T: ?Sized> Handle<T> {
	pub fn new(data: &Arc<T>) -> Self {
		let data = Arc::downgrade(data);
		Self { data }
	}

	pub fn get(&self) -> HandleRef<T> {
		let data = self.upgrade();
		HandleRef { data }
	}

	pub fn get_map<U, P: Fn(&T) -> &U>(&self, predicate: P) -> HandleMap<T, U> {
		let main = self.upgrade();
		let data = predicate(&main) as *const U;
		HandleMap { main, data }
	}

	pub fn read<U, P: FnOnce(&T) -> U>(&self, predicate: P) -> U {
		let data = self.get();
		predicate(&data)
	}

	pub fn kind(&self) -> &'static str {
		std::any::type_name::<T>()
	}

	pub fn as_ptr(&self) -> *const T {
		self.data.as_ptr()
	}

	fn upgrade(&self) -> Arc<T> {
		if let Some(data) = self.data.upgrade() {
			data
		} else {
			panic!("orphaned Handle<{}>", self.kind());
		}
	}
}

impl<T> PartialEq for Handle<T> {
	fn eq(&self, other: &Self) -> bool {
		self.as_ptr() == other.as_ptr()
	}
}

impl<T> Eq for Handle<T> {}

impl<T> Debug for Handle<T> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let kind = self.kind();
		let ptr = self.as_ptr();
		write!(f, "<Handle {ptr:?} of {kind}>")
	}
}

impl<T> Clone for Handle<T> {
	fn clone(&self) -> Self {
		Handle {
			data: self.data.clone(),
		}
	}
}

//====================================================================================================================//
// HandleRef
//====================================================================================================================//

pub struct HandleRef<T: ?Sized> {
	data: Arc<T>,
}

impl<T: ?Sized> HandleRef<T> {
	pub(crate) fn to_inner(self) -> Arc<T> {
		self.data
	}
}

impl<T: ?Sized> Deref for HandleRef<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

//====================================================================================================================//
// HandleMap
//====================================================================================================================//

pub struct HandleMap<T: ?Sized, U: ?Sized> {
	main: Arc<T>,
	data: *const U,
}

impl<T: ?Sized, U: ?Sized> HandleMap<T, U> {
	pub fn map<V: ?Sized, Q: Fn(&U) -> &V>(&self, predicate: Q) -> HandleMap<T, V> {
		let data = self.deref();
		let data = predicate(data) as *const V;
		HandleMap {
			main: self.main.clone(),
			data,
		}
	}

	pub fn as_ptr(&self) -> *const U {
		self.data
	}
}

impl<T: ?Sized, U: ?Sized> Deref for HandleMap<T, U> {
	type Target = U;

	fn deref(&self) -> &Self::Target {
		let _ = self.main;
		unsafe { &*self.data }
	}
}

//----------------------------------------------------------------------------------------------------------------//
// Display & Debug
//----------------------------------------------------------------------------------------------------------------//

impl<T: ?Sized, U: ?Sized + Display> Display for HandleMap<T, U> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		(self as &U).fmt(f)
	}
}

impl<T: ?Sized, U: ?Sized + Debug> Debug for HandleMap<T, U> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		(self as &U).fmt(f)
	}
}

//----------------------------------------------------------------------------------------------------------------//
// Equality
//----------------------------------------------------------------------------------------------------------------//

impl<T: ?Sized, U: ?Sized + PartialEq> PartialEq for HandleMap<T, U> {
	fn eq(&self, other: &Self) -> bool {
		(self as &U) == (other as &U)
	}
}

impl<T: ?Sized, U: ?Sized + Eq> Eq for HandleMap<T, U> {}

impl<T: ?Sized, U: ?Sized + PartialEq> PartialEq<U> for HandleMap<T, U> {
	fn eq(&self, other: &U) -> bool {
		(self as &U) == other
	}
}

impl<T: ?Sized, U: ?Sized + PartialEq> PartialEq<&U> for HandleMap<T, U> {
	fn eq(&self, other: &&U) -> bool {
		(self as &U) == *other
	}
}
