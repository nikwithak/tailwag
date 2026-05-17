use std::path::{Path, PathBuf};

use crate::OrmResult;

#[derive(Clone)]
pub struct LocalStorageFileProvider {
    root_path: Box<Path>,
}

impl LocalStorageFileProvider {
    /// Creates a LocalStorageProvider that saves files to the provided `directory`.
    /// This struct will naively do its best to prevent requests from accessing files
    /// outside the sanitized path.
    pub fn new<'a>(directory: &str) -> OrmResult<Self> {
        let directory: &Path = Path::new(directory);
        std::fs::create_dir_all(directory)?;
        Ok(LocalStorageFileProvider {
            root_path: directory.into(),
        })
    }

    fn get_sanitized_path(
        &self,
        relative_path: &Path,
    ) -> OrmResult<PathBuf> {
        let mut full_path = self.root_path.to_path_buf();
        full_path.push(relative_path);

        let contains_path_traversal_exploit = full_path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .find(|c| *c == "..")
            .is_some();

        // TODO: Actually sanitize  it here.
        if !full_path.starts_with(&self.root_path)
            || relative_path.has_root()
            || contains_path_traversal_exploit
        {
            Err(crate::Error::InvalidPath)?;
        }
        Ok(full_path)
    }

    pub fn save_file(
        &self,
        relative_path: &str,
        data: Vec<u8>,
    ) -> OrmResult<()> {
        let relative_path = Path::new(relative_path);
        let path = self.get_sanitized_path(relative_path)?;
        std::fs::write(path, &data)?;
        Ok(())
    }

    pub fn read_file(
        &self,
        relative_path: &str,
    ) -> OrmResult<Vec<u8>> {
        let relative_path = Path::new(relative_path);
        let path = self.get_sanitized_path(relative_path)?;
        let bytes = std::fs::read(path)?;
        Ok(bytes)
    }
}
