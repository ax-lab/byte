use crate::core::input::*;

use super::*;

mod literal;

use std::{any::TypeId, sync::Arc};

pub use literal::*;

/// High-level abstraction for a strongly-typed program expression that can be
/// compiled or executed.
///
/// This is closer to source code text than low-level VM instructions, which
/// makes it easier to generate and work with.
///
/// At the same time, expressions are low-level enough to be easily translated
/// to machine instructions, operating closer to the final language output.
///
/// Expression can also offer debugging support, being able to be set up with
/// positions.
#[derive(Clone, Debug)]
pub struct Expr {
	kind: TypeId,
	node: Arc<dyn IsExpr>,
	span: Option<Span>,
}

impl Expr {
	pub fn new<T: IsExpr>(node: T) -> Expr {
		Expr {
			kind: TypeId::of::<T>(),
			node: Arc::new(node),
			span: None,
		}
	}

	pub fn val(&self) -> &dyn IsExpr {
		self.node.as_ref()
	}

	pub fn get<T: IsExpr>(&self) -> Option<&T> {
		if self.kind == TypeId::of::<T>() {
			let node = self.node.as_ref();
			let node = unsafe { &*(node as *const dyn IsExpr as *const T) };
			Some(node)
		} else {
			None
		}
	}

	pub fn at(mut self, span: Span) -> Expr {
		self.span = Some(span);
		self
	}

	pub fn span(&self) -> Option<Span> {
		self.span.clone()
	}
}

pub trait IsExpr: std::fmt::Debug + 'static {
	fn eval(&self, rt: &mut Runtime) -> Value;

	fn get_type(&self) -> Type;
}
