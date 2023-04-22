use super::cell::*;
use super::num::*;
use super::repr::*;
use super::traits::*;

/// Trait for any generic value that can be used with a [`Cell`].
///
/// This trait provides a blanket implementation for all supported values.
pub trait IsValue: CanBox + DynClone + DynEq + HasTraits {}

impl<T: CanBox + DynClone + DynEq + HasTraits> IsValue for T {}

/// Holds a generic value with support for dynamic typing, ARC sharing, and
/// copy-on-write semantics.
#[derive(Clone, PartialEq)]
pub struct Value {
	cell: Cell,
}

impl From<Cell> for Value {
	fn from(cell: Cell) -> Self {
		Value { cell }
	}
}

#[allow(unused)]
impl Value {
	pub fn unit() -> Self {
		Cell::unit().into()
	}

	pub fn never() -> Self {
		Cell::never().into()
	}

	pub fn any_int(value: AnyInt) -> Self {
		Cell::any_int(value).into()
	}

	pub fn any_float(value: AnyFloat) -> Self {
		Cell::any_float(value).into()
	}

	pub fn from<T: IsValue>(value: T) -> Self {
		Cell::from(value).into()
	}

	pub fn is_unit(&self) -> bool {
		self.cell.kind() == CellKind::Unit
	}

	pub fn is_never(&self) -> bool {
		self.cell.kind() == CellKind::Never
	}

	pub fn is_int(&self) -> bool {
		matches!(self.cell.kind(), CellKind::Int(..))
	}

	pub fn is_float(&self) -> bool {
		matches!(self.cell.kind(), CellKind::Float(..))
	}

	pub fn is_any_int(&self) -> bool {
		self.cell.kind() == CellKind::Int(kind::Int::Any)
	}

	pub fn is_any_float(&self) -> bool {
		self.cell.kind() == CellKind::Float(kind::Float::Any)
	}

	pub fn get<T: IsValue>(&self) -> Option<&T> {
		self.cell.get()
	}

	pub fn get_mut<T: CanBox>(&mut self) -> Option<&mut T> {
		self.cell.get_mut()
	}

	pub fn as_str(&self) -> Option<&str> {
		self.cell.as_str()
	}

	pub fn as_traits(&self) -> Option<&dyn IsValue> {
		self.cell.as_value()
	}
}

impl HasTraits for Value {
	fn get_trait(&self, type_id: std::any::TypeId) -> Option<&dyn HasTraits> {
		self.cell.as_value()?.get_trait(type_id)
	}
}

//--------------------------------------------------------------------------------------------------------------------//
// Utility traits
//--------------------------------------------------------------------------------------------------------------------//

impl std::fmt::Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let repr = get_trait!(self, HasRepr);
		if let Some(repr) = repr {
			repr.fmt_display(f)
		} else {
			write!(f, "{self:?}")
		}
	}
}

impl std::fmt::Debug for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let repr = get_trait!(self, HasRepr);
		if let Some(repr) = repr {
			repr.fmt_debug(f)
		} else {
			write!(f, "Value({:?})", self.cell.kind())
		}
	}
}

//--------------------------------------------------------------------------------------------------------------------//
// Tests
//--------------------------------------------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_value_has_traits() {
		let value = SomeType {
			a: "from A".into(),
			b: "from B".into(),
		};
		let value = Value::from(value);
		let va = get_trait!(&value, A).unwrap();
		let vb = get_trait!(&value, B).unwrap();
		assert_eq!(va.a(), "from A");
		assert_eq!(vb.b(), "from B");
	}

	has_traits!(SomeType: A, B);

	trait A {
		fn a(&self) -> String;
	}

	trait B {
		fn b(&self) -> String;
	}

	#[derive(Debug, Clone, PartialEq)]
	struct SomeType {
		a: String,
		b: String,
	}

	impl A for SomeType {
		fn a(&self) -> String {
			self.a.clone()
		}
	}

	impl B for SomeType {
		fn b(&self) -> String {
			self.b.clone()
		}
	}
}
