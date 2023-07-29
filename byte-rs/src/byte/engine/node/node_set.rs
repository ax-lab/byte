use super::*;

/// Manages a collection of [`Node`].
pub struct NodeSet<'a, T: IsNode> {
	store: &'a NodeStore<T>,
	bindings: BindingMap<'a, T>,
}

impl<'a, T: IsNode> NodeSet<'a, T> {
	pub fn new(store: &'a NodeStore<T>) -> Self {
		let bindings = BindingMap::new();
		Self { store, bindings }
	}

	pub fn store(&self) -> &'a NodeStore<T> {
		self.store
	}

	pub fn new_node(&mut self, expr: T::Expr<'a>) -> Node<'a, T> {
		let data = self.store.nodes.push(NodeData::new(expr));
		let node = Node { data };
		let key = node.key();
		self.bindings.add_node(key, &node);
		node
	}

	pub fn bind(&mut self, key: T::Key, scope: Scope, data: T::Val) {
		self.bindings.bind(key, scope, data);
	}

	pub fn list_from(&self, nodes: &[Node<'a, T>]) -> NodeList<'a, T> {
		match nodes.len() {
			0 => NodeList::empty(),
			1 => NodeList::single(nodes[0]),
			2 => NodeList::pair(nodes[0], nodes[1]),
			3 => NodeList::triple(nodes[0], nodes[1], nodes[2]),
			_ => NodeList::from_list(&self.store, nodes),
		}
	}

	pub fn resolve(&mut self) {
		todo!()
	}
}

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
			let node = writer.node();
			match self {
				ExprOp::None => (),
				ExprOp::LineBreak => {
					let mut lines = Vec::new();
					let nodes = writer.nodes();
					let mut cursor = 0;
					for i in 0..writer.target_count() {
						let index = writer.target_index(i);
						if index > cursor + 1 {
							let line = nodes.slice(cursor..index);
							let line = writer.new_node(Expr::List(line));
							lines.push(line);
						}
						cursor = index + 1;
					}
					writer.replace_all(&lines);
				}
				ExprOp::ParseExpr => match node.expr() {
					Expr::List(nodes) => {
						if nodes.len() != 1 {
							panic!("invalid expression: {nodes:?}");
						}
						writer.replace_all(&[nodes.get(0).unwrap()]);
					}
					_ => unreachable!(),
				},
				ExprOp::ParseLet => {
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
					writer.remove_range(node, cursor);
				}
				ExprOp::Bind(value) => {
					let res = Expr::Data(*value);
					writer.set_value(res);
				}
				ExprOp::Add => {
					let lhs = node.prev().expect("add: lhs expected");
					let rhs = node.next().expect("add: rhs expected");
					let a = lhs.expr().value();
					let b = lhs.expr().value();
					let res = Expr::Data(a + b);
					let res = writer.new_node(res);
					writer.replace_range(lhs, rhs, res);
				}
				ExprOp::Mul => {
					let lhs = node.prev().expect("mul: lhs expected");
					let rhs = node.next().expect("mul: rhs expected");
					let a = lhs.expr().value();
					let b = lhs.expr().value();
					let res = Expr::Data(a * b);
					let res = writer.new_node(res);
					writer.replace_range(lhs, rhs, res);
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
				ExprOp::None => 0,
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
