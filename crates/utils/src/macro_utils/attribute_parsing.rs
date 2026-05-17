use syn::{AttrStyle, Attribute, DeriveInput, Field};

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
            .filter(|a| matches!(a.style, AttrStyle::Outer))
            .find(|a| a.path().is_ident(attr_name))
    }
}

impl<'a> GetAttribute<'a> for DeriveInput {
    fn get_attribute(
        &'a self,
        attr_name: &str,
    ) -> Option<&'a Attribute> {
        self.attrs
            .iter()
            .filter(|a| matches!(a.style, AttrStyle::Outer))
            .find(|a| a.path().is_ident(attr_name))
    }
}
