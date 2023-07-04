use super::*;

const SOURCE_OFFSET_ALIGNMENT: usize = 64;

impl Context {
	/// Normalized absolute path used as base path for compilation.
	pub fn base_path(&self) -> PathBuf {
		self.read_sources(|data| data.base_path.clone())
	}

	/// Returns the source that contains the given offset.
	pub fn source_at(&self, offset: usize) -> Option<Source> {
		self.read_sources(|data| {
			let sources = data.sources_sorted.read().unwrap();
			let index = sources.partition_point(|it| it.offset > offset);
			sources.get(index).map(|data| Source { data: data.clone() })
		})
	}

	pub fn load_source_text<T: Into<String>, U: Into<String>>(&self, name: T, text: U) -> Source {
		self.read_sources(|data| {
			let data = data.add_text(name.into(), text.into());
			Source { data }
		})
	}

	pub fn load_source_file<P: AsRef<Path>>(&self, path: P) -> Result<Source> {
		self.read_sources(|data| {
			let path = path.as_ref();
			let full_path = if path.is_relative() {
				data.base_path.join(path)
			} else {
				path.to_owned()
			};

			let full_path = match std::fs::canonicalize(full_path) {
				Ok(path) => path,
				Err(err) => {
					let path = path.to_string_lossy();
					let base = data.base_path.to_string_lossy();
					return Err(Errors::from(format!(
						"could not solve `{path}`: {err} (base path is `{base}`)"
					)));
				}
			};

			data.get_file(full_path).map(|data| Source { data })
		})
	}

	fn read_sources<T, P: FnOnce(&Data) -> T>(&self, reader: P) -> T {
		self.read(|data| reader(&data.sources))
	}
}

//====================================================================================================================//
// Source
//====================================================================================================================//

/// Handle to a source file or string.
#[derive(Clone)]
pub struct Source {
	data: Arc<SourceData>,
}

impl Source {
	/// Globally unique offset for this source. Can be used as a unique key
	/// for the source.
	///
	/// Any range between `offset` and `offset + len` can be used to uniquely
	/// identify a position in this source across all sources.
	pub fn offset(&self) -> usize {
		self.data.offset
	}

	/// Length of the source in bytes.
	pub fn len(&self) -> usize {
		self.text().len()
	}

	/// Name for this source.
	///
	/// All sources have a non-empty name, but those are not necessarily unique.
	///
	/// For file sources, the name will represent the source file path.
	pub fn name(&self) -> &str {
		&self.data.name
	}

	/// Absolute normalized path to this source, if it comes from a file.
	///
	/// File sources are de-duplicated based on the path, so no two different
	/// sources will have the same path.
	pub fn path(&self) -> Option<&Path> {
		self.data.path.as_ref().map(|x| x.as_path())
	}

	/// Full source text.
	pub fn text(&self) -> &str {
		self.data.text.as_str()
	}
}

impl PartialEq for Source {
	fn eq(&self, other: &Self) -> bool {
		self.offset() == other.offset()
	}
}

impl Eq for Source {}

impl Ord for Source {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.offset().cmp(&other.offset())
	}
}

impl PartialOrd for Source {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

//====================================================================================================================//
// Context writer and data
//====================================================================================================================//

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
	sources_offset: Arc<RwLock<usize>>,
	sources_by_path: Arc<RwLock<HashMap<PathBuf, Result<Arc<SourceData>>>>>,
	sources_sorted: Arc<RwLock<Vec<Arc<SourceData>>>>,
}

impl Default for Data {
	fn default() -> Self {
		let base_path = std::fs::canonicalize(".").expect("failed to get the canonical current dir, giving up");
		Self {
			base_path,
			sources_offset: Default::default(),
			sources_by_path: Default::default(),
			sources_sorted: Default::default(),
		}
	}
}

//====================================================================================================================//
// Internals
//====================================================================================================================//

struct SourceData {
	name: String,
	text: String,
	path: Option<PathBuf>,
	offset: usize,
}

impl Data {
	fn add_text(&self, name: String, text: String) -> Arc<SourceData> {
		// those need to be locked in the same order as get_file
		let mut offset = self.sources_offset.write().unwrap();
		let mut sorted = self.sources_sorted.write().unwrap();

		let offset = Self::compute_offset(&mut offset, text.len());
		let source = Arc::new(SourceData {
			name,
			text,
			path: None,
			offset,
		});

		sorted.push(source.clone());
		source
	}

	fn get_file(&self, abs_path: PathBuf) -> Result<Arc<SourceData>> {
		// those need to be locked first in the same order as add_text
		let mut offset = self.sources_offset.write().unwrap();
		let mut sorted = self.sources_sorted.write().unwrap();

		// we also need this for the files
		let mut by_path = self.sources_by_path.write().unwrap();

		if let Some(result) = by_path.get(&abs_path) {
			result.clone()
		} else {
			let name = abs_path.to_string_lossy().to_string();
			let result = match std::fs::read(&abs_path) {
				Ok(data) => match String::from_utf8(data) {
					Ok(text) => {
						let offset = Self::compute_offset(&mut offset, text.len());
						let source = Arc::new(SourceData {
							name,
							text,
							path: Some(abs_path.clone()),
							offset,
						});
						sorted.push(source.clone());
						Ok(source)
					}
					Err(err) => Err(Errors::from(format!("opening `{name}`: {err}"))),
				},
				Err(err) => {
					return Err(Errors::from(format!("opening `{name}`: {err}")));
				}
			};

			by_path.insert(abs_path, result.clone());
			result
		}
	}

	fn compute_offset(offset: &mut usize, source_len: usize) -> usize {
		// compute a padding to the next offset so sources never overlap
		let output = *offset;
		let align = SOURCE_OFFSET_ALIGNMENT;
		let new_offset = output + source_len;
		let new_offset = new_offset + (align - new_offset % align);
		*offset = new_offset;
		output
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn source_loading() -> Result<()> {
		let context = Context::get();
		let a = context.load_source_text("input A", "123456");
		let b = context.load_source_text("input B", "some data");
		let c = context.load_source_text("empty", "");
		let d = context.load_source_text("empty", "");

		assert_eq!(a.len(), 6);
		assert_eq!(b.len(), 9);
		assert_eq!(c.len(), 0);
		assert_eq!(a.offset(), 0);
		assert_eq!(b.offset(), SOURCE_OFFSET_ALIGNMENT);
		assert_eq!(c.offset(), SOURCE_OFFSET_ALIGNMENT * 2);
		assert_eq!(d.offset(), SOURCE_OFFSET_ALIGNMENT * 3);

		assert_eq!(a.name(), "input A");
		assert_eq!(b.name(), "input B");
		assert_eq!(c.name(), "empty");
		assert_eq!(a.text(), "123456");
		assert_eq!(b.text(), "some data");
		assert_eq!(c.text(), "");

		assert!(a != b);
		assert!(a != c);
		assert!(a != d);
		assert!(b != c);
		assert!(b != d);
		assert!(c != d);

		let f1 = context.load_source_file("testdata/input.txt")?;
		assert_eq!(f1.text(), "some test data\n");
		assert!(f1.path().unwrap().to_string_lossy().contains("input.txt"));
		assert!(f1.name().contains("input.txt"));

		let f2 = context.load_source_file("./testdata/../testdata/input.txt")?;
		assert!(f2 == f1);
		assert_eq!(f2.text(), f1.text());
		assert_eq!(f2.name(), f1.name());
		assert_eq!(f2.path(), f1.path());
		assert_eq!(f2.offset(), f1.offset());

		Ok(())
	}
}
