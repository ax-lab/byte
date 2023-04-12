use crate::core::str::*;

use super::*;

#[derive(Clone, Debug)]
pub enum Literal {
	Bool(bool),
	String(Str),
	Number(Str),
}

impl IsExpr for Literal {}
