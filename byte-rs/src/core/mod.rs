pub mod any;
pub mod error;
pub mod input;
pub mod str;

pub trait IsValue: std::any::Any + Send + Sync + std::fmt::Debug {}

impl<T: std::any::Any + Send + Sync + std::fmt::Debug> IsValue for T {}
