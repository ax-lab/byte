use std::{
	cmp::Ordering,
	ops::{Range, RangeBounds},
};

use super::*;

/// Provides all data context for [`Node`] evaluation, which is at the core
/// of the language parsing and evaluation.
///
/// Contexts essentially store data and make that data accessible to nodes
/// during evaluation. They also support the implementation of scoping in the
/// language.
///
/// Contexts are hierarchical, being able to inherit and access data from
/// parent contexts. The root context will usually relate to a module or
/// source file.
///
/// Cloning a context creates a shallow clone with the same data. Changes to
/// a context can only be made by generating a new [`Context`] value.
#[derive(Clone)]
pub struct Context {
	compiler: CompilerRef,
	data: Arc<ContextData>,
}

#[derive(Clone, Default)]
struct ContextData {
	parent: Option<Weak<ContextData>>,
	scanner: Option<Scanner>,
}

impl Context {
	/// Create a new unbound root context.
	pub fn new_root(compiler: &Compiler) -> Self {
		let compiler = compiler.get_ref();
		let data = Default::default();
		Self { compiler, data }
	}

	/// Return the parent compiler for this context.
	pub fn compiler(&self) -> Compiler {
		self.compiler.get()
	}

	/// Return the active scanner for this context.
	pub fn scanner(&self) -> ContextRef<Scanner> {
		self.resolve(|x| x.scanner.as_ref())
			.unwrap_or_else(|| self.compiler().scanner())
	}

	pub fn resolve_all(&self, nodes: NodeList) -> Result<(Context, NodeList)> {
		let mut context = self.clone();
		let mut nodes = nodes;
		loop {
			let done;
			(context, nodes, done) = context.resolve_next(nodes)?;
			if done {
				return Ok((context, nodes));
			}
		}
	}

	fn resolve_next(&self, node_list: NodeList) -> Result<(Context, NodeList, bool)> {
		// filter nodes that are ready to be evaluated
		let mut nodes = node_list
			.iter()
			.enumerate()
			.filter_map(|(pos, node)| {
				if let Some(prec) = node.value().precedence(self) {
					Some((prec, pos, node))
				} else {
					None
				}
			})
			.collect::<Vec<_>>();

		// sort nodes by evaluation precedence
		nodes.sort_by(|((prec1, seq1), pos1, ..), ((prec2, seq2), pos2, ..)| {
			let order = prec1.cmp(prec2);
			if order == Ordering::Equal {
				assert!(seq1 == seq2);
				let (a, b) = match seq1 {
					Sequence::Ordered => (*pos1, *pos2),
					Sequence::Reverse => (*pos2, *pos1),
					Sequence::AtOnce => (0, 0),
				};
				a.cmp(&b)
			} else {
				order
			}
		});

		let (first_prec, first_seq) = if let Some(((prec, seq), ..)) = nodes.first() {
			(prec, seq)
		} else {
			return Ok((self.clone(), NodeList::empty(), true));
		};

		let nodes = nodes
			.iter()
			.enumerate()
			.take_while(|(n, item)| {
				let ((prec, ..), ..) = item;
				*n == 0 || prec == first_prec && first_seq == &Sequence::AtOnce
			})
			.map(|(index, (.., node))| (index, node));

		let mut pending = false;
		let mut changes = Vec::new();
		for (index, node) in nodes {
			let mut ctx = EvalContext {
				context: self,
				errors: Errors::new(),
				nodes: &node_list,
				index,
				changes: Default::default(),
			};
			let result = node.value().evaluate(&mut ctx)?;
			if result != NodeEval::Complete {
				// TODO: what to do with complete nodes?
				pending = true;
			}

			if ctx.changes.len() > 0 {
				changes.push((ctx.index, ctx.changes));
			}
		}

		let pending = pending || changes.len() > 0;
		let (new_context, node_list) = if changes.len() > 0 {
			let node_list = EvalChange::replace_nodes(&node_list, changes)?;
			(self.clone(), node_list)
		} else {
			(self.clone(), node_list)
		};

		Ok((new_context, node_list, !pending))
	}

	fn resolve<T, F: Fn(&ContextData) -> Option<&T>>(&self, predicate: F) -> Option<ContextRef<T>> {
		let mut data = self.data.clone();
		loop {
			if let Some(value) = predicate(&data) {
				let value = value as *const T;
				return Some(unsafe { ContextRef::new_from_context(data, value) });
			} else if let Some(parent) = &data.parent {
				let parent = parent.upgrade().expect("orphaned child context");
				data = parent;
			} else {
				return None;
			}
		}
	}
}

//====================================================================================================================//
// ContextRef
//====================================================================================================================//

/// Keeps a reference to a value from the context.
///
/// This MUST not be stored, as it holds a strong reference to either the
/// parent [`Context`] or the [`Compiler`].
pub struct ContextRef<T> {
	// parent is only owned to keep it alive while the ref is in use
	parent: ContextParent,
	data: *const T,
}

