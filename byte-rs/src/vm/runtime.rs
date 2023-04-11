use super::*;

pub struct Runtime {}

impl Runtime {
	pub fn print(typ: &Type, val: &Value) {
		print::print_value(typ, val);
	}

	pub fn print_type(typ: &Type) {
		print::print_type(typ);
	}
}
