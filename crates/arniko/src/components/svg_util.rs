//! Shared SVG path-building helpers used by arniko chart components
//! (SvgLineChart, Sparkline, etc.).

/// Build an SVG polyline path `d` attribute from a list of (x, y) points.
/// Returns `"M x0,y0 L x1,y1 L x2,y2 ..."`.
pub(crate) fn build_path(points: &[(f64, f64)]) -> String {
    if points.is_empty() {
        return String::new();
    }
    let mut s = format!("M {:.1},{:.1}", points[0].0, points[0].1);
    for &(x, y) in &points[1..] {
        s.push_str(&format!(" L {:.1},{:.1}", x, y));
    }
    s
}

/// Build an SVG area-fill path `d` attribute that drops to `bottom_y` and closes.
/// Returns `"M x0,y0 L x1,y1 ... L xn,bottom_y L x0,bottom_y Z"`.
pub(crate) fn build_area(points: &[(f64, f64)], bottom_y: f64) -> String {
    if points.is_empty() {
        return String::new();
    }
    let mut s = format!("M {:.1},{:.1}", points[0].0, points[0].1);
    for &(x, y) in &points[1..] {
        s.push_str(&format!(" L {:.1},{:.1}", x, y));
    }
    let last = points.last().unwrap();
    s.push_str(&format!(
        " L {:.1},{:.1} L {:.1},{:.1} Z",
        last.0, bottom_y, points[0].0, bottom_y
    ));
    s
}
