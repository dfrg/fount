use super::data::*;
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

impl Default for Library {
    fn default() -> Self {
        let mut user = CollectionData::default();
        user.is_user = true;
        Self {
            inner: Arc::new(Inner {
                system: SystemCollectionData::Static(StaticCollection::new(
                    &super::platform::STATIC_DATA,
                )),
                user: Arc::new(RwLock::new(user)),
                user_version: Arc::new(AtomicU64::new(0)),
            }),
        }
    }
}

pub struct Inner {
    pub system: SystemCollectionData,
    pub user: Arc<RwLock<CollectionData>>,
    pub user_version: Arc<AtomicU64>,
}

/// Builder for configuring a font library.
#[derive(Default)]
pub struct LibraryBuilder {}
