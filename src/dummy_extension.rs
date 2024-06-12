use sbml_macros::{SBase, XmlWrapper};

use crate::{
    core::Model,
    xml::{RequiredChild, RequiredProperty, XmlElement, XmlSubtype, XmlSupertype, XmlWrapper},
};

const EXTENSION_URL: &str = "http://foo";

#[derive(Clone, Debug, XmlWrapper, SBase)]
pub struct ExtendedModel(XmlElement);

impl XmlSubtype<Model> for ExtendedModel {
    fn try_cast_from_super(value: &Model) -> Option<Self> {
        let doc = value.read_doc();
        let declarations = value
            .xml_element()
            .raw_element()
            .collect_namespace_prefixes(&doc, EXTENSION_URL);
        if declarations.is_empty() {
            // The extension is not declared for this model.
            return None;
        } else {
            return unsafe { Some(ExtendedModel::unchecked_cast(value.clone())) };
        }
    }
}

impl XmlSupertype for Model {}

impl ExtendedModel {
    pub fn extra_annotation(&self) -> RequiredChild<XmlElement> {
        RequiredChild::new(self.xml_element(), "extraAnnotation", EXTENSION_URL)
    }

    pub fn extra_property(&self) -> RequiredProperty<String> {
        RequiredProperty::new(self.xml_element(), "extraProperty")
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dummy_extension::ExtendedModel,
        xml::{OptionalXmlChild, RequiredXmlChild, RequiredXmlProperty, XmlSupertype, XmlWrapper},
        Sbml,
    };

    #[test]
    pub fn example_test() {
        let doc = Sbml::read_str(r#"
<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2" xmlns:extra="http://foo" extra:required="false">
    <model id="model-1" extraProperty="Some property text">
        <extra:extraAnnotation>Some child text</extra:extraAnnotation>
    </model>
</sbml>
        "#).unwrap();

        let model = doc.model().get().unwrap();
        let e_model = model.try_downcast::<ExtendedModel>().unwrap();
        assert_eq!(
            e_model.extra_property().get(),
            String::from("Some property text")
        );
        assert_eq!(
            e_model
                .extra_annotation()
                .get()
                .xml_element()
                .text_content(),
            String::from("Some child text")
        );
    }
}
