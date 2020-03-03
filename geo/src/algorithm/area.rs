use crate::{
    CoordinateType, GeometryCollection, LineString,
    MultiPolygon, Point, Polygon, Rect, Triangle, GeometryCow
};
use num_traits::Float;

use crate::algorithm::winding_order::twice_signed_ring_area;

/// Signed planar area of a geometry.
///
/// # Examples
///
/// ```
/// use geo::polygon;
/// use geo::algorithm::area::Area;
///
/// let mut polygon = polygon![
///     (x: 0., y: 0.),
///     (x: 5., y: 0.),
///     (x: 5., y: 6.),
///     (x: 0., y: 6.),
///     (x: 0., y: 0.),
/// ];
///
/// assert_eq!(polygon.area(), 30.);
///
/// polygon.exterior_mut(|line_string| {
///     line_string.0.reverse();
/// });
///
/// assert_eq!(polygon.area(), -30.);
/// ```
pub trait Area<'a, T>
where
    T: CoordinateType,
{
    fn area(&'a self) -> T;
}

// Calculation of simple (no interior holes) Polygon area
pub(crate) fn get_linestring_area<T>(linestring: &LineString<T>) -> T
where
    T: CoordinateType,
{
    twice_signed_ring_area(linestring) / (T::one() + T::one())
}

fn area_polygon<T: CoordinateType>(polygon: &Polygon<T>) -> T {
    polygon
        .interiors()
        .iter()
        .fold(get_linestring_area(polygon.exterior()), |total, next| {
            total - get_linestring_area(next)
        })
}

fn area_multi_polygon<T: CoordinateType>(multi_polygon: &MultiPolygon<T>) -> T {
    multi_polygon
        .0
        .iter()
        .fold(T::zero(), |total, next| total + next.area())
}

fn area_geometry_collection<T: CoordinateType>(geometry_collection: &GeometryCollection<T>) -> T {
    geometry_collection
        .iter()
        .fold(T::zero(), |total, geometry| total + geometry.area())
}

fn area_rect<T: CoordinateType>(rect: &Rect<T>) -> T {
    rect.width() * rect.height()
}

fn area_triangle<T: CoordinateType>(triangle: &Triangle<T>) -> T {
    triangle
        .to_lines()
        .iter()
        .fold(T::zero(), |total, line| total + line.determinant())
        / (T::one() + T::one())
}

impl<'a, I: 'a, T: 'a> Area<'a, T> for I
where
    &'a I: Into<GeometryCow<'a, T>>,
    T: CoordinateType,
{
    fn area(&'a self) -> T {
        let geometry_cow: GeometryCow<'a, T> = self.into();
        match geometry_cow {
            GeometryCow::Point(_) => T::zero(),
            GeometryCow::Line(_) => T::zero(),
            GeometryCow::LineString(_) => T::zero(),
            GeometryCow::Polygon(g) => area_polygon(&*g),
            GeometryCow::MultiPoint(_) => T::zero(),
            GeometryCow::MultiLineString(_) => T::zero(),
            GeometryCow::MultiPolygon(g) => area_multi_polygon(&*g),
            GeometryCow::GeometryCollection(g) => area_geometry_collection(&*g),
            GeometryCow::Rect(g) => area_rect(&*g),
            GeometryCow::Triangle(g) => area_triangle(&*g),
        }
    }
}

///////////////////////////////////////////////

struct NewPoint<T: Float>(Point<T>);

impl<'a, T: Float> Into<GeometryCow<'a, T>> for &'a NewPoint<T> {
    fn into(self) -> GeometryCow<'a, T> {
        GeometryCow::Point(std::borrow::Cow::Borrowed(&self.0))
    }
}

struct NewPoint2<T: Float>(T, T);

impl<'a, T: Float> Into<GeometryCow<'a, T>> for &'a NewPoint2<T> {
    fn into(self) -> GeometryCow<'a, T> {
        GeometryCow::Point(std::borrow::Cow::Owned(Point::new(self.0, self.1)))
    }
}

