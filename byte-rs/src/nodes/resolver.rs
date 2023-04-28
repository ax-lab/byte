use std::{
	collections::{HashMap, HashSet, VecDeque},
	panic::AssertUnwindSafe,
	sync::{Arc, Condvar, Mutex},
	thread,
};

use super::*;

const NUM_RESOLVERS: usize = 1;

pub struct NodeResolver {
	queue: Arc<NodeQueue>,
}

impl NodeResolver {
	pub fn new() -> Self {
		let result = Self {
			queue: Default::default(),
		};

		for _ in 0..NUM_RESOLVERS {
			let queue = result.queue.clone();
			thread::spawn(move || {
				Self::process_queue(queue);
			});
		}

		result
	}

	pub fn resolve(&mut self, node: Node) {
		self.queue.push(node);
	}

	pub fn wait(&self) {
		self.queue.done();
		self.queue.wait();
	}

	fn process_queue(queue: Arc<NodeQueue>) {
		while let Some(mut node) = queue.take_next() {
			let eval = {
				let value_node = node.clone();
				let value = node.val();
				let result = std::panic::catch_unwind(AssertUnwindSafe(move || {
					let result = value.eval(value_node);
					result
				}));
				match result {
					Ok(value) => value,
					Err(err) => {
						let err = err
							.downcast_ref::<&str>()
							.map(|x| x.to_string())
							.or_else(|| err.downcast_ref::<String>().cloned())
							.unwrap_or_default();
						let mut node_err = node.clone();
						let mut errors = node_err.errors_mut();
						errors.add(Error::new(format!(
							"eval panicked: {err:?}\n\n{}\n\n{node:?}\n",
							">>> Node <<<"
						)));
						NodeEval::Complete
					}
				}
			};
			match eval {
				NodeEval::Complete => {
					node.set_done();
					queue.complete(node);
				}
				NodeEval::Changed => {
					queue.add_again_with_dependencies(node, Vec::new());
				}
				NodeEval::DependsOn(deps) => {
					queue.add_again_with_dependencies(node, deps);
				}
			}
		}
	}
}

#[derive(Default)]
struct NodeQueue {
	queue: Mutex<NodeQueueInner>,
	signal: Condvar,
}

impl NodeQueue {
	/// Queue a node for processing.
	pub fn push(&self, node: Node) {
		let mut queue = self.queue.lock().unwrap();
		queue.add(node);
		self.signal.notify_all();
	}

	pub fn done(&self) {
		let mut queue = self.queue.lock().unwrap();
		queue.done();
		self.signal.notify_all();
	}

	/// Wait until all pending nodes have processing completed.
	pub fn wait(&self) {
		let mut queue = self.queue.lock().unwrap();
		loop {
			if queue.is_done() {
				return;
			}
			queue = self.signal.wait(queue).unwrap();
		}
	}

	/// Wait for the next node to be available for processing and retrieve it
	/// from the queue.
	///
	/// Either returns the next node or [`None`] when the queue is complete.
	pub fn take_next(&self) -> Option<Node> {
		let mut queue = self.queue.lock().unwrap();
		loop {
			if let Some(next) = queue.take_next() {
				return Some(next);
			} else {
				if queue.is_done() {
					// queue is complete, return
					return None;
				} else {
					// release the lock and wait for next signal
					queue = self.signal.wait(queue).unwrap();
				}
			}
		}
	}

	/// Flag a completed node.
	pub fn complete(&self, node: Node) {
		let mut queue = self.queue.lock().unwrap();
		queue.complete(&node);
		self.signal.notify_all();
	}

	/// Add a node for further processing with its pending dependencies.
	pub fn add_again_with_dependencies(&self, node: Node, deps: Vec<Node>) {
		let mut queue = self.queue.lock().unwrap();
		queue.add_again_with_dependencies(node, deps);
		self.signal.notify_all();
	}
}

#[derive(Default)]
struct NodeQueueInner {
	done: bool,

	/// Queue of nodes next in line for processing.
	ready: VecDeque<Node>,

	/// List of node ids being processed.
	processing: HashSet<u64>,

