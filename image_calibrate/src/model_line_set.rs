//a Imports
use std::default::Default;

use geo_nd::{Vector, Vector3};
use serde::{Deserialize, Serialize};

use crate::{utils, CameraPtMapping, Point3D, PointMapping};

//a ModelLine
//tp ModelLine
/// A line in model space
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ModelLine {
    pts: [Point3D; 2],
}

//ip ModelLine
impl ModelLine {
    #[track_caller]
    pub fn new(p0: Point3D, p1: Point3D) -> Self {
        assert!((p0 - p1).length() > 1E-10);
        Self { pts: [p0, p1] }
    }
    pub fn mid_point(&self) -> Point3D {
        (self.pts[0] + self.pts[1]) / 2.0
    }
    pub fn direction(&self) -> Point3D {
        self.pts[1] - self.pts[0]
    }
    pub fn length(&self) -> f64 {
        (self.pts[1] - self.pts[0]).length()
    }
    pub fn unit_perpendicular(&self) -> Point3D {
        let direction = self.direction();
        let direction = direction / direction.length();
        let k = self.pts[0].cross_product(&direction);
        let l = k.length();
        if l > 0.001 {
            return k / l;
        }
        for v in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
            let perp = direction.cross_product(&v.into());
            let l = perp.length();
            if l > 0.001 {
                return perp / l;
            }
        }
        unreachable!();
    }

    //cp of_vector_eq
    /// From a vector equation p x direction = k
    #[allow(dead_code)]
    pub fn of_vector_eq(direction: &Point3D, k: &Point3D) -> Self {
        let l = direction.length();
        let k = (*k) / l;
        let direction = (*direction) / l;
        let p0 = direction.cross_product(&k);
        let p1 = p0 + direction;
        Self::new(p0, p1)
    }

    //mp closest_pt_to_pt
    #[allow(dead_code)]
    pub fn closest_pt_to_pt(&self, p: &Point3D) -> Point3D {
        let p = *p - self.pts[0];
        let d = self.pts[1] - self.pts[0];
        let len_d2 = d.length_sq();
        let l = p.dot(&d);
        self.pts[0] + (d * (l / len_d2))
    }
    //mp cos_angle_subtended
    /// Get cos(angle) for the angle subtended by the line when viewed from p
    #[allow(dead_code)]
    pub fn cos_angle_subtended(&self, p: &Point3D) -> f64 {
        let p_p0 = *p - self.pts[0];
        let p_p1 = *p - self.pts[1];
        p_p0.dot(&p_p1) / (p_p0.length() * p_p1.length())
    }

    //mp radius_of_circumcircle
    /// Given a model space point p, find the radius of the circle that
    /// includes both the model points of the line and the point p
    ///
    /// The circle will be in the plane of the three points
    ///
    /// The center of the circumcircle will be at the intersection of
    /// the three spheres of *this* radius centred on the three points
    ///
    /// The equation really is |a||b||c| / (2|a x b|)
    ///
    /// which is |c| / 2sin(ACB) = |b| / 2sin(CBA) = |a| / 2sin(BAC)    
    pub fn radius_of_circumcircle(&self, p: &Point3D) -> f64 {
        let p0p = *p - self.pts[0];
        let p1p = *p - self.pts[1];
        let p0p_x_p1p = p0p.cross_product(&p1p);
        self.length() * p0p.length() * p1p.length() / (2.0 * p0p_x_p1p.length())
    }
}

//ip From<(Point3D, Point3D)> for ModelLine
impl From<(Point3D, Point3D)> for ModelLine {
    fn from((p0, p1): (Point3D, Point3D)) -> Self {
        Self::new(p0, p1)
    }
}

//a ModelLineSubtended
//tp ModelLineSubtended
/// A line in model space and an angle subtended
#[derive(Debug, Clone, Default, Serialize)]
pub struct ModelLineSubtended {
    model_line: ModelLine,
    theta: f64,
    #[serde(skip)]
    cos_theta: f64,
    #[serde(skip)]
    sin_theta: f64,
    #[serde(skip)]
    mid_point: Point3D,
    #[serde(skip)]
    length: f64,
    #[serde(skip)]
    circle_radius: f64,
}

//ip Deserialize for ModelLineSubtended
impl<'de> Deserialize<'de> for ModelLineSubtended {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let (model_line, theta) = <(ModelLine, f64)>::deserialize(deserializer)?;
        Ok(ModelLineSubtended::new(&model_line, theta))
    }
}

//ip ModelLineSubtended
impl ModelLineSubtended {
    //cp new
    pub fn new(model_line: &ModelLine, angle: f64) -> Self {
        let mut s = Self {
            model_line: *model_line,
            theta: angle,
            ..Default::default()
        };
        s.derive();
        s
    }

