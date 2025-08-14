use std::rc::Rc;

use geo_nd::Vector;

use ic_base::{Plane, Point2D, Point3D};
use ic_camera::CameraProjection;
use ic_image::Image;
use ic_mesh::Mesh;

use crate::NamedPoint;

//a PatchMesh
//tp PatchMesh
/// A mesh of 2D points witin a plane of a patch
#[derive(Default)]
pub struct PatchMesh {
    model_pts_projected: Vec<Point2D>,
    mesh: Mesh,
    mesh_bounds: (f64, f64, f64, f64),
}

//ip PatchMesh
impl PatchMesh {
    //ap is_empty
    pub fn is_empty(&self) -> bool {
        self.model_pts_projected.is_empty()
    }

    //mp clear
    pub fn clear(&mut self) {
        self.model_pts_projected.clear();
        self.mesh = Mesh::default();
    }

    //mp update_data
    pub fn update_data(&mut self, model_pts_projected: Vec<Point2D>, optimize: bool) {
        self.model_pts_projected = model_pts_projected;
        if optimize {
            self.mesh = Mesh::optimized(self.model_pts_projected.iter().copied(), 1E-2);
        } else {
            self.mesh = Mesh::new(self.model_pts_projected.iter().copied());
            while self
                .mesh
                .remove_duplicates(&self.mesh.find_duplicates(1E-6))
            {}
            self.mesh.create_mesh_triangles();
        }
        self.mesh_bounds = self.model_pts_projected.iter().fold(
            (f64::MAX, 0.0_f64, f64::MAX, 0.0_f64),
            |(lx, rx, by, ty), p| (lx.min(p[0]), rx.max(p[0]), by.min(p[1]), ty.max(p[1])),
        );
    }

    //ap model_pts_projected
    pub fn model_pts_projected(&self) -> &[Point2D] {
        &self.model_pts_projected
    }

    //ap mesh_bounds
    pub fn mesh_bounds(&self) -> &(f64, f64, f64, f64) {
        &self.mesh_bounds
    }

    //ap mesh
    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    //mp split_triangles_to_max_area
    pub fn split_triangles_to_max_area(&mut self, max_area: f64, optimize: bool) -> usize {
        let mut triangles_split = 0;
        let mut centroids: Vec<_> = self.model_pts_projected().iter().cloned().collect();
        for (p0, p1, p2) in self.mesh.triangle_pts() {
            let p01 = self.mesh[p0] - self.mesh[p1];
            let p02 = self.mesh[p0] - self.mesh[p2];
            let area_mmsq = (p01[0] * p02[1] - p01[1] * p02[0]) / 2.0;
            let num_triangles = area_mmsq / max_area;
            let side_split = num_triangles.sqrt();
            if side_split < 2.0 {
                break;
            }
            let side_split = side_split.round() as usize;
            for i in 0..side_split {
                let di = (i as f64) / (side_split as f64);
                for j in 0..(side_split - i) {
                    let dj = (j as f64) / (side_split as f64);
                    let dk = 1.0 - (di + dj);
                    centroids.push((self.mesh[p0] * di + self.mesh[p1] * dj + self.mesh[p2] * dk));
                }
            }
        }
        self.update_data(centroids, false); // optimize);
        triangles_split
    }

    //mp split_mesh_edges
    /// Split just the mesh edges
    pub fn split_mesh_edges(&mut self, max_len: f64, optimize: bool) -> usize {
        let edges_split = self.mesh.split_edges(max_len);
        if edges_split > 0 {
            if optimize {
                while self.mesh.optimize_mesh_quads() {}
            }
        }
        edges_split
    }

    //mp split_mesh_triangles
    /// Split just the mesh triangles
    pub fn split_mesh_triangles(&mut self, max_area: f64, optimize: bool) -> usize {
        let triangles_split = self.mesh.split_triangles(max_area);
        if triangles_split > 0 {
            if optimize {
                while self.mesh.optimize_mesh_quads() {}
            }
        }
        triangles_split
    }
}

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
    expansion_factor: f64,
    render_px_per_model: f64,
    patch_mesh: PatchMesh,
}

//ip Patch
impl Patch {
    //ap plane
    pub fn plane(&self) -> Option<&Plane> {
        if self.plane_ok {
            Some(&self.plane)
        } else {
            None
        }
    }

    //mp rendered_plane_pxy
    /// Where on the rendered plane (0 to width, 0 to height) a
    /// point in 3D (projected onto the plane) would be
    pub fn rendered_plane_pxy(&self, pt: &Point3D) -> Point2D {
        let (lx, _, by, _) = self.patch_mesh.mesh_bounds();
        let origin: Point2D = [*lx, *by].into();
        (self.plane.within_plane(pt) - origin) * self.render_px_per_model
    }

    //mp plane_pxy_in_space
    /// Map the spec point (which is expected to be a 3D point on the plane)
    pub fn plane_pxy_in_space(&self, pt: &Point3D) -> Point3D {
        // This is plane.point_projected_onto
        self.plane.point_in_space(&self.plane.within_plane(pt))
    }

    //mp set_render_px_per_model
    pub fn set_render_px_per_model(&mut self, render_px_per_model: f64) {
        self.render_px_per_model = render_px_per_model;
    }