	/// Nodes waiting for dependencies to be processed by their ids.
	waiting: HashMap<u64, Node>,

	/// Map of pending node ids with their dependencies.
	pending_nodes: HashMap<u64, HashSet<u64>>,

	/// Map of dependent nodes for a given dependency id.
	dependent_nodes: HashMap<u64, HashSet<u64>>,

	/// Nodes that are currently being handled by the queue.
	added: HashSet<u64>,
}

impl NodeQueueInner {
	/// Add a node to the processing queue.
	pub fn add(&mut self, node: Node) {
		let node_id = node.id();

		// check if the node is not being processed already
		if !self.added.contains(&node_id) {
			self.added.insert(node_id);
			self.ready.push_back(node);
		}
	}

	/// Take a node from the processing queue for processing, if any is
	/// available.
	pub fn take_next(&mut self) -> Option<Node> {
		if let Some(node) = self.ready.pop_front() {
			// flag the node as processing
			self.processing.insert(node.id());
			Some(node)
		} else {
			None
		}
	}

	pub fn add_again_with_dependencies(&mut self, node: Node, deps: Vec<Node>) {
		let node_id = node.id();

		// sanity check
		assert!(self.added.contains(&node_id));

		// clear the processing flag
		let removed = self.processing.remove(&node_id);
		assert!(removed);

		// Filter any dependency that has been resolved already.
		let deps = deps
			.into_iter()
			.filter(|x| !x.is_done())
			.collect::<Vec<_>>();

		// rebuild the pending map
		let has_deps = deps.len() > 0;
		if has_deps {
			let pending = self.pending_nodes.entry(node_id).or_default();
			pending.clear();
			pending.extend(deps.iter().map(|x| x.id()));
		};
		for dep in deps.into_iter() {
			// map the reverse dependency link
			self.dependent_nodes
				.entry(dep.id())
				.or_default()
				.insert(node_id);

			// make sure the dependency is processed
			self.add(dep);
		}

		if !has_deps {
			self.ready.push_back(node);
		} else {
			self.waiting.insert(node_id, node);
		}
	}

	/// Flag a completed node.
	pub fn complete(&mut self, node: &Node) {
		let completed_id = node.id();

		let removed = self.processing.remove(&completed_id);
		assert!(removed);

		self.added.remove(&completed_id);

		// check all dependent nodes and remove the completed node as a pending
		// dependency
		if let Some(ids) = self.dependent_nodes.remove(&completed_id) {
			for id in ids.into_iter() {
				let deps = self.pending_nodes.get_mut(&id).unwrap();
				deps.remove(&completed_id);
				if deps.len() == 0 {
					// remove the empty dependency map
					self.pending_nodes.remove(&id);

					// when all dependencies of a node have been processed, move
					// it back to the ready queue
					let node = self.waiting.remove(&id).unwrap();
					self.ready.push_back(node);
				}
			}
		}
	}

	pub fn done(&mut self) {
		self.done = true;
	}

	/// Returns true when all added nodes have been processed.
	pub fn is_done(&self) -> bool {
		let done = self.done && self.ready.len() == 0 && self.processing.len() == 0;
		if done {
			assert!(self.waiting.len() == 0);
			assert!(self.added.len() == 0);
			assert!(self.dependent_nodes.len() == 0);
			assert!(self.pending_nodes.len() == 0);
		}
		done
	}
}

#[cfg(test)]
mod tests {
	use crate::core::repr::*;

	use super::*;

	#[test]
	fn test_queue_simple() {
		let out = Arc::new(Mutex::new(Vec::new()));
		let a = Node::new(SimpleNode {
			name: "A".into(),
			out: out.clone(),
		});
		let b = Node::new(SimpleNode {
			name: "B".into(),
			out: out.clone(),
		});
		let c = Node::new(SimpleNode {
			name: "C".into(),
			out: out.clone(),
		});

		let mut resolver = NodeResolver::new();
		resolver.resolve(a.clone());
		resolver.resolve(b.clone());
		resolver.resolve(c.clone());
		resolver.wait();

		assert!(a.is_done());
		assert!(b.is_done());
		assert!(c.is_done());

		let mut out = out.lock().unwrap().clone();
		out.sort();
		assert_eq!(out, ["A done", "B done", "C done"]);
	}

