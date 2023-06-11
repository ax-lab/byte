use std::{
	cmp::Ordering,
	ops::{Range, RangeBounds},
};

use super::*;

/// Provides all context for [`Node`] evaluation, which is at the core of the
/// language parsing and evaluation.
///
/// The context provides methods to evaluate and resolve a list of [`Node`]
/// until they are complete, from which point they can be used to generate
/// executable code.
///
/// It also provides any compiler services that a node might need to complete
/// its resolution (e.g. file loading, module importing, etc.).
///
/// Contexts are designed to be immutable, with context changes being applied
/// in a single transactional step and generating a new context. Additionally,
/// a context can be freely cloned and stored to preserve a given state.
///
/// Contexts can be composed on the fly, which allow for scope rules to be
/// implemented and maintained.
///
/// Nodes can store and update their own contexts internally. This is used,
/// for example, to maintain a node's own internal scope.
#[derive(Clone)]
pub struct Context<'a> {
	#[allow(unused)]
	compiler: &'a Compiler,
	data: Arc<ContextData>,
	parent: Option<Box<Context<'a>>>,
}

impl<'a> Context<'a> {
	pub fn new_root(compiler: &'a Compiler, scanner: Scanner) -> Self {
		let data = ContextData::Root { scanner };
		let data = data.into();
		Self {
			compiler,
			data,
			parent: None,
		}
	}

	pub fn resolve_all(&self, nodes: NodeList) -> Result<(Context<'a>, NodeList)> {
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

	pub fn scanner(&self) -> &Scanner {
		match self.data.as_ref() {
			ContextData::Empty => self.parent().scanner(),
			ContextData::Root { scanner } => &scanner,
			ContextData::Partial { scanner } => {
				if let Some(scanner) = scanner {
					scanner
				} else {
					self.parent().scanner()
				}
			}
		}
	}

	fn parent(&self) -> &Context {
		self.parent.as_ref().expect("using unbound context")
	}

	fn resolve_next(&self, node_list: NodeList) -> Result<(Context<'a>, NodeList, bool)> {
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
}

#[derive(Clone, Default)]
enum ContextData {
	#[default]
	Empty,
	Root {
		scanner: Scanner,
	},
	#[allow(unused)]
	Partial {
		scanner: Option<Scanner>,
	},
}

/// Wraps a [`Context`] for a [`IsNode::evaluate`] operation.
///
/// The [`EvalContext`] is writable, and tracks changes made to it so they can
/// be applied to the [`Context`] to create the resulting context.
pub struct EvalContext<'a> {
	context: &'a Context<'a>,
	errors: Errors,
	nodes: &'a NodeList,
	index: usize,
	changes: Vec<EvalChange>,
}

impl<'a> EvalContext<'a> {
	pub fn context(&self) -> &Context {
		self.context
	}

	pub fn scanner(&self) -> &Scanner {
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

	pub fn resolve_bind(&self, name: Name) -> bool {
		let _ = name;
		todo!()
	}

	pub fn get_bind(&self, name: Name) -> Option<Node> {
		let _ = name;
		todo!()
	}

	pub fn require(&self, name: Name, path: &str) {
		let _ = (name, path);
		todo!()
	}

	pub fn declare(&self, name: Name, node: Node) {
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
