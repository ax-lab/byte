use std::{
	any::TypeId,
	collections::{HashMap, VecDeque},
	sync::{Arc, RwLock},
};

use super::*;

//====================================================================================================================//
// Arena
//====================================================================================================================//

const DEFAULT_PAGE_SIZE: usize = 64;

/// Provides growable storage optimized for large collections of small objects.
///
/// The arena does not display exterior mutability. Added objects are read-only
/// and their reference does not change for the lifetime of the arena.
pub struct Arena<T> {
	page_size: usize,
	buffer: RwLock<VecDeque<Vec<T>>>,
}

impl<T> Arena<T> {
	pub fn new() -> Self {
		Self::new_with_page_size(0)
	}

	pub fn new_with_page_size(page_size: usize) -> Self {
		let page_size = if page_size == 0 {
			DEFAULT_PAGE_SIZE
		} else {
			page_size
		};
		Self {
			page_size,
			buffer: Default::default(),
		}
	}

	/// Number of items currently stored in the arena.
	pub fn count(&self) -> usize {
		let buffer = self.buffer.read().unwrap();
		if buffer.len() == 0 {
			0
		} else {
			let page_size = self.page_size;
			let last_page = buffer.back().map(|x| x.len()).unwrap();
			(buffer.len() - 1) * page_size + last_page
		}
	}

	/// Store a new item in the arena.
	pub fn store(&self, value: T) -> &T {
		let mut buffer = self.buffer.write().unwrap();
		for _ in 0..2 {
			if let Some(page) = buffer.back_mut() {
				let buffer = page.spare_capacity_mut();
				if buffer.len() > 0 {
					// Untangle the lifetime of the inner reference from the
					// lock lifetime.
					//
					// SAFETY: as the vector allocation is never changed and
					// item references always immutable, this reference will
					// remain valid for the duration of the arena lifetime.
					let output = unsafe {
						let output = buffer[0].write(std::mem::transmute(value));
						&*(output as *const T)
					};
					unsafe { page.set_len(page.len() + 1) };
					return output;
				}
			}
			buffer.push_back(Vec::with_capacity(self.page_size));
		}
		unreachable!();
	}

	/// Returns an iterator over the items **currently** in the arena.
	pub fn iter(&self) -> ArenaIterator<T> {
		ArenaIterator {
			arena: self,
			index: 0,
			count: self.count(),
		}
	}

	pub fn take(self) -> impl Iterator<Item = T> {
		let buffer = self.buffer.into_inner().unwrap();
		buffer.into_iter().flatten()
	}

	fn get(&self, index: usize) -> &T {
		let buffer = self.buffer.read().unwrap();
		let page_size = self.page_size;
		let page = index / page_size;
		let index = index % page_size;
		let page = &buffer[page];
		unsafe { &*(&page[index] as *const T) }
	}
}

impl<T> Default for Arena<T> {
	fn default() -> Self {
		Self::new()
	}
}

pub struct ArenaIterator<'a, T> {
	arena: &'a Arena<T>,
	index: usize,
	count: usize,
}

impl<'a, T> Iterator for ArenaIterator<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.count {
			let item = self.arena.get(self.index);
			self.index += 1;
			Some(item)
		} else {
			None
		}
	}
}

impl<T> HasTraits for Arena<T> {}

//====================================================================================================================//
// ArenaSet
//====================================================================================================================//

/// Stores a set of [`Arena`] for an arbitrary set of types.
#[derive(Default)]
pub struct ArenaSet {
	page_size: usize,
	by_type: Arc<RwLock<HashMap<TypeId, Value>>>,
}

impl ArenaSet {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn new_with_page_size(page_size: usize) -> Self {
		Self {
			page_size,
			by_type: Default::default(),
		}
	}

	pub fn get<T: Cell>(&self) -> &Arena<T> {
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
					.or_insert_with(|| Value::from(Arena::<T>::new_with_page_size(self.page_size)));
				map.clone()
			};
			map
		};

		// SAFETY: the original `Value` is held by the `by_type` map, so this
		// reference is valid while self is valid
		let ptr = map.get::<Arena<T>>().unwrap() as *const Arena<T>;
		unsafe { &*ptr }
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn arena() {
		let arena = Arena::new_with_page_size(17);
		let mut refs = Vec::new();
		for i in 0..100_000 {
			let item = arena.store(i + 1);
			refs.push(item);
		}

		let mut count = 0;
		for (i, n) in arena.iter().enumerate() {
			assert_eq!(*n, i + 1, "item #{i} is invalid");
			count += 1;
		}

		assert_eq!(count, refs.len());
		assert_eq!(count, arena.count());

		for (i, n) in refs.into_iter().enumerate() {
			assert_eq!(*n, i + 1, "reference #{i} is invalid");
		}

		let items: Vec<_> = arena.take().collect();
		assert_eq!(items.len(), count);
		for i in 0..count {
			assert_eq!(items[i], i + 1, "taken item #{i} is invalid");
		}
	}

	#[test]
	fn arena_set() {
		let arenas = ArenaSet::new_with_page_size(5);
		let ints = arenas.get::<i32>();
		let strings = arenas.get::<String>();

		ints.store(1);
		ints.store(2);
		ints.store(3);
		ints.store(5);
		ints.store(7);
		ints.store(11);
		ints.store(13);
		ints.store(17);

		strings.store("abc".into());
		strings.store("123".into());
		strings.store("the answer to everything".into());
		strings.store("42".into());

		drop(ints);
		drop(strings);

		let arena = arenas.get::<i32>();
		assert!(arena.count() == 8);

		let values: Vec<_> = arena.iter().cloned().collect();
		assert_eq!(values, vec![1, 2, 3, 5, 7, 11, 13, 17]);

		let arena = arenas.get::<String>();
		assert!(arena.count() == 4);

		let values: Vec<_> = arena.iter().collect();
		assert_eq!(values, vec!["abc", "123", "the answer to everything", "42"]);
	}
}
