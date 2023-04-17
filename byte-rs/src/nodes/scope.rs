use crate::core::str::*;

/// Maintain and resolve symbol definitions during parsing.
///
/// Scopes form an hierarchy that can be constructed using the `child`
/// and `inherit` methods.
///
/// Cloning a scope just copies the reference. The clone will still refer
/// to the same scope.
#[derive(Clone)]
pub struct Scope {}

impl Scope {
	pub fn new() -> Scope {
		Scope {}
	}

	/// Return the root scope for the current hierarchy.
	pub fn root(&self) -> Scope {
		todo!()
	}

	/// Returns a reference to the parent scope for this scope, unless this is
	/// the root.
	///
	/// Note that calling parent on a scope will prevent [`make_own`] for the
	/// entire hierarchy up to the parent.
	pub fn parent(&self) -> Option<Scope> {
		todo!()
	}

	/// Transform an inherited scope into its own.
	///
	/// By default, inherited scopes just apply all changes to their parents
	/// but making a scope into its own will isolate all changes to itself.
	///
	/// **IMPORTANT:** This must be called as soon as possible, before any
	/// changes are made to the scope or [`parent`] is accessed.
	pub fn make_own(&mut self) {
		todo!()
	}

	/// Inherit a scope down a level from the current one.
	///
	/// Inherited scopes will see everything in the parent. By default, any
	/// changes to an inherit scope will still apply to the parent unless the
	/// scope is made into its own with [`make_own`].
	pub fn inherit(&self) -> Scope {
		todo!()
	}

	/// Creates a child scope. This is exactly equivalent to `inherit`
	/// followed up by `make_own`.
	pub fn child(&self) -> Scope {
		let mut child = self.inherit();
		child.make_own();
		child
	}

	pub fn get(&self, name: Str) -> ScopeCell {
		todo!()
	}
}

pub struct ScopeCell {}

impl ScopeCell {
	pub fn resolve(&self) {
		todo!()
	}
}
