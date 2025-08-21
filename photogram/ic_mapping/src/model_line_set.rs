//a Imports
use std::default::Default;

use geo_nd::Vector;

use ic_base::{utils, Point3D};
use ic_camera::CameraProjection;

use crate::{ModelLine, ModelLineSubtended, PointMapping};

//a ModelLineSet
//tp ModelLineSet
#[derive(Debug)]
pub struct ModelLineSet<C>
where
    C: CameraProjection + Sized,
{
    camera: C,

    /// The derived center-of-gravity for the model lines; i.e. the
    /// average of the midpoints of all the ModelLineSubtended
    model_cog: Point3D,

    /// The set of lines and the angle subtended by each
    lines: Vec<ModelLineSubtended>,
}

//ip ModelLineSet
impl<C> ModelLineSet<C>
where
    C: CameraProjection + Sized,
{
    //cp new
    pub fn new(camera: C) -> Self {
        Self {
            camera,
            model_cog: Point3D::default(),
            lines: vec![],
        }
    }

    //mi derive_model_cog
    pub fn derive_model_cog(&mut self) {
        if self.model_cog.is_zero() {
            let mut sum = Point3D::default();
            let n = self.lines.len();
            for l in &self.lines {
                sum += l.model_line().mid_point();
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
        let dir_p0 = pm0.get_mapped_unit_vector(&self.camera);
        let dir_p1 = pm1.get_mapped_unit_vector(&self.camera);
        let cos_theta = dir_p0.dot(&dir_p1);
        let angle = cos_theta.acos();
        let model_line = ModelLine::new(model_p0, model_p1);
        // eprintln!("{}, {}", pm0.named_point(), pm1.named_point());
        let mls = ModelLineSubtended::new(&model_line, angle);
        let n = self.lines.len();
        // eprintln!("push {mls:?}");
        self.lines.push(mls);
        self.model_cog = Point3D::default();
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
        self.model_cog = Point3D::default();
        n
    }

    //mp quality
    pub fn quality(&self) -> usize {
        let mut mean_direction = Point3D::default();
        let n = self.lines.len();
        for m in &self.lines {
            let d = m.model_line().direction().normalize();
            if d[0].abs() > d[1].abs() && d[0].abs() > d[2].abs() {
                if d[0] > 0.0 {
                    mean_direction += d;
                } else {
                    mean_direction -= d;
                }
            } else if d[1].abs() > d[2].abs() {
                if d[1] > 0.0 {
                    mean_direction += d;
                } else {
                    mean_direction -= d;
                }
            } else {
                if d[2] > 0.0 {
                    mean_direction += d;
                } else {
                    mean_direction -= d;
                }
            }
        }
        mean_direction /= n as f64;
        eprintln!(
            "mlds: quality: mean_direction {} len {}",
            mean_direction,
            mean_direction.length()
        );
        let mean_direction = mean_direction.normalize();
        let mut x: Vec<_> = self
            .lines
            .iter()
            .enumerate()
            .map(|(n, m)| {
                (
                    n,
                    m.model_line()
                        .direction()
                        .normalize()
                        .dot(&mean_direction)
                        .abs(),
                )
            })
            .collect();
        x.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        eprintln!("{x:?}");
        x[0].0
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
        eprintln!("    {pt} : err {min_err2}");
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
        let best_n = self.quality();
        let (location, err) = self.find_approx_location_using_pt(filter, best_n, n_phi, n_theta);
        eprintln!("Location given most orthogonal {best_n} : {location} : err {err}");
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

    //mp find_best_min_err_location2
    #[track_caller]
    pub fn find_best_min_err_location2<E>(
        &self,
        n_phi: usize,
        n_theta: usize,
        max_angle_subtended_error: f64,
        mut err_of_posn: E,
    ) -> (Point3D, f64)
    where
        E: FnMut(&Point3D) -> f64,
    {
        let index = self.quality();
        let mut pt = Point3D::default();
        let mut min_err = 1E8;
        for p in self.lines[index].surface(n_phi, n_theta) {
            let mut too_far_out_of_whack = false;
            for (i, l) in self.lines.iter().enumerate() {
                if i == index {
                    continue;
                }
                let err = l.error_in_p_angle(&p).abs();
                if err > max_angle_subtended_error {
                    too_far_out_of_whack = true;
                    break;
                }
            }
            // if too_far_out_of_whack {
            // continue;
            // }
            let p_err = err_of_posn(&p);
            if p_err >= min_err {
                continue;
            }
            min_err = p_err;
            pt = p;
        }
        eprintln!("mls: find_best_min_err_location2: {pt} : err {min_err}");
        (pt, min_err)
    }

    //zz All done
}
