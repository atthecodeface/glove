//a Imports
use std::default::Default;

use geo_nd::{Vector, Vector3};
use ic_base::{utils, Point3D};
use serde::{Deserialize, Serialize};

use crate::{CameraPtMapping, PointMapping};

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
    #[allow(dead_code)]
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
///
/// A model line is two known fixed points in model space. When viewed
/// by a camera, the line is perceived to subtend an angle θ
///
/// This type models such a view of a model line - it does not encode
/// the position or orientation of the camera, just the points on the
/// line and the angle that the camera perceives the line to be
///
/// The camera could lie at any point on a surface of revolution
/// (whose axis is the ModelLine) - i.e. the camera position is a
/// vector along the ModelLine (maybe κ.ML) plus a vector
/// perpendicular to this of a given radius, whose radius depends on
/// κ. For any given κ, the camera can be at any point on a circle
/// of that radius around the line, as this does not change the angle
/// subtended (which depends only on κ and the radius).
///
/// It is thus worth considering a transformation to a
/// ModelLine-centric frame of reference, with the ModelLine being
/// (-1,0,0) to (1,0,0); the locus of the points is a surface of
/// revolution around the X-axis; the locus of points with z=0 can be
/// denoted (κ,ρ,0). (And all points are (κ,ρ.cos(φ),ρ.sin(φ)), for φ
/// in 0..2π.)
///
/// Note that for every point (κ,ρ) the angle subtended by the
/// ModelLine is θ. One can thus the treat the ModelLine is the chord
/// of some circle whose points include the locus (κ,ρ), as the
/// inscribed angle of a (major/minor arc of a) circle for a chord is
/// the constant.
///
/// For θ being greater than 90 (π/2) the locus of the ModelLineSubtended is
/// the minor arc.
///
/// This circle has center (0,C) for some C, if the ModelLine is
/// viewed as (-1,0) to (1,0). The angle subtended by the ModelLine
/// from the centre (0,C) is 2θ; hence C = 1/tan(0) = tan(π/2-0). The
/// angular range for the arc is -π+0 to π-0; the radius R of this
/// circle is the 1/sin(θ).
///
/// We can hence describe the locus of points using two parameters, μ
/// in range -1 to 1 and φ in range 0..2π; we define further
/// γ=μ*(π-0).
///
/// In ModelLineSubtended space with z=0 the locus is (0,C) + (R.sin(γ), R.cos(γ))
///
/// With the surface of rotation about the X axis we have
///
///  [ R.sin(γ), (C+R.cos(γ)).cos(φ), (C+R.cos(γ)).sin(φ)) ]
///
/// The unit normal to this surface is the vector [sin(γ), cos(γ).cos(φ), cos(γ).sin(φ)]
///
/// The final step is to find the transformation from ModelLine space
/// to World space; this can be done by first creating the 'dx'
/// mapping, which maps (-1,0,0) to one end of the ModelLine, and
/// (1,0,0) to the other end of the ModelLine (note that this has
/// length 2, so the unit vector is half of it). *Any* unit normal to this
/// can then be created, and used as 'dy' - and there is a method for
/// this in ModelLine to create a normal. The cross product of these
/// two is then usable as 'dz'.
///
/// Error of a world point
///
/// Given a world point P, a reasonable error with regard to a
/// ModelLineSubtended would be the error in the angle subtended
/// (which should be in the range 0 to π). This can be calculated
/// relatively easily by the scalar product of the vectors from P to
/// the two ModelLine points.
///
/// Background visualiazation
///
/// A way to visualize it is to consider the ModelLine with its center
/// at the origin, with the line being the X axis from (-1,0) to (1,0).
///
/// Now there is a locus of points (κ,ρ) such that the angle between
/// (κ,ρ)->(-1,0) and (κ,ρ)->(1,0) is the angle subtended (θ).
///
/// Indeed, cos(θ) = (κ+1,ρ) . (κ-1,ρ) / |(κ+1,ρ)| |(κ-1,ρ)|
///
/// A different parametertization is the angle between the y-direction
/// and the point (1,0) - call this α. There is a similar angle
/// between the y-direction and the point (-1,0) - call this β. We
/// have the triangle (-1,0), (1,0), (κ,ρ) with angles at those
/// vertices of (90-β, 90-α, θ); the side lengths opposite these
/// angles we can say are (Λβ, Λα, 2).
///
/// From this we know θ = α+β.
///
/// From the sine rule for the triangle we have sin(θ)/2 = sin(90-β)/Λβ,
///
///    Λβ = 2cos(β)/sin(θ) = 2cos(θ-α)/sin(θ)
///
/// From the right-angle triangle (1,0), (κ,0), (κ,ρ) with angles
/// 90-α, 90, α and side lengths ρ, Λβ, 1-k, we have
///
///   ρ = Λβ cos(α)  ; 1-k = Λβ sin(α)
///
///   ρ = 2.cos(θ-α).cos(α)/sin(θ)  ;  k = 1 - 2cos(θ-α).sin(α)/sin(θ)
///
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
    #[allow(dead_code)]
    pub fn error_in_p(&self, p: &Point3D) -> f64 {
        self.model_line.radius_of_circumcircle(p) - self.circle_radius
    }

    //mp error_in_p_angle
    #[allow(dead_code)]
    pub fn error_in_p_angle(&self, p: &Point3D) -> f64 {
        let cos_theta = self.model_line.cos_angle_subtended(p);
        cos_theta.acos() - self.theta
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
    C: CameraPtMapping + Sized,
{
    camera: C,

    /// The derived center-of-gravity for the model lines; i.e. the
    /// average of the midpoints of all the ModelLineSubtended
    model_cog: Point3D,

    /// The set of lines and the angle subtended by each
    lines: Vec<ModelLineSubtended>,
}

impl<C> ModelLineSet<C>
where
    C: CameraPtMapping + Sized,
{
    //cp new
    pub fn new(camera: C) -> Self {
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

    //mp add_line_of_models
    pub fn add_line_of_models(
        &mut self,
        model_p0: Point3D,
        model_p1: Point3D,
        angle: f64,
    ) -> usize {
        let model_line = ModelLine::new(model_p0, model_p1);
        let mls = ModelLineSubtended::new(&model_line, angle);
        let n = self.lines.len();
        self.lines.push(mls);
        self.model_cog = Point3D::zero();
        n
    }

    //mp find_approx_location_using_pt
    #[track_caller]
    pub fn find_approx_location_using_pt<F>(
        &self,
        filter: &F,
        index: usize,
        n_phi: usize,
        n_theta: usize,
    ) -> (Point3D, f64)
    where
        F: Fn(&Point3D) -> bool,
    {
        assert!(
            index < self.lines.len(),
            "Expected index to be within the lines array length"
        );
        let mut pt = Point3D::default();
        let mut min_err2 = 1E8;
        for p in self.lines[index].surface(n_phi, n_theta) {
            if !filter(&p) {
                continue;
            }
            let mut err2 = 0.0;
            for (i, l) in self.lines.iter().enumerate() {
                if i == index {
                    continue;
                }
                let err = l.error_in_p_angle(&p);
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
            let err = l.error_in_p_angle(&p);
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
    #[track_caller]
    pub fn find_best_min_err_location<F>(
        &self,
        filter: &F,
        n_phi: usize,
        n_theta: usize,
    ) -> (Point3D, f64)
    where
        F: Fn(&Point3D) -> bool,
    {
        assert!(
            !self.lines.is_empty(),
            "Cannot find a best_min_err_location with no lines"
        );
        let (mut location, mut err) = self.find_approx_location_using_pt(filter, 0, n_phi, n_theta);
        for i in 1..self.num_lines() {
            let (l, e) = self.find_approx_location_using_pt(filter, i, n_phi, n_theta);
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
