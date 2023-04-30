use std::{
	cell::{Ref, RefCell},
	collections::HashMap,
	rc::Rc,
};

use crate::core::*;

use super::*;

#[derive(Clone, Debug)]
pub enum Inst {
	Halt,
	Pass,
	Debug(Value),
	Print(Value),
	PrintStr(RawData),
	PrintFlush,
}

#[derive(Copy, Clone, Debug)]
pub struct RawData {
	len: usize,
	pos: usize,
}

/// Provides a higher-level abstraction for code that is closer to the actual
/// language source text.
///
/// The goal of this abstraction is to make it easier to generate code that
/// can either be compiled to VM instructions or transpiled to other languages.
#[derive(Clone)]
pub struct Code {
	data: CodeData,
}

impl Code {
	pub fn compile(output: &mut Vec<Inst>) {
		todo!()
	}
}

/// Stores shared immutable data used by [`Code`] structs.
///
/// Provides interior mutability for adding new data and low-overhead clone
/// using reference counting.
#[derive(Clone, Default)]
pub struct CodeData {
	store: Rc<RefCell<CodeDataStore>>,
}

impl CodeData {
	pub fn intern_data(&self, data: &[u8]) -> RawData {
		{
			let mut store = self.store.borrow();
			if let Some(entry) = store.hash.get(data) {
				return *entry;
			}
		}

		let key = data.into();
		let val = self.save_data(data);
		let mut store = self.store.borrow_mut();
		store.hash.insert(key, val);
		val
	}

	pub fn save_data(&self, data: &[u8]) -> RawData {
		if data.len() == 0 {
			RawData { pos: 0, len: 0 }
		} else {
			let mut store = self.store.borrow_mut();
			let pos = store.data.len();
			store.data.extend(data);
			RawData {
				pos,
				len: data.len(),
			}
		}
	}

	pub fn load_data(&self, raw: &RawData) -> Ref<[u8]> {
		let RawData { pos, len } = *raw;
		let store = self.store.borrow();
		Ref::map(store, |x| &x.data[pos..len])
	}
}

#[derive(Default)]
struct CodeDataStore {
	data: Vec<u8>,
	hash: HashMap<Vec<u8>, RawData>,
}
