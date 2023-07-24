//! Arena implementations for the node engine.

use super::*;

//====================================================================================================================//
// Arena
//====================================================================================================================//

/// Simple typed arena allocator for `T` using raw pointers.
///
/// This will drop all allocated values when the arena is dropped.
pub struct Arena<T> {
	inner: RawArena,
	_elem: PhantomData<T>,
}

impl<T> Arena<T> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn new_with_page(page: usize) -> Self {
		let inner = RawArena::for_type::<T>(page);
		Self {
			inner,
			_elem: Default::default(),
		}
	}

	pub fn push(&self, value: T) -> *mut T {
		self.inner.push(value)
	}
}

impl<T> Default for Arena<T> {
	fn default() -> Self {
		const PAGE_SIZE: usize = 4096;
		let page_count = PAGE_SIZE / std::mem::size_of::<T>();
		let page_count = std::cmp::max(page_count, 64);
		Self::new_with_page(page_count)
	}
}

//====================================================================================================================//
// Buffer
//====================================================================================================================//

/// Untyped arena allocator for variable sized allocations of plain [`Copy`]
/// values using raw pointers.
///
/// Small allocations are served from a [`RawArena`] bucket close to the
/// requested size, while larger allocations use a [`Vec<u8>`].
///
/// All memory will be freed at once when dropping the buffer.
pub struct Buffer {
	list_08: RawArena,
	list_16: RawArena,
	list_32: RawArena,
	list_64: RawArena,
	list_128: RawArena,
	list_256: RawArena,
	large: RwLock<VecDeque<Vec<u8>>>,
}

impl Buffer {
	pub fn alloc(&self, size: usize) -> *mut u8 {
		unsafe {
			if size <= 8 {
				self.list_08.alloc()
			} else if size <= 16 {
				self.list_16.alloc()
			} else if size <= 32 {
				self.list_32.alloc()
			} else if size <= 64 {
				self.list_64.alloc()
			} else if size <= 128 {
				self.list_128.alloc()
			} else if size <= 256 {
				self.list_256.alloc()
			} else {
				let mut data = Vec::with_capacity(size);
				let data_ptr = data.as_mut_ptr();
				let mut large = self.large.write().unwrap();
				large.push_back(data);
				data_ptr
			}
		}
	}

	pub fn push<T: Copy>(&self, value: T) -> *mut T {
		let size = std::mem::size_of::<T>();
		let data = self.alloc(size) as *mut T;
		unsafe {
			data.write(value);
			data
		}
	}
}

impl Default for Buffer {
	fn default() -> Self {
		Self {
			list_08: RawArena::new(8, 1024),
			list_16: RawArena::new(16, 512),
			list_32: RawArena::new(32, 256),
			list_64: RawArena::new(64, 128),
			list_128: RawArena::new(128, 64),
			list_256: RawArena::new(256, 32),
			large: Default::default(),
		}
	}
}

//====================================================================================================================//
// RawArena
//====================================================================================================================//

/// Fixed size allocator of raw pointers.
pub struct RawArena {
	elem: usize,
	page: usize,
	drop: Option<fn(*mut u8)>,
	pages: RwLock<Vec<RawArenaPage>>,
	current: AtomicPtr<RawArenaPage>,
}

struct RawArenaPage {
	arena: *const RawArena,
	data: *mut u8,
	next: AtomicUsize,
}

impl Drop for RawArenaPage {
	fn drop(&mut self) {
		let arena = unsafe { &*self.arena };
		let page = arena.page_size();
		let data = self.data;
		if let Some(drop) = arena.drop {
			let elem = arena.elem;
			let last = self.next.load(Ordering::SeqCst);
			let last = std::cmp::min(last, page); // next can go past size in alloc
			let last = unsafe { data.add(last) };
			let mut cur = data;
			while cur < last {
				drop(cur);
				cur = unsafe { cur.add(elem) };
			}
		}

		let data = unsafe {
			std::ptr::write_bytes(data, 0xD0, page);
			Vec::from_raw_parts(self.data, 0, page)
		};
		drop(data);
	}
}

impl RawArena {
	/// Create a raw arena for an `elem` sized allocation and without a drop
	/// function.
	pub fn new(elem: usize, page: usize) -> Self {
		Self::new_with_drop(elem, page, None)
	}

