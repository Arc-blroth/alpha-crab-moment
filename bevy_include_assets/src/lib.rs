//! # bevy_include_assets
//!
//! A Bevy plugin that directly packages assets within the binary.
//! Adapted from Arc-blroth/TrustworthyDolphin's [assets.rs][1].
//!
//! [1]: https://github.com/Arc-blroth/TrustworthyDolphin/blob/main/src/assets.rs

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::app::App;
use bevy::asset::{AssetIo, AssetIoError, BoxedFuture, FileType, Metadata};
use bevy::prelude::{AssetServer, Plugin};

#[derive(Clone, Default, Debug)]
struct EmbeddedAssetIo {
    dirs: HashMap<&'static Path, Vec<PathBuf>>,
    assets: HashMap<&'static Path, &'static [u8]>,
}

impl EmbeddedAssetIo {
    pub fn new(assets: HashMap<&'static Path, &'static [u8]>) -> Self {
        let mut dirs = HashMap::new();
        for asset in &assets {
            if let Some(parent) = asset.0.parent() {
                let directory = match dirs.get_mut(parent) {
                    Some(directory) => directory,
                    None => {
                        dirs.insert(parent, Vec::new());
                        dirs.get_mut(parent).unwrap()
                    }
                };
                directory.push(asset.0.to_path_buf());
            }
        }

        Self { dirs, assets }
    }
}

impl AssetIo for EmbeddedAssetIo {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
        Box::pin(async move {
            match self.assets.get(path) {
                Some(asset) => Ok((*asset).into()),
                None => Err(AssetIoError::NotFound(path.to_path_buf())),
            }
        })
    }

    fn read_directory(&self, path: &Path) -> Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        match self.dirs.get(path) {
            Some(dir) => Ok(Box::new(dir.clone().into_iter())),
            None => Err(AssetIoError::NotFound(path.to_path_buf())),
        }
    }

    fn get_metadata(&self, path: &Path) -> Result<Metadata, AssetIoError> {
        Ok(Metadata::new(if self.dirs.contains_key(path) {
            FileType::Directory
        } else {
            FileType::File
        }))
    }

    fn watch_path_for_changes(&self, _path: &Path) -> Result<(), AssetIoError> {
        Ok(())
    }

    fn watch_for_changes(&self) -> Result<(), AssetIoError> {
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct EmbeddedAssetsPlugin(EmbeddedAssetIo);

impl EmbeddedAssetsPlugin {
    pub fn new(assets: HashMap<&'static ::std::path::Path, &'static [u8]>) -> Self {
        Self(EmbeddedAssetIo::new(assets))
    }
}

impl Plugin for EmbeddedAssetsPlugin {
    fn build(&self, app: &mut App) {
        let asset_server = AssetServer::with_boxed_io(Box::new(self.0.clone()));
        app.insert_resource(asset_server);
    }

    fn name(&self) -> &str {
        "EmbeddedAssetsPlugin"
    }
}

#[macro_export]
macro_rules! include_assets {
    ($path:literal / $($asset:literal),*$(,)?) => {{
        let mut assets = ::std::collections::HashMap::<&'static ::std::path::Path, &'static [u8]>::new();
        $(
            assets.insert($asset.as_ref(), &::std::include_bytes!(::std::concat!($path, "/", $asset))[..]);
        )*
        assets
    }}
}
