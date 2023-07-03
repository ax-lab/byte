use std::{
	cell::Cell,
	cell::RefCell,
	rc::Rc,
	thread::{self, JoinHandle},
};

use super::*;

/// Provides a context that can be shared between functions across the current
/// stack for the current thread.
///
/// The context is immutable, but using `write` allows creating a new context
/// with changes. The new context is used as default until its dropped,
/// restoring the previous value.
///
/// Note that when cloning a context, only the original will maintain the
/// changed value and restore the previous value once dropped.
#[derive(Default)]
pub struct Context {
	data: Rc<ContextData>,
	dispose: Option<Rc<Cell<bool>>>,
}

impl Context {
	/// Return the context for the current thread.
	pub fn get() -> Self {
		Self {
			data: ContextData::get_active(),
			dispose: None,
		}
	}

	/// Apply changes to the context for the current thread using the given
	/// function. Return a new context value with the changes applied.
	///
	/// The changes persist until the returned [`Context`] is dropped.
	///
	/// Note that even when cloning, only the original [`Context`] value will
	/// maintain and restore the previous values.
	pub fn write<T, P: FnOnce(&mut ContextWriter) -> T>(mut self, action: P) -> Context {
		let mut writer = ContextWriter {
			data: Rc::make_mut(&mut self.data),
		};
		action(&mut writer);

		if let Some(flag) = self.dispose.take() {
			flag.set(true);
		}

		self.dispose = Some(ContextData::push_context(self.data.clone()));
		self
	}

	/// Span a new thread with the current context as the default.
	pub fn spawn<T: Send + 'static, F: FnOnce() -> T + Send + 'static>(&self, run: F) -> JoinHandle<T> {
		let data = self.data.as_ref().clone();
		thread::spawn(move || {
			ContextData::set_default(Rc::new(data));
			run()
		})
	}

	pub fn tab_size(&self) -> usize {
		let tab_size = self.read(|ctx| ctx.tab_size);
		if tab_size == 0 {
			DEFAULT_TAB_SIZE
		} else {
			tab_size
		}
	}

	fn read<T, P: FnOnce(&ContextData) -> T>(&self, predicate: P) -> T {
		predicate(&self.data)
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		if let Some(flag) = self.dispose.take() {
			flag.set(true);
		}
	}
}

impl Clone for Context {
	fn clone(&self) -> Self {
		Self {
			data: self.data.clone(),
			dispose: None, // only the original context controls the changes
		}
	}
}

//====================================================================================================================//
// Writer
//====================================================================================================================//

/// Writer used to modify a [`Context`] inside the `write` method.
pub struct ContextWriter<'a> {
	data: &'a mut ContextData,
}

impl<'a> ContextWriter<'a> {
	pub fn set_tab_size(&mut self, size: usize) -> usize {
		self.write(|ctx| std::mem::replace(&mut ctx.tab_size, size))
	}

	fn write<T, P: FnOnce(&mut ContextData) -> T>(&mut self, action: P) -> T {
		action(&mut self.data)
	}
}

//====================================================================================================================//
// Internals
//====================================================================================================================//

#[derive(Default, Clone)]
struct ContextData {
	tab_size: usize,
}

type ContextStack = VecDeque<(Rc<Cell<bool>>, Rc<ContextData>)>;

thread_local! {
	static CONTEXT_DEFAULT: RefCell<Rc<ContextData>> = Default::default();
	static CONTEXT_STACK: RefCell<ContextStack> = Default::default();
}

impl ContextData {
	pub fn set_default(new_default: Rc<ContextData>) {
		CONTEXT_DEFAULT.with(|default| {
			let mut default = default.borrow_mut();
			*default = new_default;
		});
	}

	pub fn get_active() -> Rc<Self> {
		CONTEXT_STACK.with(|stack| {
			let mut stack = stack.borrow_mut();
			Self::clear_disposed(&mut stack);
			stack
				.back()
				.cloned()
				.map(|(_, rc)| rc)
				.unwrap_or_else(|| CONTEXT_DEFAULT.with(|x| x.borrow().clone()))
		})
	}

	pub fn push_context(new_data: Rc<Self>) -> Rc<Cell<bool>> {
		CONTEXT_STACK.with(|stack| {
			let mut stack = stack.borrow_mut();
			Self::clear_disposed(&mut stack);

			let disposed_flag = Rc::new(Cell::new(false));
			stack.push_back((disposed_flag.clone(), new_data));
			disposed_flag
		})
	}

