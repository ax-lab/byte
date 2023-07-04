use super::*;

impl Context {
	/// Normalized absolute path used as base path for compilation.
	pub fn base_path(&self) -> PathBuf {
		self.read_sources(|data| data.base_path.clone())
	}

	fn read_sources<T, P: FnOnce(&Data) -> T>(&self, reader: P) -> T {
		self.read(|data| reader(&data.sources))
	}
}

impl<'a> ContextWriter<'a> {
	pub fn set_base_path<T: AsRef<Path>>(&mut self, path: T) -> Result<PathBuf> {
		let path = std::fs::canonicalize(path)?;
		let path = self.write_sources(|data| std::mem::replace(&mut data.base_path, path));
		Ok(path)
	}

	fn write_sources<T, P: FnOnce(&mut Data) -> T>(&mut self, writer: P) -> T {
		self.write(|data| writer(&mut data.sources))
	}
}

#[derive(Clone)]
pub(super) struct Data {
	base_path: PathBuf,
}

impl Default for Data {
	fn default() -> Self {
		let base_path = std::fs::canonicalize(".").expect("failed to get the canonical current dir, giving up");
		Self { base_path }
	}
}
