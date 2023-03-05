use std::path::Path;

use crate::lexer::Input;

pub struct SourceFile {
	path: String,
	data: Vec<u8>,
}

pub fn open_file<P: AsRef<Path>>(path: P) -> std::io::Result<SourceFile> {
	let data = std::fs::read(path.as_ref())?;
	Ok(SourceFile {
		path: path.as_ref().to_string_lossy().to_string(),
		data,
	})
}

impl Input for SourceFile {
	fn len(&self) -> usize {
		self.data.len()
	}

	fn read(&self, pos: usize, end: usize) -> &[u8] {
		&self.data[pos..end]
	}
}

impl std::fmt::Display for SourceFile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.path)
	}
}
