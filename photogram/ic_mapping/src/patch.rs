use std::rc::Rc;

use geo_nd::Vector;

use ic_base::{Plane, Point2D, Point3D};
use ic_camera::CameraProjection;
use ic_image::Image;
use ic_mesh::Mesh;

use crate::NamedPoint;

//a Patch
//tp Patch
/// A patch in *model space* using NamedPoints
///
/// If the model is updated then the patch has to be updated
pub struct Patch {
    named_points: Vec<Rc<NamedPoint>>,
    model_pts: Vec<Point3D>,
    plane_ok: bool,
    plane: Plane,
    model_pts_projected: Vec<Point2D>,
    mesh: Mesh,
    mesh_bounds: (f64, f64, f64, f64),
    render_px_per_model: f64,
}

impl Patch {
    pub fn create<I>(pts: I) -> Option<Self>
    where
        I: Iterator<Item = Rc<NamedPoint>>,
    {
        let named_points: Vec<_> = pts.filter(|np| np.is_mapped()).collect();
        let model_pts = vec![];
        let plane = Plane::default();
        let model_pts_projected = vec![];
        let mesh = Mesh::default();
        let mut patch = Self {
            named_points,
            model_pts,
            plane,
            plane_ok: false,
            model_pts_projected,
            mesh,
            mesh_bounds: (0.0, 0.0, 0.0, 0.0),
            render_px_per_model: 1.0,
        };
        if patch.update_data() {
            Some(patch)
        } else {
            None
        }
    }

    pub fn update_data(&mut self) -> bool {
        self.model_pts = self.named_points.iter().map(|np| np.model().0).collect();
        let Some(plane) = Plane::best_fit(self.model_pts.iter()) else {
            self.plane_ok = false;
            return false;
        };
        self.plane = plane;
        self.plane_ok = true;
        self.model_pts_projected = self
            .model_pts
            .iter()
            .map(|p| self.plane.within_plane(p))
            .collect();
        self.mesh = Mesh::optimized(self.model_pts_projected.iter().copied());

        self.mesh_bounds = self.model_pts_projected.iter().fold(
            (f64::MAX, 0.0_f64, 0.0_f64, 0.0_f64),
            |(lx, rx, by, ty), p| (lx.min(p[0]), rx.max(p[0]), by.min(p[1]), ty.max(p[1])),
        );
        eprintln!("{:?}", self.mesh);
        true
    }

    pub fn sensor_pts<C>(&self, camera: &C) -> Vec<Point2D>
    where
        C: CameraProjection,
    {
        // Find the points on the sensor for all of the mesh points
        self.model_pts_projected
            .iter()
            .map(|p| self.plane.point_in_space(p))
            .map(|p| camera.world_xyz_to_px_abs_xy(&p))
            .collect()
    }

    pub fn mm_per_px_at_center<C>(&self, camera: &C) -> (f64, f64)
    where
        C: CameraProjection,
    {
        if !self.plane_ok {
            return (0.0, 0.0);
        }

        let (lx, rx, by, ty) = self.mesh_bounds;
        let cx = (lx + rx) / 2.0;
        let cy = (by + ty) / 2.0;
        let model_pt = self.plane.point_in_space(&[cx, cy].into());
        let sensor_pt = camera.world_xyz_to_px_abs_xy(&model_pt);
        let p = self.plane.point_in_space(&[cx + 1.0, cy].into());
        let d0 =
            (model_pt.distance(&p) / sensor_pt.distance(&camera.world_xyz_to_px_abs_xy(&p))).abs();

        let p = self.plane.point_in_space(&[cx - 1.0, cy].into());
        let d1 =
            (model_pt.distance(&p) / sensor_pt.distance(&camera.world_xyz_to_px_abs_xy(&p))).abs();

        let p = self.plane.point_in_space(&[cx, cy + 1.0].into());
        let d2 =
            (model_pt.distance(&p) / sensor_pt.distance(&camera.world_xyz_to_px_abs_xy(&p))).abs();

        let p = self.plane.point_in_space(&[cx, cy - 1.0].into());
        let d3 =
            (model_pt.distance(&p) / sensor_pt.distance(&camera.world_xyz_to_px_abs_xy(&p))).abs();

        (d0.min(d1).min(d2).min(d3), d0.max(d1).max(d2).max(d3))
    }

    pub fn create_img<C, I>(&self, camera: &C, src_img: &I, px_per_model: f64) -> Option<I>
    where
        C: CameraProjection,
        I: Image,
    {
        if !self.plane_ok {
            return None;
        }
        eprintln!("{:?}", self.mm_per_px_at_center(camera));
        // Find the points on the sensor for all of the mesh points
        let src_pts = self.sensor_pts(camera);

        let (src_w, src_h) = src_img.size();
        let src_w = src_w as f64;
        let src_h = src_h as f64;
        if !src_pts
            .iter()
            .any(|p| p[0] >= 0.0 && p[0] < src_w && p[1] >= 0.0 && p[1] < src_h)
        {
            return None;
        }

        let (lx, rx, by, ty) = self.mesh_bounds;
        let lx = lx * px_per_model;
        let rx = rx * px_per_model;
        let by = by * px_per_model;
        let ty = ty * px_per_model;

        let ilx = lx.floor() as isize;
        let iby = by.floor() as isize;
        let irx = rx.ceil() as isize;
        let ity = ty.ceil() as isize;
        println!("Image bounds {ilx}, {irx}, {iby}, {ity}");

        let width = (irx - ilx) as usize;
        let height = (ity - iby) as usize;
        let mut patch_img = I::new(width, height);

        for x in 0..width {
            let plane_x = ((x as isize + ilx) as f64) / px_per_model;
            for y in 0..height {
                let plane_y = ((y as isize + iby) as f64) / px_per_model;
                let model_pt = self.plane.point_in_space(&[plane_x, plane_y].into());
                let pxy = camera.world_xyz_to_px_abs_xy(&model_pt);
                if pxy[0] < 0.0 || pxy[1] < 0.0 || pxy[0] >= src_w || pxy[1] >= src_h {
                    continue;
                }
                let c = src_img.get(pxy[0] as u32, pxy[1] as u32);
                patch_img.put(x as u32, y as u32, &c);
            }
        }

        let c: I::Pixel = 125_u8.into();
        for (p0, p1, p2) in self.mesh.triangles() {
            let p0 = self.mesh[p0] * px_per_model;
            let p1 = self.mesh[p1] * px_per_model;
            let p2 = self.mesh[p2] * px_per_model;
            let p0 = [p0[0] - lx, p0[1] - by].into();
            let p1 = [p1[0] - lx, p1[1] - by].into();
            let p2 = [p2[0] - lx, p2[1] - by].into();
            patch_img.draw_line(&p0, &p1, &c);
            patch_img.draw_line(&p1, &p2, &c);
            patch_img.draw_line(&p2, &p0, &c);
        }
        /*
        let mut xy0 = [0., 0.].into();
        for pxy in corners.iter() {
            let pxy = [pxy[0] - lx, pxy[1] - by].into();
            xy0 = pxy;
        }

        let mut xy0 = [0., 0.].into();
        let c: I::Pixel = 255_u8.into();
        let model_pts_clone = model_pts.clone();
        for pxy in model_pts_clone.map(|p| model_to_flat((*p - model_origin)) * px_per_model) {
            let pxy = [pxy[0] - lx, pxy[1] - by].into();
            patch_img.draw_line(xy0, pxy, &c);
            xy0 = pxy;
        }
        */

        Some(patch_img)
    }
}
