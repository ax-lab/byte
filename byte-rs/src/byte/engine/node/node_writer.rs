use std::ops::RangeBounds;

use super::*;

// TODO: integrate error handling into the writer interface

/// Writer for a [`NodeSet`].
pub struct NodeWriter<'a, T: IsNode> {
	store: &'a NodeStore<T>,
	nodes: NodeList<'a, T>,
	target: Vec<usize>,
	new_nodes: Vec<Node<'a, T>>,
	new_binds: Vec<(T::Key, Scope, T::Val)>,
}

impl<'a, T: IsNode> NodeWriter<'a, T> {
	pub fn nodes(&self) -> &NodeList<'a, T> {
		&self.nodes
	}

	pub fn offset_end(&self) -> usize {
		self.nodes.get(self.nodes.len() - 1).unwrap().offset()
	}

	pub fn target_count(&self) -> usize {
		self.target.len()
	}

	pub fn target_index(&self, n: usize) -> usize {
		self.target[n]
	}

	pub fn new_node(&mut self, expr: T::Expr<'a>) -> Node<'a, T> {
		let data = self.store.nodes.push(NodeData::new(expr));
		let node = Node { data };
		self.new_nodes.push(node);
		node
	}

	pub fn bind(&mut self, key: T::Key, scope: Scope, value: T::Val) {
		self.new_binds.push((key, scope, value));
	}

	pub fn set_value(&mut self, _index: usize, _expr: T::Expr<'a>) {
		todo!()
	}

	pub fn replace_range<R: RangeBounds<usize>>(&mut self, _range: R, _node: Node<'a, T>) {
		todo!()
	}

	pub fn remove_range<R: RangeBounds<usize>>(&mut self, _range: R) {
		todo!()
	}

	pub fn replace_all(&mut self, _nodes: &[Node<'a, T>]) {
		todo!()
	}

	pub fn for_each<P: Fn(&mut Self, &Node<'a, T>, usize)>(&mut self, _predicate: P) {
		todo!()
	}

	// TODO: we need to validate conflicting changes across writers

	pub fn apply(self, set: &mut NodeSet<'a, T>) {
		for (key, scope, value) in self.new_binds {
			// TODO: check binds for the key with different values in overlapping scopes
			set.bind(key, scope, value);
		}

		for it in self.new_nodes {
			// TODO: new nodes could contain references to nodes already bound in the tree
			set.add_node(it);
		}
		todo!();
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_expr() {
		let store: NodeStore<Eval> = NodeStore::new();
		let mut nodes = store.new_node_set();
		nodes.bind(ExprKey::Break, Scope::Root, ExprOp::LineBreak); // TODO: needs precedence
		nodes.bind(ExprKey::List, Scope::Root, ExprOp::ParseExpr);
		nodes.bind(ExprKey::Name("*"), Scope::Root, ExprOp::Mul);
		nodes.bind(ExprKey::Name("+"), Scope::Root, ExprOp::Add);

		let expr = vec![
			// let a = 10
			nodes.new_node(Expr::Token("let")),
			nodes.new_node(Expr::Token("a")),
			nodes.new_node(Expr::Token("=")),
			nodes.new_node(Expr::Data(10)),
			nodes.new_node(Expr::Break),
			// let b = 4
			nodes.new_node(Expr::Token("let")),
			nodes.new_node(Expr::Token("b")),
			nodes.new_node(Expr::Token("=")),
			nodes.new_node(Expr::Data(4)),
			nodes.new_node(Expr::Break),
			// let c = 2
			nodes.new_node(Expr::Token("let")),
			nodes.new_node(Expr::Token("c")),
			nodes.new_node(Expr::Token("=")),
			nodes.new_node(Expr::Data(2)),
			nodes.new_node(Expr::Break),
			// let d = a * b + c
			nodes.new_node(Expr::Token("let")),
			nodes.new_node(Expr::Token("d")),
			nodes.new_node(Expr::Token("=")),
			nodes.new_node(Expr::Token("a")),
			nodes.new_node(Expr::Token("*")),
			nodes.new_node(Expr::Token("b")),
			nodes.new_node(Expr::Token("+")),
			nodes.new_node(Expr::Token("c")),
		];

		nodes.new_node(Expr::List(nodes.list_from(&expr)));

		let result = nodes.new_node(Expr::Token("d"));
		nodes.resolve();
		assert_eq!(result.expr(), &Expr::Data(42));
	}

	#[derive(Copy, Clone)]
	struct Eval;

	#[derive(Debug, Eq, PartialEq)]
	enum Expr<'a> {
		Break,
		Token(&'static str),
		List(NodeList<'a, Eval>),
		Data(i32),
	}

	#[allow(unused)]
	impl<'a> Expr<'a> {
		pub fn value(&self) -> i32 {
			match self {
				Expr::Data(value) => *value,
				_ => panic!("expected value"),
			}
		}

		pub fn symbol(&self) -> &'static str {
			match self {
				Expr::Token(symbol) => symbol,
				_ => panic!("expected symbol"),
			}
		}
	}

	#[derive(Default, Copy, Clone, Eq, PartialEq, Debug, Hash)]
	enum ExprKey {
		#[default]
		None,
		Break,
		Name(&'static str),
		List,
		Data,
	}

	#[allow(unused)]
	#[derive(Default, Copy, Clone, Eq, PartialEq, Debug, Hash)]
	enum ExprOp {
		#[default]
		None,
		LineBreak,
		ParseExpr,
		ParseLet,
		Bind(i32),
		Add,
		Mul,
	}

	impl ExprOp {
		#[allow(unused)]
		pub fn apply(&self, writer: &mut NodeWriter<Eval>) {
			match self {
				ExprOp::None => (),
				ExprOp::LineBreak => {
					let mut lines = Vec::new();
					let mut cursor = 0;
					for i in 0..writer.target_count() {
						let index = writer.target_index(i);
						if index > cursor + 1 {
							let line = writer.nodes().slice(cursor..index);
							let line = writer.new_node(Expr::List(line));
							lines.push(line);
						}
						cursor = index + 1;
					}
					writer.replace_all(&lines);
				}
				ExprOp::ParseExpr => {
					for i in 0..writer.target_count() {
						let index = writer.target_index(i);
						let node = writer.nodes().get(index).unwrap();
						match node.expr() {
							Expr::List(nodes) => {
								if nodes.len() != 1 {
									panic!("invalid expression: {nodes:?}");
								}
								writer.replace_range(index..index + 1, nodes.get(0).unwrap());
							}
							_ => unreachable!(),
						}
					}
				}
				ExprOp::ParseLet => {
					for i in 0..writer.target_count() {
						let index = writer.target_index(i);
						let node = writer.nodes().get(index).unwrap();
						let mut cursor = node.next().expect("invalid let");
						let name = cursor.expr().symbol();
						cursor = cursor.next().expect("`=` expected");
						assert_eq!(cursor.expr().symbol(), "=");

						cursor = cursor.next().expect("value expected");
						let value = cursor.expr().value();

						if let Some(next) = cursor.next() {
							panic!("let: expected end of expression, got `{next:?}`");
						}

						let offset_end = writer.offset_end();
						writer.bind(
							ExprKey::Name(name),
							Scope::Range(node.offset(), offset_end),
							ExprOp::Bind(value),
						);
						writer.remove_range(index..index + 2); // TODO: this is probably wrong
					}
				}
				ExprOp::Bind(value) => {
					for i in 0..writer.target_count() {
						let index = writer.target_index(i);
						let res = Expr::Data(*value);
						writer.set_value(index, res);
					}
				}
				ExprOp::Add => {
					for i in 0..writer.target_count() {
						let index = writer.target_index(i);
						let node = writer.nodes().get(index).unwrap();
						let lhs = node.prev().expect("add: lhs expected");
						let rhs = node.next().expect("add: rhs expected");
						let a = lhs.expr().value();
						let b = lhs.expr().value();
						let res = Expr::Data(a + b);
						let res = writer.new_node(res);
						writer.replace_range(index - 1..index + 1, res);
					}
				}
				ExprOp::Mul => {
					for i in 0..writer.target_count() {
						let index = writer.target_index(i);
						let node = writer.nodes().get(index).unwrap();
						let lhs = node.prev().expect("mul: lhs expected");
						let rhs = node.next().expect("mul: rhs expected");
						let a = lhs.expr().value();
						let b = lhs.expr().value();
						let res = Expr::Data(a * b);
						let res = writer.new_node(res);
						writer.replace_range(index - 1..index + 1, res);
					}
				}
			}
		}
	}

	impl IsNode for Eval {
		type Expr<'a> = Expr<'a>;
		type Key = ExprKey;
		type Val = ExprOp;
		type Precedence = i32;

		fn get_precedence(val: &Self::Val) -> Self::Precedence {
			match val {
				ExprOp::None => i32::MAX,
				ExprOp::LineBreak => 1,
				ExprOp::Bind(_) => 2,
				ExprOp::Mul => 3,
				ExprOp::Add => 4,
				ExprOp::ParseLet => 5,
				ExprOp::ParseExpr => 6,
			}
		}
	}

	impl<'a> IsExpr<'a, Eval> for Expr<'a> {
		fn key(&self) -> ExprKey {
			match self {
				Expr::Break => ExprKey::Break,
				Expr::Token(name) => ExprKey::Name(name),
				Expr::List(..) => ExprKey::List,
				Expr::Data(..) => ExprKey::Data,
			}
		}

		fn children(&self) -> NodeIterator<'a, Eval> {
			match self {
				Expr::List(list) => list.iter(),
				_ => NodeIterator::empty(),
			}
		}
	}
}
