use super::*;

const SOURCE_OFFSET_ALIGNMENT: usize = 64;

impl Context {
	/// Normalized absolute path used as base path for compilation.
	pub fn base_path(&self) -> PathBuf {
		self.read_sources(|data| data.base_path.clone())
	}

	/// Default tab-width used by input sources created from this context.
	///
	/// See [`DEFAULT_TAB_WIDTH`].
	pub fn tab_width(&self) -> usize {
		self.read_sources(|ctx| ctx.tab_width)
	}

	/// Returns the source that contains the given offset.
	pub fn source_at(&self, offset: usize) -> Option<Source> {
		self.read_sources(|data| {
			let sources = data.sources_sorted.read().unwrap();
			let index = sources.partition_point(|it| it.offset + it.text.len() < offset);
			if let Some(source) = sources.get(index) {
				if source.offset <= offset {
					Some(Source { data: source.clone() })
				} else {
					None
				}
			} else {
				None
			}
		})
	}

	pub fn load_source_text<T: Into<String>, U: Into<String>>(&self, name: T, text: U) -> Source {
		self.read_sources(|data| {
			let name = name.into();
			assert!(name.len() > 0);
			let data = data.add_text(name, text.into());
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

	fn read_sources<T, P: FnOnce(&ContextSources) -> T>(&self, reader: P) -> T {
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

impl Default for Source {
	fn default() -> Self {
		Context::get().source_at(0).unwrap()
	}
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

	/// Tab-width for this source. See [`DEFAULT_TAB_WIDTH`].
	pub fn tab_width(&self) -> usize {
		*self.data.tab_width.read().unwrap()
	}

	/// Sets a custom tab-width for this source.
	///
	/// This has effect globally. It is meant to set the tab-width parsed from
	/// directives in the file.
	pub fn set_tab_width(&self, size: usize) -> usize {
		let mut tab_width = self.data.tab_width.write().unwrap();
		std::mem::replace(&mut tab_width, if size == 0 { DEFAULT_TAB_WIDTH } else { size })
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

	fn as_ptr(&self) -> *const SourceData {
		Arc::as_ptr(&self.data)
	}
}

impl PartialEq for Source {
	fn eq(&self, other: &Self) -> bool {
		// consider the default source equal regardless of the context
		(self.data.offset == 0 && other.data.offset == 0) || self.as_ptr() == other.as_ptr()
	}
}

impl Eq for Source {}

impl Ord for Source {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.offset()
			.cmp(&other.offset())
			.then_with(|| self.as_ptr().cmp(&other.as_ptr()))
	}
}

impl PartialOrd for Source {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Hash for Source {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_ptr().hash(state);
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

	pub fn set_tab_width(&mut self, size: usize) -> usize {
		self.write_sources(|ctx| {
			std::mem::replace(&mut ctx.tab_width, if size == 0 { DEFAULT_TAB_WIDTH } else { size })
		})
	}

	fn write_sources<T, P: FnOnce(&mut ContextSources) -> T>(&mut self, writer: P) -> T {
		self.write(|data| writer(&mut data.sources))
	}
}

#[derive(Clone)]
pub(super) struct ContextSources {
	tab_width: usize,
	base_path: PathBuf,
	sources_offset: Arc<RwLock<usize>>,
	sources_by_path: Arc<RwLock<HashMap<PathBuf, Result<Arc<SourceData>>>>>,
	sources_sorted: Arc<RwLock<Vec<Arc<SourceData>>>>,
}

impl Default for ContextSources {
	fn default() -> Self {
		let base_path = std::fs::canonicalize(".").expect("failed to get the canonical current dir, giving up");
		let output = Self {
			base_path,
			tab_width: DEFAULT_TAB_WIDTH,
			sources_offset: Default::default(),
			sources_by_path: Default::default(),
			sources_sorted: Default::default(),
		};

		// add the default empty source text at the zero offset
		output.add_text(String::new(), String::new());
		output
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
	tab_width: RwLock<usize>,
}

impl ContextSources {
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
			tab_width: RwLock::new(self.tab_width),
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
							tab_width: RwLock::new(self.tab_width),
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
		const ALIGN: usize = SOURCE_OFFSET_ALIGNMENT;

		let context = Context::get();
		let a = context.load_source_text("input A", "123456");
		let b = context.load_source_text("input B", "some data");
		let c = context.load_source_text("empty", "");
		let d = context.load_source_text("empty", "");

		assert_eq!(a.len(), 6);
		assert_eq!(b.len(), 9);
		assert_eq!(c.len(), 0);

		assert_eq!(a.offset(), ALIGN); // the zero offset is reserved
		assert_eq!(b.offset(), ALIGN * 2);
		assert_eq!(c.offset(), ALIGN * 3);
		assert_eq!(d.offset(), ALIGN * 4);

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

		assert!(context.source_at(ALIGN) == Some(a.clone()));
		assert!(context.source_at(ALIGN + 1) == Some(a.clone()));
		assert!(context.source_at(ALIGN + 6) == Some(a.clone()));

		assert!(context.source_at(7) == None);
		assert!(context.source_at(ALIGN * 2) == Some(b.clone()));
		assert!(context.source_at(ALIGN * 2 + 1) == Some(b.clone()));
		assert!(context.source_at(ALIGN * 2 + b.len()) == Some(b.clone()));

		assert!(context.source_at(ALIGN + b.len() + 1) == None);
		assert!(context.source_at(ALIGN * 3 - 1) == None);
		assert!(context.source_at(ALIGN * 3) == Some(c.clone()));

		assert!(context.source_at(ALIGN * 4) == Some(d.clone()));
		assert!(context.source_at(ALIGN * 5) == Some(f1.clone()));
		assert!(context.source_at(ALIGN * 5 + f1.len()) == Some(f1.clone()));
		assert!(context.source_at(ALIGN * 5 + f1.len() + 1) == None);

		Ok(())
	}

	#[test]
	fn default_source() {
		let a = Source::default();
		let b = Source::default();

		let context = Context::get();
		let c = context.source_at(0).unwrap();

		assert!(a == b);
		assert!(a == c);
		assert!(b == c);

		assert_eq!(a.text(), "");
		assert_eq!(a.name(), "");
	}
}
