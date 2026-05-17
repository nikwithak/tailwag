use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
};

use crate::files::FileWalker;

#[derive(Debug)]
pub enum TemplateError {
    Unknown(String),
    KeyNotFound(String),
    IOError(std::io::Error),
}

type TemplateParams = HashMap<String, String>;

pub enum TemplateState {
    None,
    Opening,
    ScanningName(String),
    Closing(String),
}

impl From<std::io::Error> for TemplateError {
    fn from(value: std::io::Error) -> Self {
        TemplateError::IOError(value)
    }
}

pub fn fill_template(
    template: &str,
    params: &TemplateParams,
) -> Result<String, TemplateError> {
    let mut current_state = TemplateState::None;
    let mut result = String::new();
    for c in template.chars() {
        type S = TemplateState;
        current_state = match (c, current_state) {
            ('{', S::None) => S::Opening,
            ('{', S::Opening) => S::ScanningName("".into()),
            ('}', S::ScanningName(key)) => S::Closing(key),
            (c, S::ScanningName(mut key)) => {
                // TODO: Validation here?
                key.push(c);
                S::ScanningName(key)
            },
            ('}', S::Closing(key)) => {
                match params.get(key.trim()) {
                    Some(val) => result.push_str(&val),
                    None => return Err(TemplateError::KeyNotFound(key.trim().into())),
                }
                S::None
            },
            (c, S::Closing(key)) => {
                // Hacky, but fits base case. Might screw up other support
                result.push_str("{{");
                result.push_str(&key);
                result.push('}');
                result.push(c);
                S::None
            },
            (c, _) => {
                result.push(c);
                S::None
            },
        }
    }
    Ok(result)
}

/// Reads in the provided file as a String, and processes it through `fill_template()`
/// Does NOT validate anything about the file.
pub fn templatize_file(
    path: &Path,
    params: &TemplateParams,
) -> Result<String, TemplateError> {
    let input_string =
        std::fs::read_to_string(path).expect(&format!("File not found: {}", path.display()));
    return fill_template(&input_string, params);
}

/// Creates a new diretory at `output_root_dir` with the contents of template_root_dir,
/// after filling in the templated files.
/// Writes all files to disk. Fails if the `output_root_dir` exists.
pub fn create_from_template(
    template_root_dir: &Path,
    params: TemplateParams,
    output_root_dir: &Path,
) -> Result<(), TemplateError> {
    // 1. Load directory / file
    let mut output: HashMap<PathBuf, String> = HashMap::new();
    // TODO: Need to update the output path
    if output_root_dir.exists() {
        return Err(TemplateError::Unknown("Output directory already exists".into()));
    }
    let file_walker = FileWalker::new(template_root_dir);

    for file in file_walker {
        let mut input = String::new();
        let (mut file, path) = (file.0?, file.1);
        file.read_to_string(&mut input)?;
        let file_contents = fill_template(&input, &params)?;

        // Strip the old prefix from the path
        let relative_path = path.strip_prefix(template_root_dir).expect(
            "Path prefix is invalid. This is a bug in this software and is not expected to happen.",
        ).to_path_buf();
        let mut output_file_path = output_root_dir.to_path_buf();
        output_file_path.push(relative_path);

        // Templatize the file name
        let output_file_path = output_file_path.into_os_string().into_string().unwrap();
        let output_file_path = fill_template(&output_file_path, &params)?;

        output.insert(output_file_path.into(), file_contents);
    }

    for (new_path, contents) in output {
        std::fs::create_dir_all(new_path.parent().unwrap()).unwrap();
        std::fs::write(&new_path, contents)?;
        println!(
            "Wrote file to disk: {}",
            new_path.to_str().expect("Path is not string writeable. Aborting")
        );
    }

    Ok(())
}

#[test]
fn test_fill_template() {
    assert_eq!(
        "This is a template for Test #1 to {{ check for } } bugs",
        fill_template(
            "This is a {{ ident }} for {{test_number}} to {{ check for } } bugs",
            &vec![
                ("ident".into(), "template".into()),
                ("test_number".into(), "Test #1".into()),
                ("check for".into(), "shouldn't work".into()),
            ]
            .into_iter()
            .collect()
        )
        .unwrap()
    )
}

#[test]
fn test_create_template_dir() {
    use crate::files::FileWalker;
    // NOTE: Must be manually verified for now. This really just runs t on the `example_templates` directory
    let mut template_dir = std::env::current_dir().unwrap();
    template_dir.push("example_templates");
    let mut output_dir = std::env::current_dir().unwrap();
    output_dir.push("test_output");
    assert!(!output_dir.exists(), "Test output dir already exists. Aborting.");

    create_from_template(
        &template_dir,
        vec![("id", "1234567890"), ("name", "Mr. Tester, Sir")]
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect(),
        &output_dir,
    )
    .unwrap();

    let _files = FileWalker::new(&output_dir);

    todo!("This test needs some cleanup. Currently you must manually verify the output is correct in `test_output`");
}