	/// Create a raw arena for an `elem` sized allocation and an optional drop
	/// function.
	pub fn new_with_drop(elem: usize, page: usize, drop: Option<fn(*mut u8)>) -> Self {
		Self {
			elem,
			page,
			drop,
			pages: Default::default(),
			current: Default::default(),
		}
	}

	/// Create a raw arena of the appropriate size and any required drop
	/// function for a type `T`.
	pub fn for_type<T>(page: usize) -> Self {
		let elem = std::mem::size_of::<T>();
		if std::mem::needs_drop::<T>() {
			let drop = |ptr: *mut u8| {
				let ptr = ptr as *mut T;
				unsafe { std::ptr::drop_in_place(ptr) }
			};
			Self::new_with_drop(elem, page, Some(drop))
		} else {
			Self::new(elem, page)
		}
	}

	pub fn alloc(&self) -> *mut u8 {
		loop {
			let ptr = self.current.load(Ordering::SeqCst);
			let ptr = if ptr.is_null() {
				let mut pages = self.pages.write().unwrap();
				let last = self.current.load(Ordering::SeqCst);
				if last.is_null() {
					let data = Vec::with_capacity(self.page_size());
					let size = data.capacity();
					let next = AtomicUsize::new(0);

					let mut data = ManuallyDrop::new(data);
					let data = data.as_mut_ptr();
					let page = RawArenaPage {
						arena: self,
						data,
						next,
					};
					pages.push(page);
					let page = pages.last_mut().unwrap() as *mut RawArenaPage;
					self.current.store(page, Ordering::SeqCst);
					page
				} else {
					last
				}
			} else {
				ptr
			};

			let mut page = unsafe { NonNull::new_unchecked(ptr) };
			let page = unsafe { page.as_mut() };
			let next = page.next.fetch_add(self.elem, Ordering::SeqCst);
			if next < self.page_size() {
				unsafe {
					let data = page.data.add(next);
					return data;
				}
			} else {
				let _ = self
					.current
					.compare_exchange(ptr, std::ptr::null_mut(), Ordering::SeqCst, Ordering::SeqCst);
			}
		}
	}

	/// Allocate a buffer for the given `T` value.
	///
	/// NOTE: if `T` needs drop, the drop function must be supplied to the
	/// arena using `RawArena::new_with_drop`.
	pub fn push<T>(&self, value: T) -> *mut T {
		assert_eq!(std::mem::size_of::<T>(), self.elem);
		let data = self.alloc() as *mut T;
		unsafe {
			data.write(value);
			data
		}
	}

	#[inline(always)]
	fn page_size(&self) -> usize {
		self.page * self.elem
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use std::{sync::Arc, time::Instant};

	use super::*;

	#[test]
	fn arena_simple() {
		let mut arena = Arena::new();
		let mut pointers = Vec::new();
		for i in 0..2048 {
			let ptr = arena.push(i);
			pointers.push(ptr);
		}

		assert_eq!(pointers.len(), 2048);
		for (n, ptr) in pointers.into_iter().enumerate() {
			let value = unsafe { ptr.as_ref().unwrap() };
			assert_eq!(value, &n);
		}
	}

	#[test]
	fn arena_drop() {
		let counter = Arc::new(RwLock::new(0));

		let mut arena = Arena::new();
		for i in 1..=2048 {
			arena.push(Value::new(i, counter.clone()));
		}

		assert_eq!(*counter.read().unwrap(), 2098176);
		drop(arena);
		assert_eq!(*counter.read().unwrap(), 0);

		struct Value(i32, Arc<RwLock<i32>>);

		impl Value {
			pub fn new(val: i32, counter: Arc<RwLock<i32>>) -> Self {
				{
					let mut counter = counter.write().unwrap();
					*counter += val;
				}
				Self(val, counter)
			}
		}

		impl Drop for Value {
			fn drop(&mut self) {
				let counter = &mut self.1;
				let mut counter = counter.write().unwrap();
				*counter -= self.0;
			}
		}
	}

	#[allow(unused)]
	fn arena_benchmark() {
		let now = Instant::now();
		let mut arena = Arena::new();
		let mut counter = 0;
		for i in 0..2_000_000 {
			arena.push(i);
			counter += 1;
		}

		let elapsed = now.elapsed();
		let average = elapsed / counter;
		println!("stored {counter} items in {elapsed:?} (avg: {average:?})");
	}
}
