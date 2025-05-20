use crate::constants::namespaces::NS_LAYOUT;
use crate::core::sbase::SbmlUtils;
use crate::layout::point::Point;
use crate::xml::{
    OptionalChild, OptionalXmlChild, RequiredChild, RequiredSbmlProperty, RequiredXmlChild,
    RequiredXmlProperty, XmlDocument, XmlElement, XmlList, XmlPropertyType, XmlSubtype,
    XmlSupertype, XmlWrapper,
};
use sbml_macros::{SBase, XmlWrapper};
use std::fmt::Display;

#[derive(Debug, Clone, XmlWrapper, SBase)]
pub struct Curve(XmlElement);

impl Curve {
    pub fn new(document: XmlDocument, segments: XmlList<LineSegment>) -> Self {
        let curve = Curve::new_empty(document, "curve");
        curve.curve_segments().set(segments);
        curve
    }
    pub fn curve_segments(&self) -> RequiredChild<XmlList<LineSegment>> {
        self.required_package_child("listOfCurveSegments", NS_LAYOUT, false)
    }
}

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
                Err(_) => Err("Invalid xsi type".to_string()),
            },
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(self.to_string().clone())
    }
}

#[derive(Debug, Clone, XmlWrapper, SBase)]
pub struct LineSegment(XmlElement);

impl XmlSupertype for LineSegment {}

impl LineSegment {
    pub fn new(document: XmlDocument, start: Point, end: Point) -> Self {
        let line = LineSegment::new_empty(document, "curveSegment");

        line.xsi_type().set(&XsiType::LineSegment);
        line.start().set(start);
        line.end().set(end);
        line
    }

    pub fn xsi_type(&self) -> RequiredSbmlProperty<XsiType> {
        self.required_package_property("xsi:type", NS_LAYOUT, NS_LAYOUT)
    }

    pub fn start(&self) -> RequiredChild<Point> {
        self.required_package_child("start", NS_LAYOUT, false)
    }

    pub fn end(&self) -> RequiredChild<Point> {
        self.required_package_child("end", NS_LAYOUT, false)
    }

    pub fn base_point1(&self) -> OptionalChild<Point> {
        self.optional_package_child("basePoint1", NS_LAYOUT, false)
    }
    pub fn base_point2(&self) -> OptionalChild<Point> {
        self.optional_package_child("basePoint2", NS_LAYOUT, false)
    }
}

#[derive(Debug, Clone, SBase, XmlWrapper)]
pub struct CubicBezier(XmlElement);

impl XmlSubtype<LineSegment> for CubicBezier {
    fn try_cast_from_super(value: &LineSegment) -> Option<Self> {
        if value.xsi_type().get() == XsiType::CubicBezier {
            Some(unsafe { CubicBezier::unchecked_cast(value.clone()) })
        } else {
            None
        }
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
    pub fn xsi_type(&self) -> RequiredSbmlProperty<XsiType> {
        self.required_package_property("xsi:type", NS_LAYOUT, NS_LAYOUT)
    }
    pub fn start(&self) -> RequiredChild<Point> {
        self.required_package_child("start", NS_LAYOUT, false)
    }
    pub fn end(&self) -> RequiredChild<Point> {
        self.required_package_child("end", NS_LAYOUT, false)
    }
    pub fn base_point1(&self) -> OptionalChild<Point> {
        self.optional_package_child("basePoint1", NS_LAYOUT, false)
    }
    pub fn base_point2(&self) -> OptionalChild<Point> {
        self.optional_package_child("basePoint2", NS_LAYOUT, false)
    }
}
