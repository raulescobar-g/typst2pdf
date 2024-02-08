use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::path::PathBuf;

use comemo::Prehashed;
use typst::diag::{eco_format, FileError, FileResult, PackageError, PackageResult};
use typst::eval::Tracer;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, PackageSpec, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::Library;

pub fn typst2pdf(files: HashMap<String, String>) -> Vec<u8> {
    let world = WorldWrapper::new(files);
    let mut tracer = Tracer::default();
    let document = typst::compile(&world, &mut tracer).expect("Error compiling typst.");
    return typst_pdf::pdf(&document, None, None);
}

/// Main interface that determines the environment for Typst.
struct WorldWrapper {
    /// Root path to which files will be resolved.
    root: PathBuf,
    /// The content of a source.
    source: Source,
    /// The standard library.
    library: Prehashed<Library>,
    /// Metadata about all known fonts.
    book: Prehashed<FontBook>,
    /// Metadata about all known fonts.
    fonts: Vec<Font>,
    /// Map of all known files.
    files: RefCell<HashMap<FileId, FileEntry>>,
    // Cache directory (e.g. where packages are downloaded to).
    //cache_directory: PathBuf,
    // http agent to download packages.
    //http: ureq::Agent,
    /// Datetime.
    time: time::OffsetDateTime,
}

impl WorldWrapper {
    pub fn new(source: HashMap<String, String>) -> Self {
        let fonts = fonts();
        let file_entries = source.iter().map(|(fname, cont)| {
            let id = FileId::new(None, VirtualPath::new(fname.to_owned()));

            (id, FileEntry::new(Source::new(id, cont.to_owned())))
        });

        let mut files = HashMap::new();

        file_entries.for_each(|(id, entry)| {
            files.insert(id, entry);
        });

        if fonts.len() == 0 {
            panic!("Fontless");
        }

        let source = source.get("main.typ");
        if source.is_none() {
            panic!("Mainless");
        }

        let id = FileId::new(None, VirtualPath::new("/main.typ"));

        Self {
            library: Prehashed::new(Library::build()),
            book: Prehashed::new(FontBook::from_fonts(&fonts)),
            root: PathBuf::from("./"),
            fonts,
            source: Source::new(id, source.unwrap().to_owned()),
            time: time::OffsetDateTime::now_utc(),
            //cache_directory: std::env::var_os("CACHE_DIRECTORY")
            //    .map(|os_path| os_path.into())
            //    .unwrap_or(std::env::temp_dir()),
            //http: ureq::Agent::new(),
            files: RefCell::new(files),
        }
    }
}

/// A File that will be stored in the HashMap.
#[derive(Clone, Debug)]
struct FileEntry {
    source: Source,
}

impl FileEntry {
    fn new(source: Source) -> Self {
        Self { source }
    }
}

impl WorldWrapper {
    /// Helper to handle file requests.
    ///
    /// Requests will be either in packages or a local file.
    fn file(&self, id: FileId) -> FileResult<RefMut<'_, FileEntry>> {
        if let Ok(entry) = RefMut::filter_map(self.files.borrow_mut(), |files| files.get_mut(&id)) {
            return Ok(entry);
        } else {
            panic!("what")
        }
    }

    /// Downloads the package and returns the system path of the unpacked package.
    fn download_package(&self, package: &PackageSpec) -> PackageResult<PathBuf> {
        unimplemented!()
    }
}

impl typst::World for WorldWrapper {
    /// Standard library.
    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    /// Metadata about all known Books.
    fn book(&self) -> &Prehashed<FontBook> {
        &self.book
    }

    /// Accessing the main source file.
    fn main(&self) -> Source {
        self.source.clone()
    }

    /// Accessing a specified source file (based on `FileId`).
    fn source(&self, id: FileId) -> FileResult<Source> {
        Ok(self.file(id)?.source.clone())
    }

    /// Accessing a specified file (non-file).
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.file(id)
            .map(|file| Bytes::from(file.source.text().as_bytes()))
    }

    /// Accessing a specified font per index of font book.
    fn font(&self, id: usize) -> Option<Font> {
        self.fonts.get(id).cloned()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let offset = offset.unwrap_or(-6);
        let offset = time::UtcOffset::from_hms(offset.try_into().ok()?, 0, 0).ok()?;
        let time = self.time.checked_to_offset(offset)?;
        Some(Datetime::Date(time.date()))
    }
}

fn fonts() -> Vec<Font> {
    std::fs::read_dir("fonts")
        .unwrap()
        .map(Result::unwrap)
        .flat_map(|entry| {
            let path = entry.path();
            let bytes = std::fs::read(&path).unwrap();
            let buffer = Bytes::from(bytes);
            let face_count = ttf_parser::fonts_in_collection(&buffer).unwrap_or(1);
            (0..face_count).map(move |face| {
                Font::new(buffer.clone(), face).unwrap_or_else(|| {
                    panic!("failed to load font from {path:?} (face index {face})")
                })
            })
        })
        .collect()
}
