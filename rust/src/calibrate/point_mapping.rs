//a Imports
use super::{LCamera, Point2D, Point3D, Projection};
use geo_nd::matrix;

//a PointMapping
//tp PointMapping
#[derive(Debug)]
pub struct PointMapping {
    /// The 3D model coordinate this point corresponds to
    ///
    /// This is known for a calibration point!
    model: Point3D,
    /// Screen coordinate
    screen: Point2D,
}

//ip PointMapping
impl PointMapping {
    //fp new
    pub fn new(model: &Point3D, screen: &Point2D) -> Self {
        PointMapping {
            model: *model,
            screen: *screen,
        }
    }

    //mp model
    pub fn model(&self) -> &Point3D {
        &self.model
    }

    //fp show_error
    pub fn show_error(&self, camera: &LCamera) {
        let camera_scr_xy = camera.to_scr_xy(&self.model);
        let dx = self.screen[0] - camera_scr_xy[0];
        let dy = self.screen[1] - camera_scr_xy[1];
        let esq = dx * dx + dy * dy;
        eprintln!(
            "model {} has screen {}, camera maps it to {}, error {}",
            self.model, self.screen, camera_scr_xy, esq
        );
    }

    //fp get_sq_error
    pub fn get_sq_error(&self, camera: &LCamera) -> f64 {
        let camera_scr_xy = camera.to_scr_xy(&self.model);
        let dx = self.screen[0] - camera_scr_xy[0];
        let dy = self.screen[1] - camera_scr_xy[1];
        dx * dx + dy * dy
    }