// ///////////////////////////////////////////////

fn foo() {
    let n = NewPoint(geo_types::point!(x: 1.0, y: 1.0));
    let a = n.area();
    let b = n.area();

    let n = NewPoint2(1.0, 1.0);
    let a = n.area();
    let b = n.area();
}

///////////////////////////////////////////////

#[cfg(test)]
mod test {
    use crate::algorithm::area::Area;
    use crate::{line_string, polygon, Coordinate, Line, MultiPolygon, Polygon, Rect, Triangle};

    // Area of the polygon
    #[test]
    fn area_empty_polygon_test() {
        let poly: Polygon<f32> = polygon![];
        assert_relative_eq!(poly.area(), 0.);
    }

    #[test]
    fn area_one_point_polygon_test() {
        let poly = polygon![(x: 1., y: 0.)];
        assert_relative_eq!(poly.area(), 0.);
    }
    #[test]
    fn area_polygon_test() {
        let polygon = polygon![
            (x: 0., y: 0.),
            (x: 5., y: 0.),
            (x: 5., y: 6.),
            (x: 0., y: 6.),
            (x: 0., y: 0.)
        ];
        assert_relative_eq!(polygon.area(), 30.);
    }
    #[test]
    fn rectangle_test() {
        let rect1: Rect<f32> =
            Rect::new(Coordinate { x: 10., y: 30. }, Coordinate { x: 20., y: 40. });
        assert_relative_eq!(rect1.area(), 100.);

        let rect2: Rect<i32> = Rect::new(Coordinate { x: 10, y: 30 }, Coordinate { x: 20, y: 40 });
        assert_eq!(rect2.area(), 100);
    }
    #[test]
    fn area_polygon_inner_test() {
        let poly = polygon![
            exterior: [
                (x: 0., y: 0.),
                (x: 10., y: 0.),
                (x: 10., y: 10.),
                (x: 0., y: 10.),
                (x: 0., y: 0.)
            ],
            interiors: [
                [
                    (x: 1., y: 1.),
                    (x: 2., y: 1.),
                    (x: 2., y: 2.),
                    (x: 1., y: 2.),
                    (x: 1., y: 1.),
                ],
                [
                    (x: 5., y: 5.),
                    (x: 6., y: 5.),
                    (x: 6., y: 6.),
                    (x: 5., y: 6.),
                    (x: 5., y: 5.)
                ],
            ],
        ];
        assert_relative_eq!(poly.area(), 98.);
    }
    #[test]
    fn area_multipolygon_test() {
        let poly0 = polygon![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: 10., y: 10.),
            (x: 0., y: 10.),
            (x: 0., y: 0.)
        ];
        let poly1 = polygon![
            (x: 1., y: 1.),
            (x: 2., y: 1.),
            (x: 2., y: 2.),
            (x: 1., y: 2.),
            (x: 1., y: 1.)
        ];
        let poly2 = polygon![
            (x: 5., y: 5.),
            (x: 6., y: 5.),
            (x: 6., y: 6.),
            (x: 5., y: 6.),
            (x: 5., y: 5.)
        ];
        let mpoly = MultiPolygon(vec![poly0, poly1, poly2]);
        assert_relative_eq!(mpoly.area(), 102.);
        assert_relative_eq!(mpoly.area(), 102.);
    }
    #[test]
    fn area_line_test() {
        let line1 = Line::new(Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 1.0, y: 1.0 });
        assert_relative_eq!(line1.area(), 0.);
    }

    #[test]
    fn area_triangle_test() {
        let triangle = Triangle(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 1.0, y: 0.0 },
            Coordinate { x: 0.0, y: 1.0 },
        );
        assert_relative_eq!(triangle.area(), 0.5);

        let triangle = Triangle(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 0.0, y: 1.0 },
            Coordinate { x: 1.0, y: 0.0 },
        );
        assert_relative_eq!(triangle.area(), -0.5);
    }
}
