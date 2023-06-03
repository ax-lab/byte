use std::{
	any::TypeId,
	collections::HashMap,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Arc, RwLock,
	},
};

use super::*;

/// Globally unique handle for arbitrary values.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Handle(usize);

impl Handle {
	pub fn new() -> Self {
		static COUNTER: AtomicUsize = AtomicUsize::new(1);
		let id = COUNTER.fetch_add(1, Ordering::SeqCst);
		Self(id)
	}
}

/// Map [`Handle`] to arbitrary values in a type-safe manner.
#[derive(Default, Clone)]
pub struct HandleMap {
	by_type: Arc<RwLock<HashMap<TypeId, Value>>>,
}

impl HandleMap {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add<T: Cell>(&mut self, value: T) -> Handle {
		let map = self.get_map::<T>();
		map.add(value)
	}

	pub fn get<T: Cell>(&self, handle: Handle) -> Option<&T> {
		let map = self.get_map::<T>();
		map.get(handle)
	}

	fn get_map<T: Cell>(&self) -> &HandleMapFor<T> {
		let map = {
			let by_type = self.by_type.read().unwrap();
			let key = TypeId::of::<T>();
			let map = if let Some(map) = by_type.get(&key) {
				map.clone()
			} else {
				drop(by_type);
				let mut by_type = self.by_type.write().unwrap();
				let map = by_type
					.entry(key)
					.or_insert_with(|| Value::from(HandleMapFor::<T>::new()));
				map.clone()
			};
			map
		};

		// SAFETY: the original `Value` is held by the `by_type` map, so this
		// reference is valid while self is valid
		let ptr = map.get::<HandleMapFor<T>>().unwrap() as *const HandleMapFor<T>;
		unsafe { &*ptr }
	}
}

struct HandleMapFor<T> {
	values: Arc<RwLock<HashMap<Handle, T>>>,
}

impl<T> HandleMapFor<T> {
	pub fn new() -> Self {
		Self {
			values: Default::default(),
		}
	}

	pub fn add(&self, value: T) -> Handle {
		let handle = Handle::new();
		let mut values = self.values.write().unwrap();
		values.insert(handle, value);
		handle
	}

	pub fn get(&self, handle: Handle) -> Option<&T> {
		let values = self.values.read().unwrap();
		// SAFETY: the lock only applies to the outer HashMap, so let the
		// reference to the inner immutable value escape the lock lifetime
		values.get(&handle).map(|x| unsafe { &*(x as *const T) })
	}
}

impl<T> HasTraits for HandleMapFor<T> {}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn handle_map() {
		let mut map = HandleMap::new();

		let a1 = map.add("abc");
		let a2 = map.add("123");
		let b1 = map.add(String::from("[abc]"));
		let b2 = map.add(String::from("[123]"));
		let c1 = map.add(42);
		let c2 = map.add(100);
		let c3 = map.add(1024);

		assert!(map.get::<()>(Handle::new()).is_none());
		assert!(map.get::<i32>(Handle::new()).is_none());
		assert!(map.get::<i32>(a1).is_none());
		assert!(map.get::<String>(a1).is_none());

		let a1 = map.get::<&'static str>(a1).cloned().unwrap();
		let a2 = map.get::<&'static str>(a2).cloned().unwrap();
		assert_eq!(a1, "abc");
		assert_eq!(a2, "123");

		let b1: &String = map.get(b1).unwrap();
		let b2: &String = map.get(b2).unwrap();
		assert_eq!(b1, "[abc]");
		assert_eq!(b2, "[123]");

		let c1: i32 = map.get(c1).cloned().unwrap();
		let c2: i32 = map.get(c2).cloned().unwrap();
		let c3: i32 = map.get(c3).cloned().unwrap();
		assert_eq!(c1, 42);
		assert_eq!(c2, 100);
		assert_eq!(c3, 1024);
	}
}
