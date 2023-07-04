use super::*;

pub trait CanHandle: Sized {
	type Data;

	fn inner_data(&self) -> &Arc<Self::Data>;

	fn from_inner_data(data: Arc<Self::Data>) -> Self;

	fn handle(&self) -> Handle<Self> {
		Handle::new(self)
	}

	fn new_cyclic<P: FnOnce(Handle<Self>) -> Self::Data>(predicate: P) -> Self {
		let data = Arc::new_cyclic(|data: &Weak<Self::Data>| {
			let handle = Handle { data: data.clone() };
			predicate(handle)
		});
		Self::from_inner_data(data)
	}
}

/// Provides a weakly referenced handle to a shared value.
pub struct Handle<T: CanHandle> {
	data: Weak<T::Data>,
}

impl<T: CanHandle> Handle<T> {
	pub fn new(value: &T) -> Self {
		let data = value.inner_data();
		let data = Arc::downgrade(data);
		Self { data }
	}

	pub fn get(&self) -> HandleRef<T> {
		let data = self.upgrade();
		let data = T::from_inner_data(data);
		HandleRef { data }
	}

	pub unsafe fn get_mut(&self) -> HandleMut<T> {
		let data = self.upgrade();
		let data = T::from_inner_data(data);
		HandleMut { data }
	}

	pub fn get_map<U, P: Fn(&T) -> &U>(&self, predicate: P) -> HandleMap<T, U> {
		let main = self.upgrade();
		let main = T::from_inner_data(main);
		let data = predicate(&main) as *const U;
		HandleMap::Owned { main, data }
	}

	pub fn read<U, P: FnOnce(&T) -> U>(&self, predicate: P) -> U {
		let data = self.get();
		predicate(&data)
	}

	pub fn kind(&self) -> &'static str {
		std::any::type_name::<T>()
	}

	pub fn as_ptr(&self) -> *const () {
		self.data.as_ptr() as *const ()
	}

	fn upgrade(&self) -> Arc<T::Data> {
		if let Some(data) = self.data.upgrade() {
			data
		} else {
			panic!("orphaned Handle<{}>", self.kind());
		}
	}
}

impl<T: CanHandle> PartialEq for Handle<T> {
	fn eq(&self, other: &Self) -> bool {
		self.as_ptr() == other.as_ptr()
	}
}

impl<T: CanHandle> Eq for Handle<T> {}

impl<T: CanHandle> Debug for Handle<T> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let kind = self.kind();
		let ptr = self.as_ptr();
		write!(f, "<Handle {ptr:?} of {kind}>")
	}
}

impl<T: CanHandle> Clone for Handle<T> {
	fn clone(&self) -> Self {
		Handle {
			data: self.data.clone(),
		}
	}
}

//====================================================================================================================//
// HandleRef
//====================================================================================================================//

pub struct HandleRef<T: CanHandle> {
	data: T,
}

impl<T: CanHandle> Deref for HandleRef<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

//====================================================================================================================//
// HandleMut
//====================================================================================================================//

pub struct HandleMut<T: CanHandle> {
	data: T,
}

impl<T: CanHandle> Deref for HandleMut<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<T: CanHandle> DerefMut for HandleMut<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.data
	}
}

//====================================================================================================================//
// HandleMap
//====================================================================================================================//

pub enum HandleMap<T: CanHandle, U: ?Sized> {
	Static(*const U),
	Owned { main: T, data: *const U },
}

impl<T: CanHandle, U: ?Sized> HandleMap<T, U> {
	pub fn new_static(value: &'static U) -> Self {
		Self::Static(value)
	}

	pub fn map<V: ?Sized, Q: Fn(&U) -> &V>(&self, predicate: Q) -> HandleMap<T, V> {
		let data = predicate(self.deref()) as *const V;
		match self {
			HandleMap::Static(..) => HandleMap::Static(data),
			HandleMap::Owned { main, .. } => {
				let main = main.inner_data().clone();
				let main = T::from_inner_data(main);
				HandleMap::Owned { main, data }
			}
		}
	}

	pub fn as_ptr(&self) -> *const U {
		match self {
			HandleMap::Static(data) => *data,
			HandleMap::Owned { data, .. } => *data,
		}
	}
}

impl<T: CanHandle, U: ?Sized> Deref for HandleMap<T, U> {
	type Target = U;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.as_ptr() }
	}
}

//----------------------------------------------------------------------------------------------------------------//
// Display & Debug
//----------------------------------------------------------------------------------------------------------------//

impl<T: CanHandle, U: ?Sized + Display> Display for HandleMap<T, U> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		(self as &U).fmt(f)
	}
}

impl<T: CanHandle, U: ?Sized + Debug> Debug for HandleMap<T, U> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		(self as &U).fmt(f)
	}
}

//----------------------------------------------------------------------------------------------------------------//
// Equality
//----------------------------------------------------------------------------------------------------------------//

impl<T: CanHandle, U: ?Sized + PartialEq> PartialEq for HandleMap<T, U> {
	fn eq(&self, other: &Self) -> bool {
		(self as &U) == (other as &U)
	}
}

impl<T: CanHandle, U: ?Sized + Eq> Eq for HandleMap<T, U> {}

impl<T: CanHandle, U: ?Sized + PartialEq> PartialEq<U> for HandleMap<T, U> {
	fn eq(&self, other: &U) -> bool {
		(self as &U) == other
	}
}

impl<T: CanHandle, U: ?Sized + PartialEq> PartialEq<&U> for HandleMap<T, U> {
	fn eq(&self, other: &&U) -> bool {
		(self as &U) == *other
	}
}