	#[test]
	fn test_queue_complex() {
		let out = Arc::new(Mutex::new(Vec::new()));
		let c1 = Node::new(ComplexNode::new("C1", out.clone()));
		let c2 = Node::new(ComplexNode::new("C2", out.clone()));

		let mut resolver = NodeResolver::new();
		resolver.resolve(c1.clone());
		resolver.resolve(c2.clone());
		resolver.wait();

		assert!(c1.is_done());
		assert!(c2.is_done());

		let s1 = c1.get::<SimpleNode>().unwrap();
		let s2 = c2.get::<SimpleNode>().unwrap();
		assert_eq!(s1.name, "C1: 2 - Final");
		assert_eq!(s2.name, "C2: 2 - Final");

		let mut out = out.lock().unwrap().clone();
		out.sort();
		assert_eq!(
			out,
			[
				"C1: 0",
				"C1: 0 - A done",
				"C1: 0 - B done",
				"C1: 1",
				"C1: 1 - C done",
				"C1: 2",
				"C1: 2 - Final done",
				"C2: 0",
				"C2: 0 - A done",
				"C2: 0 - B done",
				"C2: 1",
				"C2: 1 - C done",
				"C2: 2",
				"C2: 2 - Final done",
			]
		);
	}

	#[derive(Debug, Clone)]
	struct SimpleNode {
		name: String,
		out: Arc<Mutex<Vec<String>>>,
	}

	has_traits!(SimpleNode: IsNode);
	repr_from_fmt!(SimpleNode);

	impl IsNode for SimpleNode {
		fn eval(&self, _node: Node) -> NodeEval {
			let mut out = self.out.lock().unwrap();
			out.push(format!("{} done", self.name));
			NodeEval::Complete
		}

		fn span(&self) -> Option<Span> {
			None
		}
	}

	impl std::fmt::Display for SimpleNode {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "{self:?}")
		}
	}

	#[derive(Debug)]
	struct ComplexNode {
		name: String,
		next: Mutex<usize>,
		out: Arc<Mutex<Vec<String>>>,
	}

	impl Clone for ComplexNode {
		fn clone(&self) -> Self {
			unreachable!()
		}
	}

	impl ComplexNode {
		pub fn new(name: &'static str, out: Arc<Mutex<Vec<String>>>) -> Self {
			Self {
				name: name.into(),
				out,
				next: Mutex::new(0),
			}
		}

		fn say(&self, msg: &str) {
			let mut out = self.out.lock().unwrap();
			out.push(format!("{}: {}", self.name, msg));
		}
	}

	has_traits!(ComplexNode: IsNode);
	repr_from_fmt!(ComplexNode);

	impl IsNode for ComplexNode {
		fn eval(&self, mut node: Node) -> NodeEval {
			let mut next = self.next.lock().unwrap();
			let state = *next;
			*next += 1;
			match state {
				0 => {
					self.say("0");
					let out = self.out.clone();
					let a = SimpleNode {
						name: format!("{}: 0 - A", self.name),
						out,
					};
					let out = self.out.clone();
					let b = SimpleNode {
						name: format!("{}: 0 - B", self.name),
						out,
					};
					let a = Node::new(a);
					let b = Node::new(b);
					NodeEval::DependsOn(vec![a, b])
				}
				1 => {
					self.say("1");
					let out = self.out.clone();
					let c = SimpleNode {
						name: format!("{}: 1 - C", self.name),
						out,
					};
					let c = Node::new(c);
					NodeEval::DependsOn(vec![c])
				}
				2 => {
					self.say("2");
					let out = self.out.clone();
					let d = SimpleNode {
						name: format!("{}: 2 - Final", self.name),
						out,
					};
					node.set(Value::from(d));
					NodeEval::Changed
				}

				_ => panic!("invalid state"),
			}
		}

		fn span(&self) -> Option<Span> {
			None
		}
	}

	impl std::fmt::Display for ComplexNode {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "{self:?}")
		}
	}
}
