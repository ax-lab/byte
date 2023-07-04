use std::{
	cell::Cell,
	cell::RefCell,
	rc::Rc,
	thread::{self, JoinHandle},
};

use super::*;

pub mod sources;
pub use sources::*;

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

	pub fn with<T, P: FnOnce(&Self) -> T>(self, action: P) -> T {
		action(&self)
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

	fn read<T, P: FnOnce(&ContextData) -> T>(&self, reader: P) -> T {
		reader(&self.data)
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
	fn write<T, P: FnOnce(&mut ContextData) -> T>(&mut self, writer: P) -> T {
		writer(&mut self.data)
	}
}

//====================================================================================================================//
// Internals
//====================================================================================================================//

#[derive(Default, Clone)]
struct ContextData {
	sources: ContextDataSources,
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
		assert_eq!(context.tab_width(), DEFAULT_TAB_WIDTH);

		// change default
		let context = context.write(|ctx| ctx.set_tab_width(8));
		assert_eq!(context.tab_width(), 8);

		// check that changes are visible globally
		assert_eq!(Context::get().tab_width(), 8);
		check_tab_width(8);

		// a cloned context will inherit the values (and not change)
		let cloned = context.clone();
		assert_eq!(cloned.tab_width(), 8);

		// any new context will inherit the values (and not change)
		let another = Context::get();
		assert_eq!(another.tab_width(), 8);

		// check that nothing has changed
		check_tab_width(8);

		// make a new change to the "original" context
		let context = context.write(|ctx| ctx.set_tab_width(12));
		assert_eq!(context.tab_width(), 12);

		// change should not be visible to already created contexts
		assert_eq!(another.tab_width(), 8);
		assert_eq!(cloned.tab_width(), 8);

		// but should be visible elsewhere
		check_tab_width(12);

		// save this new change to check later
		let cloned2 = context.clone();

		// test that changes are properly stacked
		let sub = context.clone().write(|ctx| ctx.set_tab_width(11));
		assert_eq!(sub.tab_width(), 11);
		check_tab_width(11);
		drop(sub);
		check_tab_width(12);

		// dropping all changes should return to the original
		drop(context);
		check_tab_width(DEFAULT_TAB_WIDTH);
		assert_eq!(Context::get().tab_width(), DEFAULT_TAB_WIDTH);

		// created contexts should not be affected
		assert_eq!(another.tab_width(), 8);
		assert_eq!(cloned.tab_width(), 8);
		assert_eq!(cloned2.tab_width(), 12);

		// everything should still be back to normal after dropping all clones
		drop(another);
		check_tab_width(DEFAULT_TAB_WIDTH);
		drop(cloned2);
		check_tab_width(DEFAULT_TAB_WIDTH);
		drop(cloned);
		check_tab_width(DEFAULT_TAB_WIDTH);
	}

	/// Test that interleaved context changes work properly.
	#[test]
	pub fn interleaved() {
		check_tab_width(DEFAULT_TAB_WIDTH);

		let context1 = Context::get();
		let context2 = Context::get();

		let context1 = context1.write(|ctx| ctx.set_tab_width(1));
		check_tab_width(1);

		let context2 = context2.write(|ctx| ctx.set_tab_width(2));
		check_tab_width(2);

		drop(context1);
		check_tab_width(2);
		drop(context2);
		check_tab_width(DEFAULT_TAB_WIDTH);

		let context1 = Context::get().write(|ctx| ctx.set_tab_width(1));
		check_tab_width(1);
		let context2 = context1.clone().write(|ctx| ctx.set_tab_width(2));
		check_tab_width(2);
		let context3 = context2.clone().write(|ctx| ctx.set_tab_width(3));
		check_tab_width(3);
		let context4 = context3.clone().write(|ctx| ctx.set_tab_width(4));
		check_tab_width(4);

		drop(context3);
		check_tab_width(4);

		drop(context2);
		check_tab_width(4);

		drop(context4);
		check_tab_width(1);

		drop(context1);
		check_tab_width(DEFAULT_TAB_WIDTH);
	}

	/// Test context behavior with threads.
	#[test]
	pub fn threads() {
		check_tab_width(DEFAULT_TAB_WIDTH);

		// set a new context
		let context = Context::get().write(|ctx| ctx.set_tab_width(8));
		check_tab_width(8);

		// normal threads won't see the context changes by default
		let t1 = thread::spawn(|| check_tab_width(DEFAULT_TAB_WIDTH));
		t1.join().unwrap();

		let sub = context.clone().write(|ctx| ctx.set_tab_width(13));

		let main = Arc::new((Mutex::new(false), Condvar::new()));
		let wait = Arc::clone(&main);

		// spawning a thread through the context use that context as default,
		// even if the default has changed
		let t2 = context.spawn(move || {
			// inherited the changes from the spawn context
			check_tab_width(8);

			// make changes only in this thread
			let thread_context = Context::get().write(|ctx| ctx.set_tab_width(12));
			check_tab_width(12);
			drop(thread_context);
			check_tab_width(8);

			// wait for the original context to drop...
			let (done, var) = &*wait;
			let mut done = done.lock().unwrap();
			while !*done {
				done = var.wait(done).unwrap();
			}

			// ...it should not affect the thread
			check_tab_width(8);
		});

		// dropping contexts won't affect the already spawned thread
		check_tab_width(13);
		drop(sub);
		drop(context);
		check_tab_width(DEFAULT_TAB_WIDTH);

		let (done, var) = &*main;
		*done.lock().unwrap() = true;
		var.notify_all();

		// the thread finishing should have no effect on the original
		t2.join().unwrap();
		check_tab_width(DEFAULT_TAB_WIDTH);
	}

	fn check_tab_width(size: usize) {
		assert_eq!(Context::get().tab_width(), size);
	}
}