enum ContextParent {
	Context(Arc<ContextData>),
	Compiler(Compiler),
}

impl<T> ContextRef<T> {
	unsafe fn new_from_context(context: Arc<ContextData>, data: *const T) -> Self {
		let parent = ContextParent::Context(context);
		Self { parent, data }
	}

	pub(crate) fn new_from_compiler<F: FnOnce(&Compiler) -> &T>(compiler: Compiler, predicate: F) -> Self {
		let data = predicate(&compiler) as *const T;
		let parent = ContextParent::Compiler(compiler);
		Self { parent, data }
	}
}

impl<T> std::ops::Deref for ContextRef<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		let _ = self.parent;
		unsafe { &*self.data }
	}
}

//====================================================================================================================//
// EvalContext
//====================================================================================================================//

/// Wraps a [`Context`] for a [`IsNode::evaluate`] operation.
///
/// The [`EvalContext`] is writable, and tracks changes made to it so they can
/// be applied to the [`Context`] to create the resulting context.
pub struct EvalContext<'a> {
	context: &'a Context,
	errors: Errors,
	nodes: &'a NodeList,
	index: usize,
	changes: Vec<EvalChange>,
}

impl<'a> EvalContext<'a> {
	pub fn context(&self) -> &Context {
		self.context
	}

	pub fn scanner(&self) -> ContextRef<Scanner> {
		self.context.scanner()
	}

	pub fn append_errors(&mut self, errors: &Errors) {
		self.errors.append(errors)
	}

	pub fn current(&self) -> &Node {
		&self.nodes[self.index]
	}

	pub fn current_index(&self) -> usize {
		self.index
	}

	pub fn nodes(&self) -> &NodeList {
		&self.nodes
	}

	pub fn replace_current<T: IntoIterator<Item = Node>>(&mut self, nodes: T) {
		let cur = self.current_index();
		self.replace_nodes(cur..=cur, nodes);
	}

	pub fn replace_nodes<T: RangeBounds<usize>, U: IntoIterator<Item = Node>>(&mut self, range: T, nodes: U) {
		let range = compute_range(range, self.nodes.len());
		assert!(range.start <= range.end && range.end <= self.nodes.len());

		let nodes = nodes.into_iter().collect();
		self.changes.push(EvalChange::Replace { range, nodes });
	}

	pub fn resolve_bind(&self, name: &str) -> bool {
		let _ = name;
		todo!()
	}

	pub fn get_bind(&self, name: &str) -> Option<Node> {
		let _ = name;
		todo!()
	}

	pub fn require(&self, name: &str, path: &str) {
		let _ = (name, path);
		todo!()
	}

	pub fn declare(&self, name: &str, node: Node) {
		let _ = (name, node);
		todo!()
	}

	pub fn queue_resolve(&self, context: Context, nodes: NodeList) -> Option<(NodeList, Context)> {
		let _ = (context, nodes);
		todo!()
	}
}

//====================================================================================================================//
// Context changes
//====================================================================================================================//

enum EvalChange {
	Replace {
		range: Range<usize>, // range to replace
		nodes: Vec<Node>,    // nodes to replace with
	},
}

#[allow(irrefutable_let_patterns)]
impl EvalChange {
	pub fn replace_nodes(source: &NodeList, changes: Vec<(usize, Vec<EvalChange>)>) -> Result<NodeList> {
		let mut changes = changes
			.iter()
			.flat_map(|(index, changes)| {
				changes.iter().filter_map(|change| {
					if let EvalChange::Replace { range, nodes } = change {
						Some((*index, range, nodes))
					} else {
						None
					}
				})
			})
			.collect::<Vec<_>>();
		changes.sort_by_key(|(_, range, _)| range.start);

		let mut result = Vec::new();
		let mut overlaps = Vec::new();
		let mut cursor = 0;
		let mut previous = None;
		for (index, range, nodes) in changes.into_iter() {
			if range.start > cursor {
				result.extend_from_slice(source.slice(cursor..range.start));
				cursor = range.start;
			}
			if range.start < cursor {
				if let Some(previous) = previous.take() {
					overlaps.push(previous);
				}
				overlaps.push((index, range));
			} else {
				previous = Some((index, range));
			}

			result.extend_from_slice(nodes);
			cursor = std::cmp::max(cursor, range.end);
		}

		if overlaps.len() > 0 {
			let mut errors = Errors::new();
			for (index, range) in overlaps.into_iter() {
				let node = &source[index];
				let index = index + 1;
				let pos = range.start + 1;
				let end = range.end + 1;
				errors.add_at(
					format!("evaluating node at #{index}: replaced range #{pos}…{end} overlaps -- `{node}`"),
					node.span().cloned().or_else(|| source.span()),
				);
			}
		}

		result.extend_from_slice(source.slice(cursor..));
		Ok(NodeList::new(result))
	}
}
