use std::io::Write;

use super::*;

pub struct Runtime {}

impl Runtime {
	pub fn exec(code: &[Inst]) {
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
					print!("{str}");
				}
				Inst::PrintFlush => {
					std::io::stdout().flush();
				}
			}
		}
	}
}
