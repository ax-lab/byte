use super::*;

pub mod eval;
pub mod expr;
pub mod parsing;

pub use eval::*;
pub use expr::*;
pub use parsing::*;

const SHOW_INDENT: bool = false;

#[derive(Clone)]
pub struct Node {
	data: Arc<NodeData>,
}

struct NodeData {
	id: Id,
	span: RwLock<Span>,
	value: RwLock<Expr>,
	version: RwLock<usize>,
	scope: ScopeHandle,
	_code: RwLock<Option<Func>>, // TODO: implement this
}

impl Node {
	pub fn new(value: Expr, scope: ScopeHandle, span: Span) -> Self {
		let value = value.into();
		let version = 0.into();
		let id = id();
		let span = span.into();
		let data = NodeData {
			id,
			span,
			value,
			version,
			scope,
			_code: Default::default(),
		};
		let node = Self { data: data.into() };
		node
	}

	pub fn get_type(&self) -> Result<Type> {
		self.expr().get_type()
	}

	pub fn raw(nodes: Vec<Node>, scope: ScopeHandle) -> Self {
		let span = Span::from_node_vec(&nodes);
		Expr::Raw(nodes.into()).at(scope, span)
	}

	pub fn id(&self) -> Id {
		self.data.id.clone()
	}

	pub fn version(&self) -> usize {
		*self.data.version.read().unwrap()
	}

	pub fn expr(&self) -> Expr {
		self.data.value.read().unwrap().clone()
	}

	pub fn span(&self) -> Span {
		self.data.span.read().unwrap().clone()
	}

	pub fn offset(&self) -> usize {
		self.span().offset()
	}

	pub fn indent(&self) -> usize {
		self.span().indent()
	}

	pub fn scope(&self) -> Scope {
		self.data.scope.get()
	}

	pub fn scope_handle(&self) -> ScopeHandle {
		self.data.scope.clone()
	}

	pub fn get_dependencies<P: FnMut(&Node)>(&self, output: P) {
		self.expr().get_dependencies(output)
	}

	pub fn set_value(&mut self, new_value: Expr, new_span: Span) {
		self.write(|| {
			let mut value = self.data.value.write().unwrap();
			let mut span = self.data.span.write().unwrap();
			*value = new_value;
			*span = new_span;
		});
	}

	pub unsafe fn set_value_inner(&self, new_value: Expr, new_span: Span) {
		self.write(|| {
			let mut value = self.data.value.write().unwrap();
			let mut span = self.data.span.write().unwrap();
			*value = new_value;
			*span = new_span;
		});
	}

