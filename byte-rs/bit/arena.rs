use std::sync::{
	atomic::{AtomicUsize, Ordering},
	RwLock,
};

const ORDER: Ordering = Ordering::SeqCst;

/// Arena allocator for a single type.
///
/// The arena maintains ownership of allocated values and frees them at once
/// when the arena goes out of scope.
///
/// Elements are bulk allocated in pages to amortize allocation cost.
///
/// The arena does not exhibit exterior mutability and returned pointers and
/// references are stable for the lifetime of the arena.
pub struct Arena<T> {
	pages: RwLock<Vec<Vec<T>>>,
	page_size: usize,
	count: AtomicUsize,
}

impl<T> Arena<T> {
	pub fn new() -> Self {
		Self {
			pages: Default::default(),
			page_size: 256,
			count: Default::default(),
		}
	}

	pub fn len(&self) -> usize {
		self.count.load(ORDER)
	}

	pub fn push(&self, value: T) -> &T {
		self.push_with_index(value).0
	}

	pub fn push_with_index(&self, value: T) -> (&T, usize) {
		let (ptr, index) = self.alloc(value);
		let item = unsafe { &*ptr };
		(item, index)
	}

	pub fn get(&self, index: usize) -> *const T {
		let pages = self.pages.read().unwrap();
		let item = &pages[index / self.page_size][index % self.page_size];
		item as *const T
	}

	pub fn alloc(&self, value: T) -> (*mut T, usize) {
		let mut pages = self.pages.write().unwrap();
		let page = match pages.last_mut() {
			Some(page) => {
				if page.len() < page.capacity() {
					Some(page)
				} else {
					None
				}
			}
			None => None,
		};
		let page = if let Some(page) = page {
			page
		} else {
			let vec = Vec::with_capacity(self.page_size);
			pages.push(vec);
			pages.last_mut().unwrap()
		};

		unsafe {
			let index = self.count.fetch_add(1, ORDER);
			page.push(value);
			(page.as_mut_ptr().add(page.len() - 1), index)
		}
	}
}

impl<T> Default for Arena<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T> Drop for Arena<T> {
	fn drop(&mut self) {
		let mut pages = self.pages.write().unwrap();
		let pages = std::mem::take(&mut *pages);
		for mut page in pages.into_iter() {
			let len = page.len();
			let cap = page.capacity();
			let ptr = page.as_mut_ptr();
			unsafe {
				if std::mem::needs_drop::<T>() {
					for i in 0..len {
						std::ptr::drop_in_place(ptr.add(i));
					}
				}
				page.set_len(0);
				ptr.write_bytes(0xCD, cap);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use super::*;

	#[test]
	fn simple_arena() {
		let arena = Arena::new();
		let a = arena.push(1);
		let b = arena.push(2);
		let c = arena.push(3);
		assert_eq!(*a, 1);
		assert_eq!(*b, 2);
		assert_eq!(*c, 3);

		let mut items = Vec::new();
		for i in 0..1024 {
			let item = arena.push(i + 10_000);
			items.push(item);
		}

		for (n, it) in items.into_iter().enumerate() {
			assert_eq!(*it, n + 10_000);
		}
	}

	#[test]
	fn arena_index() {
		let arena = Arena::new();
		let (a, na) = arena.push_with_index(1);
		let (b, nb) = arena.push_with_index(2);
		let (c, nc) = arena.push_with_index(3);
		assert_eq!(*a, 1);
		assert_eq!(*b, 2);
		assert_eq!(*c, 3);

		assert_eq!(na, 0);
		assert_eq!(nb, 1);
		assert_eq!(nc, 2);
		assert_eq!(arena.len(), 3);

		for i in 0..1024 {
			let (_, n) = arena.push_with_index(i + 10_000);
			assert_eq!(n, i + 3);
			assert_eq!(arena.len(), i + 1 + 3);
		}
	}

	#[test]
	fn arena_drops() {
		let counter: Arc<RwLock<usize>> = Default::default();

		let arena = Arena::new();
		let num = 1000;

		for _ in 0..num {
			arena.push(DropCounter::new(counter.clone()));
		}

		assert_eq!(*counter.read().unwrap(), num);
		drop(arena);
		assert_eq!(*counter.read().unwrap(), 0);

		// Harness

		#[derive(Debug)]
		struct DropCounter(Arc<RwLock<usize>>);

		impl DropCounter {
			pub fn new(value: Arc<RwLock<usize>>) -> Self {
				{
					let mut value = value.write().unwrap();
					*value += 1;
				}
				Self(value)
			}
		}

		impl Drop for DropCounter {
			fn drop(&mut self) {
				let mut value = self.0.write().unwrap();
				*value -= 1;
			}
		}
	}
}
