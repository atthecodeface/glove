//a Imports
use geo_nd::{SqMatrix, Vector};

use ic_base::{Mat3x3, Plane, Point2D, Point3D};

use crate::Image;

//a Patch
//tp Patch
#[derive(Debug)]
pub struct Patch<I: Image> {
    img: I,
    flat_origin: Point2D,
    model_origin: Point3D,
    flat_to_model: Mat3x3,
}

//ip Patch
impl<I: Image> Patch<I> {
    //ap img
    pub fn img(&self) -> &I {
        &self.img
    }

    //ap flat_origin
    pub fn flat_origin(&self) -> Point2D {
        self.flat_origin
    }

    //ap model_origin
    pub fn model_origin(&self) -> Point3D {
        self.model_origin
    }

    //mp normal
    pub fn normal(&self) -> Point3D {
        [
            self.flat_to_model[2],
            self.flat_to_model[5],
            self.flat_to_model[8],
        ]
        .into()
    }

    //cp create
    /// Create a patch from a source image and a set of N model points
    /// *which should be on a plane*, where the first is the origin,
    /// the second is the X axis direction, and a scale in px per
    /// model unit is provided
    ///
    /// Additionally a function to map from Model space to Image space
    /// is needed
    ///
    /// None is returned if the image would have been empty (no valid pixels)
    pub fn create<'a, F, P>(
        src_img: &I,
        px_per_model: f64,
        model_pts: P,
        model_to_flat: &F,
    ) -> Result<Option<Self>, String>
    where
        F: Fn(Point3D) -> Point2D,
        P: Clone + ExactSizeIterator<Item = &'a Point3D>,
    {
        let mut model_pts_clone = model_pts.clone();

        let model_plane = Plane::best_fit(model_pts.clone()).unwrap();
        //        let model_normal = model_x_axis.cross_product(&p_sum).normalize();
        let model_normal = *model_plane.normal();

        let num_model_pts = model_pts.len();
        let model_origin = *model_pts_clone.next().unwrap();
        let model_origin = model_plane.point_projected_onto(&model_origin).0;
        let flat_origin = model_to_flat(model_origin);

        let model_x_axis = (model_plane
            .point_projected_onto(model_pts_clone.next().unwrap())
            .0
            - model_origin)
            .normalize();
        let p_sum = model_pts_clone.fold(Point3D::default(), |acc, p| acc + *p);
        let p_sum = p_sum - (model_origin * (num_model_pts - 2) as f64);
        let model_y_axis = model_normal.cross_product(&model_x_axis).normalize();

        let flat_to_model: Mat3x3 = [
            model_x_axis[0],
            model_y_axis[0],
            model_normal[0],
            model_x_axis[1],
            model_y_axis[1],
            model_normal[1],
            model_x_axis[2],
            model_y_axis[2],
            model_normal[2],
        ]
        .into();
        let flat_to_model_inv = flat_to_model.inverse();

        let model_pts_clone = model_pts.clone();
        let flat_pts: Vec<_> = model_pts_clone.map(|model| model_to_flat(*model)).collect();
        let (src_w, src_h) = src_img.size();
        let src_w = src_w as f64;
        let src_h = src_h as f64;
        if !flat_pts
            .iter()
            .any(|p| p[0] >= 0.0 && p[0] < src_w && p[1] >= 0.0 && p[1] < src_h)
        {
            return Ok(None);
        }

        let model_pts_clone = model_pts.clone();
        let corners: Vec<_> = model_pts_clone
            .map(|p| flat_to_model_inv.transform(&(*p - model_origin)) * px_per_model)
            .collect();

        eprintln!(
            "Model origin {model_origin}, axes {model_x_axis}, {model_y_axis}, {model_normal}, flat corners {corners:?}"
        );

        let (lx, rx, by, ty) = corners.iter().fold(
            (f64::MAX, 0.0_f64, 0.0_f64, 0.0_f64),
            |(lx, rx, by, ty), p| (lx.min(p[0]), rx.max(p[0]), by.min(p[1]), ty.max(p[1])),
        );

        let ilx = lx.floor() as isize;
        let iby = by.floor() as isize;
        let irx = rx.ceil() as isize;
        let ity = ty.ceil() as isize;
        println!("Image bounds {ilx}, {irx}, {iby}, {ity}");

        let width = (irx - ilx) as usize;
        let height = (ity - iby) as usize;
        let mut patch_img = I::new(width, height);

        for x in 0..width {
            let model_fx =
                model_origin + model_x_axis * (((x as isize + ilx) as f64) / px_per_model);
            for y in 0..height {
                let mfy = model_y_axis * (((y as isize + iby) as f64) / px_per_model);
                let model_pt = model_fx + mfy;
                let pxy = model_to_flat(model_pt);
                if pxy[0] < 0.0 || pxy[1] < 0.0 || pxy[0] >= src_w || pxy[1] >= src_h {
                    continue;
                }
                let c = src_img.get(pxy[0] as u32, pxy[1] as u32);
                patch_img.put(x as u32, y as u32, &c);
            }
        }

        let mut xy0 = [0., 0.].into();
        let c: I::Pixel = 125_u8.into();
        for pxy in corners.iter() {
            let pxy = [pxy[0] - lx, pxy[1] - by].into();
            patch_img.draw_line(xy0, pxy, &c);
            xy0 = pxy;
        }

        let mut xy0 = [0., 0.].into();
        let c: I::Pixel = 255_u8.into();
        let model_pts_clone = model_pts.clone();
        for pxy in model_pts_clone.map(|p| model_to_flat(*p - model_origin) * px_per_model) {
            let pxy = [pxy[0] - lx, pxy[1] - by].into();
            patch_img.draw_line(xy0, pxy, &c);
            xy0 = pxy;
        }

        Ok(Some(Self {
            img: patch_img,
            flat_origin,
            model_origin,
            flat_to_model,
        }))
    }
}
