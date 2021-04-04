#[cfg(not(debug_assertions))]
use assets_manager::source::{embed, Embedded};

use assets_manager::{loader::Loader, Asset, AssetCache};
use once_cell::sync::OnceCell;
use skulpin::skia_safe::{Data, Typeface};

#[cfg(debug_assertions)]
pub fn builtin() -> &'static AssetCache {
    static BUILTIN_CACHE: OnceCell<AssetCache> = OnceCell::new();

    BUILTIN_CACHE.get_or_init(|| AssetCache::new("resources").unwrap())
}

#[cfg(not(debug_assertions))]
pub fn builtin() -> &'static AssetCache<Embedded<'static>> {
    static BUILTIN_CACHE: OnceCell<AssetCache<Embedded>> = OnceCell::new();

    BUILTIN_CACHE.get_or_init(|| {
        let embed = Embedded::from(embed!("resources"));
        AssetCache::with_source(embed)
    })
}

pub fn user() -> Option<&'static AssetCache> {
    static USER_CACHE: OnceCell<AssetCache> = OnceCell::new();

    USER_CACHE
        .get_or_try_init(|| AssetCache::new("assets"))
        .ok()
}

pub struct TypefaceContainer(pub Typeface);

pub struct TypefaceLoader;
impl Loader<TypefaceContainer> for TypefaceLoader {
    fn load(
        content: std::borrow::Cow<[u8]>,
        _ext: &str,
    ) -> Result<TypefaceContainer, assets_manager::BoxedError> {
        let data = Data::new_copy(content.as_ref());
        let typeface = Typeface::from_data(data, 0).unwrap();

        Ok(TypefaceContainer { 0: typeface })
    }
}

impl Asset for TypefaceContainer {
    const EXTENSION: &'static str = "ttf";
    type Loader = TypefaceLoader;
}
