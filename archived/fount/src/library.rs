use super::data::*;
use crate::scan::{scan_path, FontScanner};
use std::io;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};

/// Indexed collection of fonts and associated metadata supporting queries and
/// fallback.
///
/// This struct is opaque and provides shared storage for a font collection.
/// Accessing the collection is done by creating a [`FontContext`](super::context::FontContext)
/// wrapping this struct.
#[derive(Clone)]
pub struct Library {
    pub(crate) inner: Arc<Inner>,
}

impl Library {
    fn new(system: SystemCollectionData) -> Self {
        let mut user = CollectionData::default();
        user.is_user = true;
        Self {
            inner: Arc::new(Inner {
                system,
                user: Arc::new(RwLock::new(user)),
                user_version: Arc::new(AtomicU64::new(0)),
            }),
        }
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
impl Default for Library {
    fn default() -> Self {
        let system =
            SystemCollectionData::Static(StaticCollection::new(&super::platform::STATIC_DATA));
        Self::new(system)
    }
}

pub struct Inner {
    pub system: SystemCollectionData,
    pub user: Arc<RwLock<CollectionData>>,
    pub user_version: Arc<AtomicU64>,
}

/// Builder for configuring a font library.
#[derive(Default)]
pub struct LibraryBuilder {
    scanner: FontScanner,
    system: CollectionData,
    fallback: FallbackData,
}

impl LibraryBuilder {
    pub fn add_system_path<T: AsRef<Path>>(&mut self, path: T) -> Result<(), io::Error> {
        scan_path(
            path.as_ref(),
            &mut self.scanner,
            &mut self.system,
            &mut self.fallback,
        )
    }

    pub fn build(self) -> Library {
        let system = SystemCollectionData::Scanned(ScannedCollectionData {
            collection: self.system,
            fallback: self.fallback,
        });
        Library::new(system)
    }
}
