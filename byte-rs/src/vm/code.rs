use super::*;

#[derive(Copy, Clone, Debug)]
pub enum Inst {
	Halt,
	Pass,
	Debug(Var),
	Print(Var),
	PrintStr(&'static str),
	PrintFlush,
}

#[derive(Copy, Clone)]
pub struct Code {}
