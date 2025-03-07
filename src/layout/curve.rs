use crate::core::sbase::SbmlUtils;
use crate::layout::point::Point;
use crate::xml::{
    RequiredChild, RequiredDynamicChild, RequiredProperty, RequiredXmlChild, RequiredXmlProperty,
    XmlDocument, XmlElement, XmlList, XmlNamedSubtype, XmlPropertyType, XmlSupertype, XmlWrapper,
};
use sbml_macros::{SBase, XmlWrapper};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XsiType {
    CubicBezier,
    LineSegment,
}

impl Display for XsiType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let xsi = match self {
            XsiType::CubicBezier => "CubicBezier",
            XsiType::LineSegment => "LineSegment",
        };

        write!(f, "{}", xsi)
    }
}

impl From<XsiType> for String {
    fn from(value: XsiType) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for XsiType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "CubicBezier" => Ok(XsiType::CubicBezier),
            "LineSegment" => Ok(XsiType::LineSegment),
            _ => Err(format!("xsi:type '{value}' is not valid.")),
        }
    }
}

impl XmlPropertyType for XsiType {
    fn try_get(value: Option<&str>) -> Result<Option<Self>, String> {
        match value {
            Some(value) => match XsiType::try_from(value.to_string()) {
                Ok(xsi) => Ok(Some(xsi)),
                Err(_) => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(self.to_string().clone())
    }
}

#[derive(Debug, Clone, XmlWrapper, SBase)]
pub struct Curve(XmlElement);

impl Curve {
    pub fn curve_segments(&self) -> RequiredChild<XmlList<LineSegment>> {
        self.required_sbml_child("curveSegments")
    }
}

#[derive(Debug, Clone, XmlWrapper, SBase)]
pub struct LineSegment(XmlElement);

impl XmlSupertype for LineSegment {}

impl LineSegment {
    pub fn new(document: XmlDocument, start: Point, end: Point) -> Self {
        let line = LineSegment::new_empty(document, "lineSegment");

        line.xsi_type().set(&XsiType::LineSegment);
        line.start().set(start);
        line.end().set(end);
        line
    }

    pub fn xsi_type(&self) -> RequiredProperty<XsiType> {
        self.required_sbml_property("xsiType")
    }

    pub fn start(&self) -> RequiredDynamicChild<Point> {
        self.required_child("start", "")
    }

    pub fn end(&self) -> RequiredDynamicChild<Point> {
        self.required_child("end", "")
    }
}

#[derive(Debug, Clone, SBase, XmlWrapper)]
pub struct CubicBezier(XmlElement);

impl XmlNamedSubtype<LineSegment> for CubicBezier {
    fn expected_tag_name() -> &'static str {
        "cubicBezier"
    }
}

impl CubicBezier {
    pub fn new(
        document: XmlDocument,
        start: Point,
        end: Point,
        base_point1: Point,
        base_point2: Point,
    ) -> Self {
        let cub = CubicBezier::new_empty(document, "cubicBezier");

        cub.xsi_type().set(&XsiType::CubicBezier);
        cub.start().set(start);
        cub.end().set(end);
        cub.base_point1().set(base_point1);
        cub.base_point2().set(base_point2);
        cub
    }
    pub fn xsi_type(&self) -> RequiredProperty<XsiType> {
        self.required_sbml_property("xsiType")
    }
    pub fn start(&self) -> RequiredDynamicChild<Point> {
        self.required_child("start", "")
    }
    pub fn end(&self) -> RequiredDynamicChild<Point> {
        self.required_child("end", "")
    }
    pub fn base_point1(&self) -> RequiredDynamicChild<Point> {
        self.required_child("basePoint1", "")
    }
    pub fn base_point2(&self) -> RequiredDynamicChild<Point> {
        self.required_child("basePoint2", "")
    }
}
