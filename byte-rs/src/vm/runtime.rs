use std::io::Write;

use super::*;

pub struct Runtime {
	data: CodeData,
}

impl Runtime {
	pub fn exec(&self, code: &[Inst]) {
		let mut pc = 0;
		while pc < code.len() {
			let next = &code[pc];
			pc += 1;
			match next {
				Inst::Halt => break,
				Inst::Pass => {}
				Inst::Debug(var) => {
					print!("{var:?}");
				}
				Inst::Print(var) => {
					print!("{var}");
				}
				Inst::PrintStr(str) => {
					let str = self.data.load_data(str);
					let str = unsafe { std::str::from_utf8_unchecked(&str) };
					print!("{str}");
				}
				Inst::PrintFlush => {
					std::io::stdout().flush();
				}
			}
		}
	}
}
