use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::atomic::{self, AtomicU64};
use std::sync::{Arc, Mutex};

use crate::core::error::*;
use crate::core::input::*;
use crate::core::*;
use crate::vm::expr::Expr;

use super::*;

pub enum NodeEval {
	Complete,
	NewValue(Arc<dyn IsNode>),
	NewValueAndPos(Arc<dyn IsNode>, Span),
	DependsOn(Vec<Node>),
}

pub trait IsNode: IsValue {
	fn eval(&self) -> NodeEval;
}

/// Represents parsed content from a source code file.
///
/// A node must be resolved into an [`Expr`] which can then be executed or
/// compiled.
///
/// In the process of being fully resolved into an [`Expr`], a node may
/// transform into intermediate [`Node`] values. A node may also depend on
/// other nodes, in which case it will only be fully resolved after those
/// nodes are resolved.
#[derive(Clone)]
pub struct Node {
	id: u64,
	value: Arc<Mutex<Value>>,
}

impl<T: IsNode> From<T> for Node {
	fn from(value: T) -> Self {
		Node::new(value)
	}
}

struct Value {
	done: bool,
	span: Option<Span>,
	node: Arc<dyn IsNode>,
}

impl Node {
	pub fn new<T: IsNode>(node: T) -> Self {
		static ID: AtomicU64 = AtomicU64::new(0);
		let id = ID.fetch_add(1, atomic::Ordering::SeqCst);
		let node: Arc<dyn IsNode> = Arc::new(node);
		let value = Value {
			node,
			done: false,
			span: None,
		};
		let value = Arc::new(Mutex::new(value));
		Node { id, value }
	}

	pub fn id(&self) -> u64 {
		self.id
	}

	pub fn span(&self) -> Option<Span> {
		let value = self.value.lock().unwrap();
		value.span.clone()
	}

	pub fn val(&self) -> Arc<dyn IsNode> {
		let value = self.value.lock().unwrap();
		value.node.clone()
	}

	pub fn get<T: IsNode>(&self) -> Option<Arc<T>> {
		let value = self.value.lock().unwrap();
		let node = value.node.clone();
		if (&*node).type_id() == TypeId::of::<T>() {
			let node_ptr = Arc::into_raw(node).cast::<T>();
			let node = unsafe { Arc::from_raw(node_ptr) };
			Some(node)
		} else {
			None
		}
	}

	pub fn set(&self, node: Arc<dyn IsNode>) {
		let mut value = self.value.lock().unwrap();
		if value.done {
			panic!("cannot set value for a resolved node");
		}

		let new_value = Value {
			node,
			done: value.done,
			span: value.span.clone(),
		};
		*value = new_value;
	}

	pub fn at(self, span: Span) -> Self {
		self.set_span(Some(span));
		self
	}

	pub fn is_done(&self) -> bool {
		let value = self.value.lock().unwrap();
		value.done
	}

	pub fn set_done(&self) {
		let mut value = self.value.lock().unwrap();
		value.done = true;
	}

	pub fn set_span(&self, span: Option<Span>) {
		let mut value = self.value.lock().unwrap();
		value.span = span;
	}

	pub fn get_span(a: &Node, b: &Node) -> Option<Span> {
		let sta = a.span();
		let end = b.span();
		let (sta, end) = if sta.is_none() {
			(end, sta)
		} else {
			(sta, end)
		};
		if let Some(sta) = sta {
			let span = if let Some(end) = end {
				let (sta, end) = if sta.sta.offset() < end.sta.offset() {
					(sta, end)
				} else {
					(end, sta)
				};
				Span {
					sta: sta.sta,
					end: end.end,
				}
			} else {
				Span {
					sta: sta.sta.clone(),
					end: sta.sta,
				}
			};
			Some(span)
		} else {
			None
		}
	}
}

impl Debug for Node {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}

pub enum NodeResolve {
	None,
	Done,
	Waiting,
	Invalid,
}
