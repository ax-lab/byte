use std::{
	ops::{Deref, DerefMut},
	sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::core::str::*;
use crate::lexer::*;

use super::*;

/// A scope maintains the state necessary for the node parsing.
#[derive(Clone)]
pub struct Scope {
	data: Arc<RwLock<ScopeData>>,
	errors: Arc<RwLock<ErrorList>>,
}

#[derive(Default)]
pub struct ScopeData {
	root: Option<Scope>,
	parent: Option<Scope>,
	children: Vec<Scope>,
	previous: Option<Scope>,
}

impl Scope {
	pub fn new() -> Self {
		let data = ScopeData {
			..Default::default()
		};
		Self {
			errors: Arc::new(RwLock::new(ErrorList::new())),
			data: Arc::new(RwLock::new(data)),
		}
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Hierarchy
	//----------------------------------------------------------------------------------------------------------------//

	/// Returns the root scope for the current hierarchy. Will return the
	/// current scope if it's the root.
	pub fn root(&self) -> Scope {
		let data = self.data.read().unwrap();
		if let Some(root) = &data.root {
			root.clone()
		} else {
			self.clone()
		}
	}

	/// Returns the parent scope for the current scope or [`None`] if this is
	/// the root scope.
	pub fn parent(&self) -> Option<Scope> {
		let data = self.data.read().unwrap();
		if let Some(parent) = &data.parent {
			Some(parent.clone())
		} else {
			None
		}
	}

	/// Creates a new child scope. A child scope can see definitions from the
	/// parent scope, but is its own defined scope.
	pub fn new_child(&mut self) -> Scope {
		let mut data = self.data.write().unwrap();
		let child = ScopeData {
			root: data.root.clone().or(Some(self.clone())),
			parent: Some(self.clone()),
			..Default::default()
		};
		let child = Scope {
			errors: self.errors.clone(),
			data: Arc::new(RwLock::new(child)),
		};
		data.children.push(child.clone());
		child
	}

	/// Creates a new scope with the same level and definitions as the current
	/// scope, but isolating further changes from the current scope.
	pub fn inherit(&self) -> Scope {
		let data = self.data.read().unwrap();
		let next = ScopeData {
			root: data.root.clone(),
			parent: data.parent.clone(),
			previous: Some(self.clone()),
			..Default::default()
		};
		let next = Scope {
			errors: self.errors.clone(),
			data: Arc::new(RwLock::new(next)),
		};
		next
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Error handling
	//----------------------------------------------------------------------------------------------------------------//

	pub fn errors_mut(&mut self) -> ScopeErrorsRefMut {
		ScopeErrorsRefMut {
			errors: self.errors.write().unwrap(),
		}
	}

	pub fn has_errors(&self) -> bool {
		let errors = self.errors.read().unwrap();
		!errors.empty()
	}

	pub fn errors(&self) -> ErrorList {
		self.errors.read().unwrap().clone()
	}

	pub fn error_if<T: ToString>(&mut self, span: Option<Span>, error: T) {
		if !self.has_errors() {
			self.errors_mut().at(span, error);
		}
	}

	//----------------------------------------------------------------------------------------------------------------//

	pub fn get(&self, name: Str) -> ScopeCell {
		todo!()
	}

	fn get_ref<T, F: Fn(&ScopeData) -> &T>(&self, read: F) -> ScopeRef<T, F> {
		ScopeRef {
			data: self.data.read().unwrap(),
			read,
		}
	}

	fn get_ref_mut<T, F: Fn(&mut ScopeData) -> &mut T>(&self, read: F) -> ScopeRefMut<T, F> {
		ScopeRefMut {
			data: self.data.write().unwrap(),
			read,
		}
	}
}

pub struct ScopeCell {}

impl ScopeCell {
	pub fn resolve(&self) {
		todo!()
	}
}

//--------------------------------------------------------------------------------------------------------------------//
// Reference types
//--------------------------------------------------------------------------------------------------------------------//

pub struct ScopeErrorsRefMut<'a> {
	errors: RwLockWriteGuard<'a, ErrorList>,
}

impl<'a> Deref for ScopeErrorsRefMut<'a> {
	type Target = ErrorList;

	fn deref(&self) -> &Self::Target {
		&self.errors
	}
}

impl<'a> DerefMut for ScopeErrorsRefMut<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.errors
	}
}

pub struct ScopeRef<'a, T, F: Fn(&ScopeData) -> &T> {
	data: RwLockReadGuard<'a, ScopeData>,
	read: F,
}

impl<'a, T, F: Fn(&ScopeData) -> &T> Deref for ScopeRef<'a, T, F> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		let read = &self.read;
		read(&*self.data)
	}
}

pub struct ScopeRefMut<'a, T, F: Fn(&mut ScopeData) -> &mut T> {
	data: RwLockWriteGuard<'a, ScopeData>,
	read: F,
}

impl<'a, T, F: Fn(&mut ScopeData) -> &mut T> Deref for ScopeRefMut<'a, T, F> {
	type Target = T;

	#[allow(mutable_transmutes)]
	fn deref(&self) -> &Self::Target {
		let read = &self.read;
		let data = &*self.data;
		read(unsafe { std::mem::transmute(data) })
	}
}

impl<'a, T, F: Fn(&mut ScopeData) -> &mut T> DerefMut for ScopeRefMut<'a, T, F> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		let read = &self.read;
		read(&mut *self.data)
	}
}
