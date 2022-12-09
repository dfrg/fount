use std::path::Path;
use std::sync::{Arc, Weak};

/// Shared reference to owned font data.
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct FontData {
    inner: Arc<FontDataInner>,
}

impl FontData {
    /// Creates font data from the specified bytes.
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            inner: Arc::new(FontDataInner::Memory(data)),
        }
    }

    /// Creates font data from the file at the specified path.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let data = std::fs::read(path)?;
        Ok(Self {
            inner: Arc::new(FontDataInner::Memory(data)),
        })
    }

    /// Creates a new weak reference to the data.
    pub fn downgrade(&self) -> WeakFontData {
        WeakFontData {
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// Returns the underlying bytes of the data.
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.data()
    }

    /// Returns the number of strong references to the data.
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl std::ops::Deref for FontData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.data()
    }
}

impl AsRef<[u8]> for FontData {
    fn as_ref(&self) -> &[u8] {
        self.inner.data()
    }
}

#[derive(Debug)]
enum FontDataInner {
    Memory(Vec<u8>),
}

impl FontDataInner {
    pub fn data(&self) -> &[u8] {
        match self {
            Self::Memory(data) => data,
        }
    }
}

/// Weak reference to owned font data.
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct WeakFontData {
    inner: Weak<FontDataInner>,
}

impl WeakFontData {
    /// Upgrades the weak reference.
    pub fn upgrade(&self) -> Option<FontData> {
        Some(FontData {
            inner: self.inner.upgrade()?,
        })
    }
}