    //mp set_expansion_factor
    pub fn set_expansion_factor(&mut self, expansion_factor: f64) {
        self.expansion_factor = expansion_factor;
    }

    //cp create
    pub fn create<I>(pts: I) -> Option<Self>
    where
        I: Iterator<Item = Rc<NamedPoint>>,
    {
        let named_points: Vec<_> = pts.filter(|np| np.is_mapped()).collect();
        let model_pts = vec![];
        let plane = Plane::default();
        let mesh = Mesh::default();
        let mut patch = Self {
            named_points,
            model_pts,
            plane_ok: false,
            plane,
            expansion_factor: 1.0,
            patch_mesh: PatchMesh::default(),
            render_px_per_model: 1.0,
        };
        if patch.update_data() {
            Some(patch)
        } else {
            None
        }
    }

    //mp update_data
    pub fn update_data(&mut self) -> bool {
        self.model_pts = self.named_points.iter().map(|np| np.model().0).collect();
        let Some(mut plane) = Plane::best_fit(self.model_pts.iter()) else {
            self.plane_ok = false;
            return false;
        };
        plane.set_tangent(&(self.model_pts[1] - self.model_pts[0]));
        self.plane = plane;
        let d2 = self.plane.distance_sq(self.model_pts.iter());
        eprintln!("Plane: {d2:.4} {:.4?}", self.plane);
        self.plane_ok = true;
        let model_pts_projected = self
            .model_pts
            .iter()
            .map(|p| self.plane.within_plane(p))
            .collect();
        self.patch_mesh.update_data(model_pts_projected, true);

        loop {
            let n_edge = self.patch_mesh.split_mesh_edges(10.0, true);
            let n_triangles = self.patch_mesh.split_mesh_triangles(10.0, true);
            eprintln!("Split {n_edge} edges, {n_triangles} triangles");
            if n_edge == 0 && n_triangles == 0 {
                break;
            }
        }
        true
    }

    //mp sensor_pts
    pub fn sensor_pts<C>(&self, camera: &C) -> Vec<Point2D>
    where
        C: CameraProjection,
    {
        // Find the points on the sensor for all of the mesh points
        self.patch_mesh
            .model_pts_projected()
            .iter()
            .map(|p| self.plane.point_in_space(p))
            .map(|p| camera.world_xyz_to_px_abs_xy(&p))
            .collect()
    }

    //mp mm_per_px_at_center
    pub fn mm_per_px_at_center<C>(&self, camera: &C) -> (f64, f64)
    where
        C: CameraProjection,
    {
        if !self.plane_ok {
            return (0.0, 0.0);
        }

        let (lx, rx, by, ty) = self.patch_mesh.mesh_bounds();
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

    //mp create_img
    pub fn create_img<C, I>(&self, camera: &C, src_img: &I) -> Option<I>
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

        let (lx, rx, by, ty) = self.patch_mesh.mesh_bounds();
        let cx = (lx + rx) / 2.0;
        let cy = (by + ty) / 2.0;
        let dx = (rx - lx) / 2.0 * self.expansion_factor;
        let dy = (ty - by) / 2.0 * self.expansion_factor;

        let lx = (cx - dx);
        let rx = (cx + dx);
        let by = (cy - dy);
        let ty = (cy + dy);

        let lx = lx * self.render_px_per_model;
        let rx = rx * self.render_px_per_model;
        let by = by * self.render_px_per_model;
        let ty = ty * self.render_px_per_model;

        let ilx = lx.floor() as isize;
        let iby = by.floor() as isize;
        let irx = rx.ceil() as isize;
        let ity = ty.ceil() as isize;
        println!("Image bounds {ilx}, {irx}, {iby}, {ity}");

        let width = (irx - ilx) as usize;
        let height = (ity - iby) as usize;
        let mut patch_img = I::new(width, height);

        for x in 0..width {
            let plane_x = ((x as isize + ilx) as f64) / self.render_px_per_model;
            for y in 0..height {
                let plane_y = ((y as isize + iby) as f64) / self.render_px_per_model;
                let plane_xy_in_model = self.plane.point_in_space(&[plane_x, plane_y].into());
                let pxy = camera.world_xyz_to_px_abs_xy(&plane_xy_in_model);

                if pxy[0] < 0.0 || pxy[1] < 0.0 || pxy[0] >= src_w || pxy[1] >= src_h {
                    continue;
                }
                let c = src_img.get(pxy[0] as u32, pxy[1] as u32);
                patch_img.put(x as u32, y as u32, &c);
            }
        }

        let mesh = self.patch_mesh.mesh();
        let c: I::Pixel = 192_u8.into();
        for (p0, p1, p2) in mesh.triangle_pts() {
            let p0 = mesh[p0] * self.render_px_per_model;
            let p1 = mesh[p1] * self.render_px_per_model;
            let p2 = mesh[p2] * self.render_px_per_model;
            let p0 = [p0[0] - lx, p0[1] - by].into();
            let p1 = [p1[0] - lx, p1[1] - by].into();
            let p2 = [p2[0] - lx, p2[1] - by].into();
            patch_img.draw_line(&p0, &p1, &c);
            patch_img.draw_line(&p1, &p2, &c);
            patch_img.draw_line(&p2, &p0, &c);
        }

        Some(patch_img)
    }
}
