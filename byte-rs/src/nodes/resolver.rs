use std::{
	collections::{HashMap, HashSet, VecDeque},
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
		self.queue.wait();
	}

	fn process_queue(mut queue: Arc<NodeQueue>) {
		while let Some(mut node) = queue.take_next() {
			match node.val().eval() {
				NodeEval::Complete => {
					node.set_done();
					queue.complete(node);
				}
				NodeEval::NewValue(value) => {
					node.set(value);
					queue.add_again_with_dependencies(node, Vec::new());
				}
				NodeEval::NewValueAndPos(value, span) => {
					node.set(value);
					node.set_span(Some(span));
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

	/// Nodes that have been fully processed.
	done: HashSet<u64>,
}

impl NodeQueueInner {
	/// Add a node to the processing queue.
	pub fn add(&mut self, node: Node) {
		let node_id = node.id();

		// fully resolved nodes should never appear here
		assert!(!self.done.contains(&node_id));
		assert!(!node.is_done());

		// check if the node is not being processed already
		if !self.added.contains(&node_id) {
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

		// rebuild the pending map
		let has_deps = deps.len() > 0;
		let pending = self.pending_nodes.entry(node_id).or_default();
		pending.clear();
		pending.extend(deps.iter().map(|x| x.id()));
		for dep in deps.into_iter() {
			// map the reverse dependency link
			self.dependent_nodes
				.entry(dep.id())
				.or_default()
				.insert(node_id);

			// make sure the dependency is processed
			self.add(dep);
		}

		if has_deps {
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
		self.done.insert(completed_id);

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

	/// Returns true when all added nodes have been processed.
	pub fn is_done(&self) -> bool {
		let done = self.ready.len() == 0 && self.processing.len() == 0;
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
	use super::*;

	#[test]
	fn test_queue_simple() {
		let a = Node::new(TestNode { name: "Node A" });
		let b = Node::new(TestNode { name: "Node B" });
		let c = Node::new(TestNode { name: "Node C" });
		let mut resolver = NodeResolver::new();
		resolver.resolve(a.clone());
		resolver.resolve(b.clone());
		resolver.resolve(c.clone());
		resolver.wait();
		assert!(a.is_done());
		assert!(b.is_done());
		assert!(c.is_done());
	}

	#[derive(Debug)]
	struct TestNode {
		name: &'static str,
	}

	impl IsNode for TestNode {
		fn eval(&self) -> NodeEval {
			if false {
				println!("processing {}", self.name);
			}
			NodeEval::Complete
		}
	}
}