	fn clear_disposed(stack: &mut ContextStack) {
		while let Some((disposed_flag, ..)) = stack.back() {
			if disposed_flag.get() {
				stack.pop_back();
			} else {
				break;
			}
		}
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use std::sync::{Condvar, Mutex};

	use super::*;

	/// Test basic context stack semantics.
	#[test]
	pub fn defaults() {
		// default context
		let context = Context::get();
		assert_eq!(context.tab_size(), DEFAULT_TAB_SIZE);

		// change default
		let context = context.write(|ctx| ctx.set_tab_size(8));
		assert_eq!(context.tab_size(), 8);

		// check that changes are visible globally
		assert_eq!(Context::get().tab_size(), 8);
		check_tab_size(8);

		// a cloned context will inherit the values (and not change)
		let cloned = context.clone();
		assert_eq!(cloned.tab_size(), 8);

		// any new context will inherit the values (and not change)
		let another = Context::get();
		assert_eq!(another.tab_size(), 8);

		// check that nothing has changed
		check_tab_size(8);

		// make a new change to the "original" context
		let context = context.write(|ctx| ctx.set_tab_size(12));
		assert_eq!(context.tab_size(), 12);

		// change should not be visible to already created contexts
		assert_eq!(another.tab_size(), 8);
		assert_eq!(cloned.tab_size(), 8);

		// but should be visible elsewhere
		check_tab_size(12);

		// save this new change to check later
		let cloned2 = context.clone();

		// test that changes are properly stacked
		let sub = context.clone().write(|ctx| ctx.set_tab_size(11));
		assert_eq!(sub.tab_size(), 11);
		check_tab_size(11);
		drop(sub);
		check_tab_size(12);

		// dropping all changes should return to the original
		drop(context);
		check_tab_size(DEFAULT_TAB_SIZE);
		assert_eq!(Context::get().tab_size(), DEFAULT_TAB_SIZE);

		// created contexts should not be affected
		assert_eq!(another.tab_size(), 8);
		assert_eq!(cloned.tab_size(), 8);
		assert_eq!(cloned2.tab_size(), 12);

		// everything should still be back to normal after dropping all clones
		drop(another);
		check_tab_size(DEFAULT_TAB_SIZE);
		drop(cloned2);
		check_tab_size(DEFAULT_TAB_SIZE);
		drop(cloned);
		check_tab_size(DEFAULT_TAB_SIZE);
	}

	/// Test that interleaved context changes work properly.
	#[test]
	pub fn interleaved() {
		check_tab_size(DEFAULT_TAB_SIZE);

		let context1 = Context::get();
		let context2 = Context::get();

		let context1 = context1.write(|ctx| ctx.set_tab_size(1));
		check_tab_size(1);

		let context2 = context2.write(|ctx| ctx.set_tab_size(2));
		check_tab_size(2);

		drop(context1);
		check_tab_size(2);
		drop(context2);
		check_tab_size(DEFAULT_TAB_SIZE);

		let context1 = Context::get().write(|ctx| ctx.set_tab_size(1));
		check_tab_size(1);
		let context2 = context1.clone().write(|ctx| ctx.set_tab_size(2));
		check_tab_size(2);
		let context3 = context2.clone().write(|ctx| ctx.set_tab_size(3));
		check_tab_size(3);
		let context4 = context3.clone().write(|ctx| ctx.set_tab_size(4));
		check_tab_size(4);

		drop(context3);
		check_tab_size(4);

		drop(context2);
		check_tab_size(4);

		drop(context4);
		check_tab_size(1);

		drop(context1);
		check_tab_size(DEFAULT_TAB_SIZE);
	}

	/// Test context behavior with threads.
	#[test]
	pub fn threads() {
		check_tab_size(DEFAULT_TAB_SIZE);

		// set a new context
		let context = Context::get().write(|ctx| ctx.set_tab_size(8));
		check_tab_size(8);

		// normal threads won't see the context changes by default
		let t1 = thread::spawn(|| check_tab_size(DEFAULT_TAB_SIZE));
		t1.join().unwrap();

		let sub = context.clone().write(|ctx| ctx.set_tab_size(13));

		let main = Arc::new((Mutex::new(false), Condvar::new()));
		let wait = Arc::clone(&main);

		// spawning a thread through the context use that context as default,
		// even if the default has changed
		let t2 = context.spawn(move || {
			// inherited the changes from the spawn context
			check_tab_size(8);

			// make changes only in this thread
			let thread_context = Context::get().write(|ctx| ctx.set_tab_size(12));
			check_tab_size(12);
			drop(thread_context);
			check_tab_size(8);

			// wait for the original context to drop...
			let (done, var) = &*wait;
			let mut done = done.lock().unwrap();
			while !*done {
				done = var.wait(done).unwrap();
			}

			// ...it should not affect the thread
			check_tab_size(8);
		});

		// dropping contexts won't affect the already spawned thread
		check_tab_size(13);
		drop(sub);
		drop(context);
		check_tab_size(DEFAULT_TAB_SIZE);

		let (done, var) = &*main;
		*done.lock().unwrap() = true;
		var.notify_all();

		// the thread finishing should have no effect on the original
		t2.join().unwrap();
		check_tab_size(DEFAULT_TAB_SIZE);
	}

	fn check_tab_size(size: usize) {
		assert_eq!(Context::get().tab_size(), size);
	}
}
