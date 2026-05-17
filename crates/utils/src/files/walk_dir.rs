use std::{
    fs::File,
    path::{Path, PathBuf},
};

// enum FileWalkerStrategy {
//     IgnoreDirectories,
//     BreadthFirst,
//     DepthFirst,
// }

// An Iterator over a directory of files.
pub struct FileWalker {
    stack: Vec<PathBuf>,
}

impl Iterator for FileWalker {
    type Item = (Result<File, std::io::Error>, PathBuf);

    fn next(&mut self) -> Option<Self::Item> {
        let mut cur_path = self.stack.pop();

        while cur_path.as_ref().map_or(false, |path| path.is_dir()) {
            println!("Found directory: {}", cur_path.as_ref()?.as_os_str().to_str().unwrap());
            let directory = std::fs::read_dir(cur_path?).unwrap();
            for new_path in directory {
                self.stack.push(new_path.unwrap().path());
            }
            cur_path = self.stack.pop();
        }

        cur_path.map(|path| (File::open(&path), path))
    }
}

impl FileWalker {
    pub fn new(path: &Path) -> Self {
        path.into()
    }
}

impl From<&Path> for FileWalker {
    fn from(val: &Path) -> Self {
        FileWalker {
            stack: vec![val.to_path_buf()],
        }
    }
}
