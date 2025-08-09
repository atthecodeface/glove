//a Imports
use std::default::Default;

use geo_nd::{Vector, Vector3};
use ic_base::Point3D;
use serde::{Deserialize, Serialize};

use crate::ModelLine;

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