	fn write<T, P: FnOnce() -> T>(&self, write: P) -> T {
		let mut version = self.data.version.write().unwrap();
		*version = *version + 1;
		(write)()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Node value helpers
	//----------------------------------------------------------------------------------------------------------------//

	pub fn short_repr(&self) -> String {
		self.expr().short_repr()
	}

	/// Number of child nodes.
	pub fn len(&self) -> usize {
		self.expr().len()
	}

	/// Get a node children by its index.
	pub fn get(&self, index: usize) -> Option<Node> {
		self.expr().get(index).cloned()
	}

	/// Return a new [`Expr::Raw`] from a slice of this node's children.
	pub fn slice<T: RangeBounds<usize>>(&self, range: T) -> Node {
		let scope = self.scope_handle();
		let node = self.expr();

		// TODO: maybe have a `can_slice` property
		assert!(matches!(node, Expr::Raw(..))); // we don't want slice to be used with any node
		let list = node.children();
		let range = compute_range(range, list.len());
		let index = range.start;
		let slice = &list[range];
		let span = Span::from_nodes(slice);
		let span = span.or_with(|| list.get(index).map(|x| x.span().pos()).unwrap_or_default());
		let list = Arc::new(slice.iter().map(|x| (*x).clone()).collect());
		Expr::Raw(list).at(scope, span)
	}

	/// Iterator over this node's children.
	pub fn iter(&self) -> impl Iterator<Item = Node> {
		let node = self.expr();
		let list = node.iter().cloned().collect::<Vec<_>>();
		list.into_iter()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Parsing helpers
	//----------------------------------------------------------------------------------------------------------------//

	pub fn to_raw(self) -> Node {
		let span = self.span();
		let scope = self.scope_handle();
		Expr::Raw(vec![self].into()).at(scope, span)
	}

	pub fn is_symbol(&self, expected: &Symbol) -> bool {
		match self.expr() {
			Expr::Token(Token::Symbol(symbol) | Token::Word(symbol)) => &symbol == expected,
			_ => false,
		}
	}

	pub fn is_keyword(&self, expected: &Symbol) -> bool {
		match self.expr() {
			Expr::Token(Token::Word(symbol)) => &symbol == expected,
			_ => false,
		}
	}

	pub fn has_symbol(&self, symbol: &Symbol) -> bool {
		match self.expr() {
			Expr::Token(Token::Symbol(s) | Token::Word(s)) => &s == symbol,
			_ => false,
		}
	}

	pub fn symbol(&self) -> Option<Symbol> {
		self.expr().symbol()
	}

	pub fn execute(&self, scope: &mut RuntimeScope) -> Result<ExprValue> {
		match self.expr() {
			Expr::Never => {
				let error = format!("never expression cannot be evaluated");
				Err(Errors::from(error, self.span().clone()))
			}
			Expr::Unit => Ok(Value::from(()).into()),
			Expr::Null => Ok(Value::Null.into()),
			Expr::Let(name, offset, expr) => {
				let value = expr.execute(scope)?;
				scope.set(name.clone(), offset, value.clone().into());
				Ok(value)
			}
			Expr::Boolean(value) => Ok(Value::from(value).into()),
			Expr::Str(value) => Ok(Value::from(value.to_string()).into()),
			Expr::Int(value) => Ok(Value::from(value.clone()).into()),
			Expr::Float(value) => Ok(Value::from(value.clone()).into()),
			Expr::Variable(name, offset, ..) => match scope.get(&name, offset).cloned() {
				Some(value) => Ok(ExprValue::Variable(name.clone(), offset.clone(), value)),
				None => Err(Errors::from(format!("variable `{name}` not set"), self.span().clone())),
			},
			Expr::Print(expr, tail) => {
				let list = expr.as_sequence();
				let mut values = Vec::new();
				for expr in list.iter() {
					let value = expr.execute(scope)?.into_value();
					if !value.is_unit() {
						values.push(value)
					}
				}

				let mut output = scope.stdout();
				for (i, it) in values.into_iter().enumerate() {
					if i > 0 {
						write!(output, " ")?;
					}
					write!(output, "{it}")?;
				}
				write!(output, "{tail}")?;
				Ok(Value::from(()).into())
			}
			Expr::UnaryOp(op, op_impl, arg) => {
				if let Some(op_impl) = op_impl {
					op_impl.get().execute(scope, &arg)
				} else {
					let error = format!("unresolved unary operator `{op}`");
					let error = Errors::from(error, self.span());
					Err(error)
				}
			}
			Expr::BinaryOp(op, op_impl, lhs, rhs) => {
				if let Some(op_impl) = op_impl {
					op_impl.get().execute(scope, &lhs, &rhs)
				} else {
					let error = format!("unresolved binary operator `{op}`");
					let error = Errors::from(error, self.span());
					Err(error)
				}
			}
			Expr::Sequence(list) => {
				let mut value = Value::from(()).into();
				for it in list.iter() {
					value = it.execute(scope)?;
				}
				Ok(value)
			}
			Expr::Conditional(cond, a, b) => {
				let cond = cond.execute(scope)?;
				let cond = cond.value().bool()?;
				if cond {
					a.execute(scope)
				} else {
					b.execute(scope)
				}
			}
			Expr::If {
				expr,
				if_true,
				if_false,
			} => {
				let expr = expr.execute(scope)?;
				let expr = expr.value().bool()?;
				if expr {
					if_true.execute(scope)
				} else if let Some(if_false) = if_false {
					if_false.execute(scope)
				} else {
					Ok(Value::Unit.into())
				}
			}
			Expr::For {
				var,
				offset,
				from,
				to,
				body,
			} => {
				let from_value = from.execute(scope)?;
				let from_value = from_value.value();
				let from_type = from_value.get_type();
				let from_value = from_value.int_value(&IntType::I128, NumericConversion::None)?;
				let from_value = from_value.signed();

				let to_value = to.execute(scope)?;
				let to_value = to_value.value();
				let to_value = to_value.int_value(&IntType::I128, NumericConversion::None)?;
				let to_value = to_value.signed();

				let step = if from_value <= to_value { 1 } else { -1 };

				let mut cur = from_value;
				loop {
					let value = Value::from(cur).cast_to(&from_type, NumericConversion::None)?;
					scope.set(var.clone(), offset, value);
					body.execute(scope)?;
					if cur == to_value {
						break;
					}
					cur += step;
				}
				Ok(Value::Unit.into())
			}
			Expr::Group(expr) => expr.execute(scope),
			Expr::Token(token) => {
				let error = format!("raw token `{token}` cannot be executed");
				Err(Errors::from(error, self.span().clone()))
			}
			Expr::Raw(..) => {
				let error = format!("unresolved raw token list");
				Err(Errors::from(error, self.span().clone()))
			}
			Expr::Block(head, _) => {
				let error = format!("unresolved block expression: {head}");
				Err(Errors::from(error, self.span().clone()))
			}
		}
	}

	pub fn as_sequence(&self) -> Vec<Node> {
		let mut output = Vec::new();
		match self.expr() {
			Expr::Sequence(list) => output = (*list).clone(),
			_ => output.push(self.clone()),
		}
		output
	}
}

impl Hash for Node {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let value = self.data.value.read().unwrap();
		value.hash(state)
	}
}

impl PartialEq for Node {
	fn eq(&self, other: &Self) -> bool {
		if Arc::as_ptr(&self.data) == Arc::as_ptr(&other.data) {
			true
		} else {
			let va = self.data.value.read().unwrap();
			let vb = other.data.value.read().unwrap();
			*va == *vb && self.data.scope == other.data.scope
		}
	}
}

impl Eq for Node {}

impl Debug for Node {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.expr())?;

		let ctx = Context::get();
		let format = ctx.format().with_mode(Mode::Minimal).with_separator(" @");
		ctx.with_format(format, || write!(f, "{}", self.span()))?;
		if SHOW_INDENT {
			write!(f, "~{}", self.indent())?;
		}
		Ok(())
	}
}

impl Display for Node {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.expr())?;

		let ctx = Context::get();
		let format = ctx.format().with_mode(Mode::Minimal).with_separator(" @");
		ctx.with_format(format, || write!(f, "{:#}", self.span()))?;
		if SHOW_INDENT {
			write!(f, "~{}", self.indent())?;
		}
		Ok(())
	}
}
