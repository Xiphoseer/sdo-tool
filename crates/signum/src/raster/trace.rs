//! # Bitmap tracing algorithm
//!
//! The algorithms in this module are implemented based on the paper
//! [Potrace: a polygon-based tracing algorithm, P. Selinger 2003][potrace] but do not use
//! the library of the same name and are not guaranteed to see the same results.
//!
//! [potrace]: https://potrace.sourceforge.net/potrace.pdf

/// Cardinal Direction
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Dir {
    /// Down (positive Y)
    Down = 0b0001,
    /// Up (negative Y)
    Up = 0b0010,
    /// Left (negative X)
    Left = 0b0100,
    /// Right (positive X)
    Right = 0b1000,
}

impl Dir {
    #[allow(dead_code)]
    fn of((ax, ay): (u32, u32), (bx, by): (u32, u32)) -> Dir {
        if ax == bx {
            if ay < by {
                Dir::Down
            } else {
                Dir::Up
            }
        } else if ax < bx {
            Dir::Right
        } else {
            Dir::Left
        }
    }

    /// ```
    /// # use signum::raster::Dir;
    ///
    /// assert_eq!(Dir::of_dir((0.0, 1.0)), Dir::Down);
    /// assert_eq!(Dir::of_dir((0.0, -1.0)), Dir::Up);
    /// assert_eq!(Dir::of_dir((-1.0, 0.0)), Dir::Left);
    /// assert_eq!(Dir::of_dir((1.0, 0.0)), Dir::Right);
    /// ```
    pub fn of_dir((dx, dy): (f64, f64)) -> Self {
        if dy > 0.0 {
            Dir::Down
        } else if dy < 0.0 {
            Dir::Up
        } else if dx > 0.0 {
            Dir::Right
        } else if dx < 0.0 {
            Dir::Left
        } else {
            panic!()
        }
    }
}

/// A constraint on the angle from a cardinal direction that the points v_k can have
#[derive(Debug, Copy, Clone, PartialEq)]
struct Constraint {
    min: f64,
    max: f64,
}

type Val = fn(f64, f64) -> f64;
type MakeConstraint = fn((f64, f64), (f64, f64)) -> Constraint;

impl Constraint {
    fn of(main_diff: f64, cross: f64) -> Self {
        let main = main_diff.abs();
        let near = (main - 1.0).max(0.0);
        let far = main + 1.0;

        Self {
            min: (cross - 1.0) / if cross >= 1.0 { far } else { near },
            max: (cross + 1.0) / if cross <= -1.0 { far } else { near },
        }
    }

    fn h_val(xdiff: f64, ydiff: f64) -> f64 {
        ydiff / xdiff
    }

    fn v_val(xdiff: f64, ydiff: f64) -> f64 {
        xdiff / ydiff
    }

    fn of_h(v_i: (f64, f64), v_j: (f64, f64)) -> Self {
        let xdiff = v_j.0 - v_i.0;
        let ydiff = v_j.1 - v_i.1;
        Self::of(xdiff, ydiff)
    }

    fn of_v(v_i: (f64, f64), v_j: (f64, f64)) -> Self {
        let xdiff = v_j.0 - v_i.0;
        let ydiff = v_j.1 - v_i.1;
        Self::of(ydiff, xdiff)
    }

    fn update(&mut self, new: Constraint) {
        self.min = self.min.max(new.min);
        self.max = self.max.min(new.max);
    }
}

#[inline]
fn v_sub((ax, ay): (f64, f64), (bx, by): (f64, f64)) -> (f64, f64) {
    (ax - bx, ay - by)
}

#[inline]
#[allow(dead_code)]
fn v_add((ax, ay): (f64, f64), (bx, by): (f64, f64)) -> (f64, f64) {
    (ax + bx, ay + by)
}

#[inline]
fn to_float((x, y): (u32, u32)) -> (f64, f64) {
    (x as f64, y as f64)
}

/// Check for how many points the paths should be considered straight
pub fn straight_up_to(points: &[(u32, u32)], i: usize) -> usize {
    let n = points.len();
    assert!(n >= 4);
    let v_i = to_float(points[i]);
    let i_1 = (i + 1) % n;
    let mut v_jm1 = to_float(points[i_1]); // v_{j - 1}
    let d0 = v_sub(v_jm1, v_i);
    let dir = Dir::of_dir(d0);
    let mut directions = dir as u8;

    let mut j = (i_1 + 1) % n;
    let kmax = (j + n - 3) % n;

    let (make_constraint, val): (MakeConstraint, Val) = match dir {
        Dir::Down | Dir::Up => (Constraint::of_v, Constraint::v_val),
        Dir::Left | Dir::Right => (Constraint::of_h, Constraint::h_val),
    };
    let mut constraint = make_constraint(v_i, v_jm1);

    while j != kmax {
        let v_j = to_float(points[j]);
        let d_j = v_sub(v_j, v_jm1);
        directions |= Dir::of_dir(d_j) as u8;

        if directions == 0b1111 {
            return j - 1;
        }
        let (xdiff, ydiff) = v_sub(v_j, v_i);
        let r = val(xdiff, ydiff);
        if r >= constraint.min && r <= constraint.max {
            // Point valid, update constraint
            constraint.update(make_constraint(v_i, v_j));
        }

        v_jm1 = v_j;
        j = (j + 1) % n;
    }

    return 0;
}

#[cfg(test)]
mod tests {
    use super::Constraint;

    #[test]
    #[rustfmt::skip]
    fn test_constraint() {
        // Down, Right
        let c1 = Constraint::of_h((0.0,0.0), (1.0, 1.0));
        assert_eq!(c1, Constraint { min: 0.0, max: f64::INFINITY });
        let c2 = Constraint::of_h((0.0,0.0), (2.0, 1.0));
        assert_eq!(c2, Constraint { min: 0.0, max: 2.0 });
        let c3 = Constraint::of_h((0.0,0.0), (3.0, 1.0));
        assert_eq!(c3, Constraint { min: 0.0, max: 1.0 });
        
        // Right
        let c4 = Constraint::of_h((0.0,0.0), (1.0, 0.0));
        assert_eq!(c4, Constraint { min: f64::NEG_INFINITY, max: f64::INFINITY });
        let c4 = Constraint::of_h((0.0,0.0), (2.0, 0.0));
        assert_eq!(c4, Constraint { min: -1.0, max: 1.0 });
        let c4 = Constraint::of_h((0.0,0.0), (3.0, 0.0));
        assert_eq!(c4, Constraint { min: -0.5, max: 0.5 });

        // Up, Right
        let c4 = Constraint::of_h((0.0,0.0), (2.0, -2.0));
        assert_eq!(c4, Constraint { min: -3.0, max: -1.0 / 3.0 });

        // Up
        let c4 = Constraint::of_h((0.0,0.0), (0.0, -2.0));
        assert_eq!(c4, Constraint { min: f64::NEG_INFINITY, max: -1.0 });

        // Up, Left
        let c4 = Constraint::of_h((0.0,0.0), (-2.0, -2.0));
        assert_eq!(c4, Constraint { min: -3.0, max: -1.0 / 3.0 });

        // Down
        let c4 = Constraint::of_h((0.0,0.0), (0.0, 2.0));
        assert_eq!(c4, Constraint { min: 1.0, max: f64::INFINITY });
    }
}
