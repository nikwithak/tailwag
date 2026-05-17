use syn::{AttrStyle, Attribute, Field};

pub trait GetAttribute<'a> {
    fn get_attribute(
        &'a self,
        attr_name: &str,
    ) -> Option<&'a Attribute>;
}

impl<'a> GetAttribute<'a> for Field {
    fn get_attribute(
        &'a self,
        attr_name: &str,
    ) -> Option<&'a Attribute> {
        self.attrs
            .iter()
            .filter(|a| a.style == AttrStyle::Outer)
            .find(|a| a.path().is_ident(attr_name))
    }
}
