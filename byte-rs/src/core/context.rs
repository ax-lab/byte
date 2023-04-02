use std::{
	any::{Any, TypeId},
	cell::RefCell,
	collections::HashMap,
};

/// Context is a lightweight struct with copy semantics that provides access
/// to all compile-time data for a compilation run.
#[derive(Copy, Clone)]
pub struct Context(&'static ContextPtr);

impl Context {
	pub fn new() -> Context {
		// once created, we never deallocate the context root data
		let data = Box::new(ContextPtr {
			data: Some(Box::new(ContextData::new())),
		});
		let data = Box::leak(data);
		Context(data)
	}

	pub fn save<T>(&self, value: T) -> &'static T {
		self.data().save(value)
	}

	fn data(&self) -> &ContextData {
		let data = &self.0.data.as_ref();
		let data = data.map(|x| x.as_ref());
		data.expect("trying to use destroyed context")
	}
}

struct ContextPtr {
	data: Option<Box<ContextData>>,
}

struct ContextData {
	arenas: RefCell<HashMap<TypeId, Box<dyn Any>>>,
}

impl ContextData {
	fn new() -> Self {
		ContextData {
			arenas: RefCell::new(Default::default()),
		}
	}

	fn save<T: 'static>(&self, value: T) -> &'static T {
		let mut arenas = self.arenas.borrow_mut();
		let id = TypeId::of::<T>();
		let entry = arenas.entry(id).or_insert_with(|| {
			let arena = Arena::<T>::new();
			let arena: Box<dyn Any> = Box::new(arena);
			arena
		});
		let arena: &Arena<T> = entry.downcast_ref().unwrap();
		arena.save(value)
	}
}

struct Arena<T> {
	size: usize,
	data: RefCell<ArenaData<T>>,
}

impl<T> Arena<T> {
	fn new() -> Self {
		Arena {
			size: 1024,
			data: RefCell::new(ArenaData {
				list: Vec::new(),
				next: 0,
			}),
		}
	}

	fn save(&self, value: T) -> &'static T {
		let mut data = self.data.borrow_mut();
		if data.list.len() == 0 || data.next == self.size {
			data.next = 0;
			data.list.push(Vec::with_capacity(self.size));
		}

		let next = data.next;
		data.next += 1;

		let data = data.list.last_mut().unwrap();
		data.push(value);
		unsafe { &*data.as_ptr().add(next) }
	}
}

struct ArenaData<T> {
	list: Vec<Vec<T>>,
	next: usize,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn arena_save() {
		let ctx = Context::new();
		let a = ctx.save("abc".to_string());
		let b = ctx.save("123".to_string());

		let c = ctx.save(RefCell::new(Vec::<i32>::new()));
		let d = ctx.save(RefCell::new(Vec::<i32>::new()));
		let e = ctx.save(RefCell::new(Vec::<String>::new()));

		assert_eq!(a, "abc");
		assert_eq!(b, "123");

		{
			let mut cx = c.borrow_mut();
			cx.push(1);
			cx.push(2);
			cx.push(3);

			let mut dx = d.borrow_mut();
			dx.push(100);
			dx.push(200);
			dx.push(300);

			let mut ex = e.borrow_mut();
			ex.push("A".to_string());
			ex.push("B".to_string());
		}

		let c = c.borrow();
		assert_eq!(c.len(), 3);
		assert_eq!(c[0], 1);
		assert_eq!(c[1], 2);
		assert_eq!(c[2], 3);

		let d = d.borrow();
		assert_eq!(d.len(), 3);
		assert_eq!(d[0], 100);
		assert_eq!(d[1], 200);
		assert_eq!(d[2], 300);

		let e = e.borrow();
		assert_eq!(e.len(), 2);
		assert_eq!(e[0], "A");
		assert_eq!(e[1], "B");
	}
}