    //fi derive
    fn derive(&mut self) {
        self.cos_theta = self.theta.cos();
        self.sin_theta = self.theta.sin();
        self.mid_point = self.model_line.mid_point();
        self.length = self.model_line.length();
        self.circle_radius = self.length / (2.0 * self.sin_theta);
    }

    //ap model_line
    #[allow(dead_code)]
    pub fn model_line(&self) -> &ModelLine {
        &self.model_line
    }

    //ap angle
    pub fn angle(&self) -> f64 {
        self.theta
    }

    //ap circle_radius
    pub fn circle_radius(&self) -> f64 {
        self.circle_radius
    }

    //ap torus_radius
    pub fn torus_radius(&self) -> f64 {
        self.circle_radius * self.cos_theta
    }

    //mp error_in_p
    pub fn error_in_p(&self, p: &Point3D) -> f64 {
        self.model_line.radius_of_circumcircle(p) - self.circle_radius
    }

    //mp surface
    pub fn surface(&self, n_phi: usize, n_theta: usize) -> ModelLineSubtendedSurfaceIter {
        ModelLineSubtendedSurfaceIter::new(self, n_phi, n_theta)
    }
}

//a ModelLineParametricPoint
//tp ModelLineParametricPoint
#[derive(Debug, Default)]
pub struct ModelLineParametricPoint {
    /// Centre of the torus - midpoint of the model line
    torus_center: Point3D,
    /// Radius of the torus - Circle radius * cos(subtended angle)
    torus_radius: f64,
    /// Radius of the circles - radius of circle such that angle subtended is that of mls
    circle_radius: f64,
    /// Unit vector perpendicular to model line direction
    dx: Point3D,
    /// Unit vector perpendicular to dx and model line direction
    dy: Point3D,
    /// Unit vector in model line direction
    dz: Point3D,

    /// Centre of the circle for given phi
    phi_circle_center: Point3D,
    /// Vector direction in plane of circle perpendicular to dz
    /// with length of the circle radius
    ///
    /// The circle is then cos(t).phi_dxy + sin(t).phi_dz
    phi_dxy: Point3D,
}

//ip ModelLineParametricPoint
impl ModelLineParametricPoint {
    //cp new
    fn new(mls: &ModelLineSubtended) -> Self {
        let torus_center = mls.model_line.mid_point();
        let torus_radius = mls.torus_radius();
        let circle_radius = mls.circle_radius();
        let dz = mls.model_line.direction().normalize();
        let dx = mls.model_line.unit_perpendicular();
        let dy = dz.cross_product(&dx);
        let mut s = Self {
            torus_center,
            torus_radius,
            dx,
            dy,
            dz,
            circle_radius,
            ..Default::default()
        };
        s.derive_from_phi(0.);
        s
    }

    //mp derive_from_phi
    pub fn derive_from_phi(&mut self, phi: f64) {
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();
        let r_cos_phi = cos_phi * self.torus_radius;
        let r_sin_phi = sin_phi * self.torus_radius;
        self.phi_circle_center = self.torus_center + (self.dx * r_cos_phi) + (self.dy * r_sin_phi);
        self.phi_dxy =
            self.dx * (cos_phi * self.circle_radius) + self.dy * (sin_phi * self.circle_radius);
    }

    //mp pt_of_theta
    #[must_use]
    pub fn pt_of_theta(&mut self, theta: f64) -> Point3D {
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();
        self.phi_circle_center - self.phi_dxy * cos_theta + self.dz * sin_theta * self.circle_radius
    }

    //zz All done
}

//a ModelLineSubtendedSurfaceIter
//tp ModelLineSubtendedSurfaceIter
#[derive(Debug)]
pub struct ModelLineSubtendedSurfaceIter {
    n_phi: usize,
    n_theta: usize,

    i_phi: usize,
    i_theta: usize,

    phi_per_i: f64,
    theta_per_i: f64,
    theta_base: f64,
    parametric_point: ModelLineParametricPoint,
}

//ip ModelLineSubtendedSurfaceIter
impl ModelLineSubtendedSurfaceIter {
    fn new(mls: &ModelLineSubtended, n_phi: usize, n_theta: usize) -> Self {
        assert!(n_phi >= 1, "Must have at least 1 phi value");
        assert!(n_theta >= 2, "Must have at least 2 theta values");
        let parametric_point = ModelLineParametricPoint::new(mls);
        let i_phi = 0;
        let i_theta = 0;
        let phi_per_i = std::f64::consts::TAU / (n_phi as f64);
        let theta_range = std::f64::consts::TAU - 2.0 * mls.angle();
        let theta_per_i = theta_range / ((n_theta + 1) as f64);
        let theta_base = mls.angle() + theta_per_i;
        Self {
            n_phi,
            n_theta,
            i_phi,
            i_theta,
            phi_per_i,
            theta_per_i,
            theta_base,
            parametric_point,
        }
    }
}

