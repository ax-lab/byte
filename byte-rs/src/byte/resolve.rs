use super::*;

pub trait NodeOperator: Cell {
	fn evaluate(&self, context: &mut ResolveContext);
}

impl Context {
	pub fn resolve(&self, nodes: &NodeValueList, errors: &mut Errors) -> (Context, NodeValueList) {
		let compiler = &self.compiler();

		let mut nodes = nodes.clone();
		let context = self.clone();

		let mut has_changes = true;
		while errors.empty() && has_changes {
			let mut ops_pending = context.get_operators();
			let mut ops_current = Vec::new();

			has_changes = false;
			while ops_pending.len() > 0 && errors.empty() {
				// figure out the next set of operators to apply
				let next_prec = ops_pending[0].0;
				let mut count = 1;
				while count < ops_pending.len() && ops_pending[count].0 == next_prec {
					count += 1;
				}

				ops_current.clear();
				ops_current.extend(ops_pending.drain(0..count));

				// apply all operators "simultaneously" to the nodes
				let mut changes = Vec::new();
				for (_, op) in ops_current.iter() {
					let mut context = ResolveContext::new(compiler, &context, &nodes);
					op.evaluate(&mut context);
					if context.errors.len() > 0 {
						errors.append(&context.errors);
					}
					changes.extend(context.changes.into_iter());
				}

				if changes.len() > 0 {
					nodes = match NodeReplace::apply(&nodes, changes) {
						Ok(nodes) => nodes,
						Err(errs) => {
							// replace errors are fatal
							errors.append(&errs);
							return (context, nodes);
						}
					};
					has_changes = true;
					break;
				}
			}
		}

		(context, nodes)
	}
}

//====================================================================================================================//
// Context
//====================================================================================================================//

pub struct ResolveContext<'a> {
	compiler: &'a Compiler,
	context: &'a Context,
	nodes: &'a NodeValueList,
	errors: Errors,
	changes: Vec<NodeReplace>,
}

impl<'a> ResolveContext<'a> {
	fn new(compiler: &'a Compiler, context: &'a Context, nodes: &'a NodeValueList) -> Self {
		Self {
			compiler,
			context,
			nodes,
			errors: Default::default(),
			changes: Default::default(),
		}
	}

	pub fn compiler(&self) -> &Compiler {
		self.compiler
	}

	pub fn nodes(&self) -> &NodeValueList {
		self.nodes
	}

	pub fn context(&self) -> &Context {
		self.context
	}

	pub fn errors_mut(&mut self) -> &mut Errors {
		&mut self.errors
	}

	pub fn replace_nodes<I: IntoIterator<Item = NodeValue>>(&mut self, nodes: I) {
		self.replace_range(.., nodes)
	}

	pub fn replace_index<I: IntoIterator<Item = NodeValue>>(&mut self, index: usize, nodes: I) {
		self.replace_range(index..index + 1, nodes)
	}

	pub fn replace_range<T: RangeBounds<usize>, I: IntoIterator<Item = NodeValue>>(&mut self, range: T, nodes: I) {
		let range = compute_range(range, self.nodes.len());
		assert!(range.end <= self.nodes.len() && range.start <= range.end);
		self.push_change(NodeReplace {
			index: range.start,
			count: range.end - range.start,
			nodes: nodes.into_iter().collect(),
		});
	}

	fn push_change(&mut self, new_change: NodeReplace) {
		self.changes.push(new_change);
	}
}

//====================================================================================================================//
// Helpers
//====================================================================================================================//

struct NodeReplace {
	index: usize,
	count: usize,
	nodes: Vec<NodeValue>,
}

impl NodeReplace {
	#[allow(unused)]
	pub fn overlaps(&self, node: &NodeReplace) -> bool {
		let (a1, a2) = (self.index, self.index + self.count);
		let (b1, b2) = (node.index, node.index + node.count);
		(a1 < b2 && b1 < a2) || a1 == b1
	}

	pub fn apply(nodes: &NodeValueList, list: Vec<NodeReplace>) -> Result<NodeValueList> {
		let mut list = list;
		list.sort_by_key(|it| (it.index, if it.count == 0 { 0 } else { 1 }, std::cmp::Reverse(it.count)));

		let mut errors = Errors::new();
		let node_list = nodes.as_slice();
		let mut output = Vec::new();
		let mut cursor = 0;
		let mut inserted = false;
		for NodeReplace { index, count, nodes } in list.into_iter() {
			let end = index + count;
			assert!(end <= node_list.len());

			// TODO: improve error handling
			if index < cursor {
				errors.add(format!("operators replace overlapping ranges #{index}…{cursor}"));
				cursor = std::cmp::max(cursor, end);
				continue;
			} else if index == cursor && count == 0 && nodes.len() > 0 && inserted {
				errors.add(format!("multiple node insertions at same position #{index}"));
				inserted = false;
				continue;
			}

			if index > cursor {
				output.extend(node_list[cursor..index].iter().cloned());
			}

			if nodes.len() > 0 {
				output.extend(nodes);
				inserted = count == 0;
			}

			cursor = end;
		}

		if cursor < node_list.len() {
			output.extend(node_list[cursor..].iter().cloned());
		}

		if errors.empty() {
			Ok(NodeValueList::new(output))
		} else {
			Err(errors)
		}
	}
}
