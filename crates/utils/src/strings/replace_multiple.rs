use std::collections::HashMap;

mod sealed {
    pub trait Sealed {}
}
impl sealed::Sealed for str {}
impl sealed::Sealed for String {}

pub trait ReplaceMultiple: sealed::Sealed {
    // Does multiple sets of character-to-string replacements in a single pass.
    // Originally created for sanitizing certain XML/HTML characters, it is a general
    // substitution for `string.replace('<', "&lt;").replace('>', "&gt").replace(...);` that can be done in a single pass.
    //
    //
    fn replace_multiple(
        &self,
        replacements: HashMap<char, &str>,
    ) -> String;
}

impl ReplaceMultiple for str {
    fn replace_multiple(
        &self,
        replacements: HashMap<char, &str>,
    ) -> String {
        let mut new_str = String::new();
        for char in self.chars() {
            match replacements.get(&char) {
                Some(repl) => new_str.push_str(repl),
                None => new_str.push(char),
            }
        }
        new_str
    }
}

impl ReplaceMultiple for String {
    fn replace_multiple(
        &self,
        replacements: HashMap<char, &str>,
    ) -> String {
        self.as_str().replace_multiple(replacements)
    }
}

pub trait SanitizeXml: sealed::Sealed {
    fn sanitize_xml(&self) -> String;
}
impl<T: ReplaceMultiple> SanitizeXml for T {
    fn sanitize_xml(&self) -> String {
        self.replace_multiple(
            vec![('&', "&amp;"), ('<', "&lt;"), ('>', "&gt;"), ('"', "&quot;"), ('\'', "&apos;")]
                .into_iter()
                .collect::<HashMap<char, &str>>(),
        )
    }
}
