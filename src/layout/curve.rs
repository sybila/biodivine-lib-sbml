use crate::constants::namespaces::NS_LAYOUT;
use crate::core::sbase::SbmlUtils;
use crate::layout::point::Point;
use crate::xml::{
    Document, OptionalSbmlChild, OptionalXmlChild, RequiredSbmlChild, RequiredSbmlProperty,
    RequiredXmlChild, RequiredXmlProperty, XmlElement, XmlList, XmlProperty, XmlPropertyType,
    XmlSubtype, XmlSupertype, XmlWrapper,
};
use sbml_macros::{SBase, XmlWrapper};
use std::fmt::Display;

/// The `<curve>` element of the Layout package specification.
#[derive(Debug, Clone, XmlWrapper, SBase)]
#[sbml_object(name = "curve", namespace = NS_LAYOUT)]
pub struct Curve(XmlElement);

impl Curve {
    /// Creates a new `<curve>` element with the given `segments`.
    pub fn new(document: Document, segments: XmlList<LineSegment>) -> Self {
        let curve = Curve::new_empty(document);
        curve.curve_segments().set(segments);
        curve
    }
    /// The `listOfCurveSegments` child of this `<curve>` element, as a list of [`LineSegment`].
    pub fn curve_segments(&self) -> RequiredSbmlChild<'_, XmlList<LineSegment>> {
        self.required_package_child("listOfCurveSegments", NS_LAYOUT, false)
    }
}

/// The value of the `xsi:type` attribute of a `<layout:curveSegment>` element (Layout Package,
/// Section 3.4): which of the two shapes of a segment the element is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XsiType {
    /// A cubic Bézier segment: a `<curveSegment>` with two base points.
    CubicBezier,
    /// A straight segment: a `<curveSegment>` with a start point and an end point.
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
                Err(message) => Err(message),
            },
            None => Ok(None),
        }
    }

    fn set(&self) -> Option<String> {
        Some(self.to_string().clone())
    }
}

/// The `<curveSegment>` element of the Layout package specification.
#[derive(Debug, Clone, XmlWrapper, SBase)]
#[sbml_object(name = "curveSegment", namespace = NS_LAYOUT)]
pub struct LineSegment(XmlElement);

impl XmlSupertype for LineSegment {}

impl LineSegment {
    /// Creates a new `<curveSegment>` element with the given `start`, `end`.
    pub fn new(document: Document, start: Point, end: Point) -> Self {
        let line = LineSegment::new_empty(document);

        // `LineSegment` deliberately declares no `xsi:type` of its own: it is the supertype that
        // also accepts a `CubicBezier` element, so the variant attribute is written here.
        line.xsi_type().set(&XsiType::LineSegment);
        line.start().set(start);
        line.end().set(end);
        line
    }

    /// The `xsi:type` attribute of this `<curveSegment>` element.
    pub fn xsi_type(&self) -> RequiredSbmlProperty<'_, XsiType> {
        self.required_package_property("xsi:type", NS_LAYOUT, NS_LAYOUT)
    }

    /// The `start` child of this `<curveSegment>` element.
    pub fn start(&self) -> RequiredSbmlChild<'_, Point> {
        self.required_package_child("start", NS_LAYOUT, false)
    }

    /// The `end` child of this `<curveSegment>` element.
    pub fn end(&self) -> RequiredSbmlChild<'_, Point> {
        self.required_package_child("end", NS_LAYOUT, false)
    }

    /// The `basePoint1` child of this `<curveSegment>` element.
    pub fn base_point1(&self) -> OptionalSbmlChild<'_, Point> {
        self.optional_package_child("basePoint1", NS_LAYOUT, false)
    }
    /// The `basePoint2` child of this `<curveSegment>` element.
    pub fn base_point2(&self) -> OptionalSbmlChild<'_, Point> {
        self.optional_package_child("basePoint2", NS_LAYOUT, false)
    }
}

/// The `<curveSegment>` element of the Layout package specification.
#[derive(Debug, Clone, SBase, XmlWrapper)]
#[sbml_object(name = "curveSegment", namespace = NS_LAYOUT, xsi_type = "CubicBezier")]
pub struct CubicBezier(XmlElement);

impl XmlSubtype<LineSegment> for CubicBezier {
    fn try_cast_from_super(value: &LineSegment) -> Option<Self> {
        // Must check the "safe" way, because this method could be used during validation.
        if value.xsi_type().get_checked() == Ok(Some(XsiType::CubicBezier)) {
            Some(CubicBezier::from(value.xml_element().clone()))
        } else {
            None
        }
    }
}

impl CubicBezier {
    /// Creates a new `<curveSegment>` element with the given `start`, `end`, `base_point1`, `base_point2`.
    pub fn new(
        document: Document,
        start: Point,
        end: Point,
        base_point1: Point,
        base_point2: Point,
    ) -> Self {
        // `new_empty` writes the `xsi:type` attribute of the declared identity, which is what
        // tells a cubic bezier apart from a plain line segment.
        let cub = CubicBezier::new_empty(document);

        cub.start().set(start);
        cub.end().set(end);
        cub.base_point1().set(base_point1);
        cub.base_point2().set(base_point2);
        cub
    }
    /// The `xsi:type` attribute of this `<curveSegment>` element.
    pub fn xsi_type(&self) -> RequiredSbmlProperty<'_, XsiType> {
        self.required_package_property("xsi:type", NS_LAYOUT, NS_LAYOUT)
    }
    /// The `start` child of this `<curveSegment>` element.
    pub fn start(&self) -> RequiredSbmlChild<'_, Point> {
        self.required_package_child("start", NS_LAYOUT, false)
    }
    /// The `end` child of this `<curveSegment>` element.
    pub fn end(&self) -> RequiredSbmlChild<'_, Point> {
        self.required_package_child("end", NS_LAYOUT, false)
    }
    /// The `basePoint1` child of this `<curveSegment>` element.
    pub fn base_point1(&self) -> OptionalSbmlChild<'_, Point> {
        self.optional_package_child("basePoint1", NS_LAYOUT, false)
    }
    /// The `basePoint2` child of this `<curveSegment>` element.
    pub fn base_point2(&self) -> OptionalSbmlChild<'_, Point> {
        self.optional_package_child("basePoint2", NS_LAYOUT, false)
    }
}