//ip Iterator for ModelLineSubtendedSurfaceIter
impl std::iter::Iterator for ModelLineSubtendedSurfaceIter {
    type Item = Point3D;
    fn next(&mut self) -> Option<Point3D> {
        if self.i_phi >= self.n_phi {
            None
        } else if self.i_theta >= self.n_theta {
            self.i_phi += 1;
            self.i_theta = 0;
            self.parametric_point
                .derive_from_phi(self.phi_per_i * (self.i_phi as f64));
            self.next()
        } else {
            let theta = self.theta_base + self.theta_per_i * (self.i_theta as f64);
            self.i_theta += 1;
            Some(self.parametric_point.pt_of_theta(theta))
        }
    }
}

//a ModelLineSet
#[derive(Debug)]
pub struct ModelLineSet<C>
where
    C: CameraPtMapping + Clone + Sized,
{
    camera: C,
    model_cog: Point3D,
    lines: Vec<ModelLineSubtended>,
}

impl<C> ModelLineSet<C>
where
    C: CameraPtMapping + Clone + Sized,
{
    //cp new
    pub fn new(camera: &C) -> Self {
        let camera = camera.clone();
        Self {
            camera,
            model_cog: Point3D::zero(),
            lines: vec![],
        }
    }

    //mi derive_model_cog
    pub fn derive_model_cog(&mut self) {
        if self.model_cog.is_zero() {
            let mut sum = Point3D::zero();
            let n = self.lines.len();
            for l in &self.lines {
                sum += l.model_line.mid_point();
            }
            self.model_cog = sum / (n as f64);
        }
    }

    //mp num_lines
    pub fn num_lines(&self) -> usize {
        self.lines.len()
    }

    //mp add_line
    pub fn add_line(&mut self, (pm0, pm1): (&PointMapping, &PointMapping)) -> Option<usize> {
        if pm0.is_unmapped() || pm1.is_unmapped() {
            return None;
        }
        let model_p0 = pm0.model();
        let model_p1 = pm1.model();
        let dir_p0 = self.camera.get_pm_unit_vector(pm0);
        let dir_p1 = self.camera.get_pm_unit_vector(pm1);
        let cos_theta = dir_p0.dot(&dir_p1);
        let angle = cos_theta.acos();
        let model_line = ModelLine::new(model_p0, model_p1);
        let mls = ModelLineSubtended::new(&model_line, angle);
        let n = self.lines.len();
        // eprintln!("push {mls:?}");
        self.lines.push(mls);
        self.model_cog = Point3D::zero();
        Some(n)
    }

    //mp find_approx_location_using_pt
    pub fn find_approx_location_using_pt(
        &self,
        index: usize,
        n_phi: usize,
        n_theta: usize,
    ) -> (Point3D, f64) {
        let mut pt = Point3D::default();
        let mut min_err2 = 1E8;
        for p in self.lines[index].surface(n_phi, n_theta) {
            let mut err2 = 0.0;
            for (i, l) in self.lines.iter().enumerate() {
                if i == index {
                    continue;
                }
                let err = l.error_in_p(&p);
                err2 += err * err;
                if err2 >= min_err2 {
                    break;
                }
            }
            if err2 >= min_err2 {
                continue;
            }
            min_err2 = err2;
            pt = p;
        }
        (pt, min_err2)
    }

    //mp total_err2
    pub fn total_err2(&self, p: Point3D) -> f64 {
        let mut err2 = 0.0;
        for l in &self.lines {
            let err = l.error_in_p(&p);
            err2 += err * err;
        }
        err2
    }

    //mp find_better_min_err_location
    /// fraction should be about 200 max
    pub fn find_better_min_err_location(
        &self,
        pt: Point3D,
        fraction: f64,
    ) -> Option<(Point3D, f64)> {
        let distance = (pt - self.model_cog).length();
        let delta = distance / fraction / 10.0;

        let f = |pt| self.total_err2(pt);
        let dp = utils::delta_p(pt, &f, delta) * 10.0;
        // Note that 0.7*26 is 0.0094%, so this should try to get to
        // about 50E-6 of the distance
        let (moved, err, pt) = utils::better_pt(&pt, &dp, &f, 26, 0.7);
        moved.then_some((pt, err))
    }

    //mp find_best_min_err_location
    pub fn find_best_min_err_location(&self, n_phi: usize, n_theta: usize) -> (Point3D, f64) {
        let (mut location, mut err) = self.find_approx_location_using_pt(0, n_phi, n_theta);
        for i in 1..self.num_lines() {
            let (l, e) = self.find_approx_location_using_pt(i, n_phi, n_theta);
            if e < err {
                err = e;
                location = l;
            }
        }
        eprintln!("Best location {location} : err {err}");

        for i in 0..10 {
            let fraction = 200.0 * (1.4_f64).powi(i);
            while let Some((l, e)) = self.find_better_min_err_location(location, fraction) {
                location = l;
                err = e;
            }
        }
        eprintln!("Better location {location} : err {err}");
        (location, err)
    }

    //zz All done
}