    //fp add_sq_error_mat
    pub fn add_sq_error_mat(
        &self,
        camera_est: &LCamera,
        desq_dp_mat: &mut [f64; 9],
        desq_dp_eq: &mut [f64; 3],
    ) {
        // view_xyz[i] = direction * (model[i] - position)
        let view_xyz = camera_est.to_camera_space(&self.model);
        dbg!(self);
        dbg!(self.get_sq_error(camera_est));
        dbg!("Estimate of view_xyz for mapping's model is ", view_xyz);
        // That gives an estimate of 'z' for view_xyz
        let z_est = view_xyz[2];

        // The camera spherical xy = direction/z_est * model - direction/z_est*position
        // Err(x) = [direction/z_est * model].x - [direction/z_est*position].x - screen_sph.x
        // Err(x) = 1/z_est * ( [direction*model].x - screen_sph.x*z_est - [direction*position].x)
        // blah_xy = (dmm*model - screen*z_est).xy

        // Convert quaternion to matrix for manipulation
        let dmm_est = camera_est.rotation_matrix();

        // Err(x)^2 = 1/(z_est[i]^2) * ((dmm*model - screen*z_est) - dmm*camera_p).x ^ 2
        //
        // First set blah = dmm * model
        let blah = matrix::transform_vec3(&dmm_est, self.model.as_ref());
        let screen = camera_est.as_spherical_xy(&self.screen);
        dbg!(
            "Estimate of view_xy for mapping's *screen* is",
            screen[0] * z_est,
            screen[1] * z_est
        );
        // blah_x and blah_y are the errors minus the rotated position
        let blah_x = blah[0] - screen[0] * z_est;
        let blah_y = blah[1] - screen[1] * z_est;
        dbg!("Should be about camera posn x y", blah_x, blah_y);

        // Hence Err(x)^2 = 1/(z_est^2) * (blah_x - dmm_est[row 0]*p) ^ 2
        //                = 1/(z_est^2) * (blah_x^2 - 2*blah_x*dmm_est[row 0]*p + (dmm_est[row 0]*p) ^ 2)
        //

        // Err(x)^2 = 0 if (blah_x - dmm_est[row 0]*p) = 0

        //
        // d[row 0]*p   = d[0,0]*p.x + d[1,0]*p.y + d[2,0]*p.z
        // d[row 0]*p^2 = (d[0,0]*p.x) ^ 2 + 2*d[0,0]*p.x*(d[1,0]*p.y + d[2,0]*p.z) + (d[1,0]*p.y + d[2,0]*p.z)^2
        //
        // d/d(p.x)[ d[row 0]*p ] = d[0,0)
        // d/d(p.x)[ d[row 0]*p^2 ] = 2*d[0,0]^2*p.x + 2*d[0,0]*(d[1,0]*p.y + d[2,0]*p.z)
        // d/d(p.x)[ d[row 0]*p^2 ] = 2*d[0,0]*(d[0,0]*p.x + d[1,0]*p.y + d[2,0]*p.z)
        //
        // d/d(p.y)[ d[row 0]*p ] = d[1,0)
        // d/d(p.y)[ d[row 0]*p^2 ] = 2*d[1,0]*p.y + 2*d[1,0]*(d[0,0]*p.x + d[2,0]*p.z)
        // d/d(p.y)[ d[row 0]*p^2 ] = 2*d[1,0]*(p.y + d[0,0]*p.x + d[2,0]*p.z)
        //
        // dE_sq_x / d(p.x) = 1/(z_est^2) * (2 * blahx * d[0,0] + 2*d[0,0]*(d[0,0]*p.x + d[1,0]*p.y + d[2,0]*p.z)
        // dE_sq_x / d(p.x) = 2*d[0,0]/(z_est^2) * (blahx + p.x + d[1,0]*p.y + d[2,0]*p.z)
        // dE_sq_x / d(p.y) = 2*d[1,0]/(z_est^2) * (blahx + p.y + d[0,0]*p.x + d[2,0]*p.z)
        // dE_sq_x / d(p.z) = 2*d[2,0]/(z_est^2) * (blahx + p.z + d[0,0]*p.x + d[1,0]*p.y)
        //
        // Similarly E_sq_y = 1/(z_est[i]^2) * (blahy^2 - 2*blahy*d[row 1]*p + (d[row 1]*p) ^ 2)
        // where blahy[i] =  d[row 1] * m[i]) - s[i].y*z_est[i]
        //
        // dE_sq_y / d(p.x) = 2*d[0,1]/(z_est[i]^2) * (blahy[i] + p.x + d[1,1]*p.y + d[2,1]*p.z)
        // dE_sq_y / d(p.y) = 2*d[1,1]/(z_est[i]^2) * (blahy[i] + p.y + d[0,1]*p.x + d[2,1]*p.z)
        // dE_sq_y / d(p.z) = 2*d[2,1]/(z_est[i]^2) * (blahy[i] + p.z + d[0,1]*p.x + d[1,1]*p.y)
        //
        // dE_sq = Sum(dE_sq_x[i]) + Sum(dE_sq_y[i])
        //
        // We can write dE_sq = M * p + v for some M and v
        //
        // Then when dE_sq = 0, M * p = -v and hence p = - M(inv) * v
        //
        // So for a given p estimate (which yields z_est and blahx/y/z[i] and hence M_est, in a sense)
        //
        let p = camera_est.position();

        // d/dpx
        let de_sq_x_dpx_scale: f64 = 2.0 * dmm_est[0] / (z_est.powf(2.0));
        desq_dp_mat[0] += de_sq_x_dpx_scale * dmm_est[0] * p[0];
        desq_dp_mat[1] += de_sq_x_dpx_scale * dmm_est[1] * p[1];
        desq_dp_mat[2] += de_sq_x_dpx_scale * dmm_est[2] * p[2];
        desq_dp_eq[0] += de_sq_x_dpx_scale * blah_x;

        // d/dpy
        let de_sq_x_dpy_scale: f64 = 2.0 * dmm_est[1] / (z_est.powf(2.0));
        desq_dp_mat[3] += de_sq_x_dpy_scale * dmm_est[3] * p[0];
        desq_dp_mat[4] += de_sq_x_dpy_scale * dmm_est[4] * p[1];
        desq_dp_mat[5] += de_sq_x_dpy_scale * dmm_est[5] * p[2];
        desq_dp_eq[1] += de_sq_x_dpy_scale * blah_x;

        // d/dpz
        let de_sq_x_dpz_scale: f64 = 2.0 * dmm_est[2] / (z_est.powf(2.0));
        desq_dp_mat[6] += de_sq_x_dpz_scale * dmm_est[0] * p[0];
        desq_dp_mat[7] += de_sq_x_dpz_scale * dmm_est[1] * p[1];
        desq_dp_mat[8] += de_sq_x_dpz_scale * dmm_est[2] * p[2];
        desq_dp_eq[2] += de_sq_x_dpz_scale * blah_x;

        // d/dpx
        let de_sq_y_dpx_scale: f64 = 2.0 * dmm_est[3] / (z_est.powf(2.0));
        desq_dp_mat[0] += de_sq_y_dpx_scale * p[0];
        desq_dp_mat[1] += de_sq_y_dpx_scale * dmm_est[4] * p[1];
        desq_dp_mat[2] += de_sq_y_dpx_scale * dmm_est[5] * p[2];
        desq_dp_eq[0] += de_sq_y_dpx_scale * blah_x;

        // d/dpy
        let de_sq_y_dpy_scale: f64 = 2.0 * dmm_est[4] / (z_est.powf(2.0));
        desq_dp_mat[3] += de_sq_y_dpy_scale * dmm_est[3] * p[0];
        desq_dp_mat[4] += de_sq_y_dpy_scale * p[1];
        desq_dp_mat[5] += de_sq_y_dpy_scale * dmm_est[5] * p[2];
        desq_dp_eq[1] += de_sq_y_dpy_scale * blah_y;

        // d/dpz
        let de_sq_y_dpz_scale: f64 = 2.0 * dmm_est[5] / (z_est.powf(2.0));
        desq_dp_mat[6] += de_sq_y_dpz_scale * dmm_est[3] * p[0];
        desq_dp_mat[7] += de_sq_y_dpz_scale * dmm_est[4] * p[1];
        desq_dp_mat[8] += de_sq_y_dpz_scale * p[2];
        desq_dp_eq[2] += de_sq_y_dpz_scale * blah_y;
    }

    //zz All done
}
