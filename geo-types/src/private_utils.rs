// To implement RStar’s traits in the geo-types crates, we need to access to a
// few geospatial algorithms, which are included in this hidden module. This
// hidden module is public so the geo crate can reuse these algorithms to
// prevent duplication. These functions are _not_ meant for public consumption.

use crate::{Line, LineString, Point};
use num_traits::Float;

pub fn line_segment_distance<T>(point: Point<T>, start: Point<T>, end: Point<T>) -> T
where
    T: Float,
{
    if start == end {
        return line_euclidean_length(Line::new(point, start));
    }
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let r =
        ((point.x() - start.x()) * dx + (point.y() - start.y()) * dy) / (dx.powi(2) + dy.powi(2));
    if r <= T::zero() {
        return line_euclidean_length(Line::new(point, start));
    }
    if r >= T::one() {
        return line_euclidean_length(Line::new(point, end));
    }
    let s = ((start.y() - point.y()) * dx - (start.x() - point.x()) * dy) / (dx * dx + dy * dy);
    s.abs() * dx.hypot(dy)
}

pub fn line_euclidean_length<T>(line: Line<T>) -> T
where
    T: Float,
{
    line.dx().hypot(line.dy())
}

pub fn point_line_string_euclidean_distance<T>(p: Point<T>, l: &LineString<T>) -> T
where
    T: Float,
{
    // No need to continue if the point is on the LineString, or it's empty
    if line_string_contains_point(l, p) || l.0.is_empty() {
        return T::zero();
    }
    l.lines()
        .map(|line| line_segment_distance(p, line.start_point(), line.end_point()))
        .fold(T::max_value(), |accum, val| accum.min(val))
}

pub fn point_line_euclidean_distance<T>(p: Point<T>, l: Line<T>) -> T
where
    T: Float,
{
    line_segment_distance(p, l.start_point(), l.end_point())
}

pub fn point_contains_point<T>(p1: Point<T>, p2: Point<T>) -> bool
where
    T: Float,
{
    let distance = line_euclidean_length(Line::new(p1, p2)).to_f32().unwrap();
    relative_eq!(distance, 0.0)
}

pub fn line_string_contains_point<T>(line_string: &LineString<T>, point: Point<T>) -> bool
where
    T: Float,
{
    // LineString without points
    if line_string.0.is_empty() {
        return false;
    }
    // LineString with one point equal p
    if line_string.0.len() == 1 {
        return point_contains_point(Point(line_string.0[0]), point);
    }
    // check if point is a vertex
    if line_string.0.contains(&point.0) {
        return true;
    }
    for line in line_string.lines() {
        if ((line.start.y == line.end.y)
            && (line.start.y == point.y())
            && (point.x() > line.start.x.min(line.end.x))
            && (point.x() < line.start.x.max(line.end.x)))
            || ((line.start.x == line.end.x)
                && (line.start.x == point.x())
                && (point.y() > line.start.y.min(line.end.y))
                && (point.y() < line.start.y.max(line.end.y)))
        {
            return true;
        }
    }
    false
}
