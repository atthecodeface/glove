use crate::Image;

pub trait FromPatchFn {
    type Pixel;
    fn set_mapping(&mut self, patch_x: u32, patch_y: u32);
    fn map_from_patch(&mut self, patch_x: u32, patch_y: u32) -> Option<Self::Pixel>;
}

/// A (rectangular) patch dervived from an image, which contains the pixel data of the patch
///
/// This is a transient type used to fill the patch using a mapping function
///
/// OLD... from the original mapping patch
///
/// THe patch is derived from a portion of an original image of a 3D model; it
/// should correspond to a plane on that image (i.e. not a curved surface)).
///
/// There will be a point on the patch (the flat_origin) that maps to a point in
/// the model (the model_origin); there should also be a rotational mapping from
/// 2D points on the patch to points in the model.
///
/// The mapping from a point on the patch to the model of point Pp == (Ppx, PPy, 0) is:
///
///  FlatToModel(Pp - FlatOrigin) + ModelOrigin
///
pub struct ImagePatch<'a, I: Image> {
    img: &'a mut I,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    from_patch: Box<dyn FromPatchFn<Pixel = I::Pixel> + 'a>,
}

impl<'a, I: Image> ImagePatch<'a, I> {
    pub fn new<F: FromPatchFn<Pixel = I::Pixel> + 'a>(
        img: &'a mut I,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        from_patch: F,
    ) -> Self {
        Self {
            img,
            x,
            y,
            width,
            height,
            from_patch: Box::new(from_patch),
        }
    }

    pub fn img(&self) -> &I {
        self.img
    }

    pub fn img_mut(&mut self) -> &mut I {
        self.img
    }

    pub fn img_origin(&self) -> (u32, u32) {
        (self.x, self.y)
    }

    pub fn img_wh(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn fill_img(&mut self) {
        for x in 0..self.width {
            self.from_patch.set_mapping(x, 0);
            for y in 0..self.height {
                if let Some(c) = self.from_patch.map_from_patch(x, y) {
                    self.img.put(x + self.x, y + self.y, &c);
                }
            }
        }
    }

    /*
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
            let _p_sum = p_sum - (model_origin * (num_model_pts - 2) as f64);
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




            let mut xy0 = [0., 0.].into();
            let c: I::Pixel = 125_u8.into();
            for pxy in corners.iter() {
                let pxy = [pxy[0] - lx, pxy[1] - by].into();
                patch_img.draw_line(&xy0, &pxy, &c);
                xy0 = pxy;
            }

            let mut xy0 = [0., 0.].into();
            let c: I::Pixel = 255_u8.into();
            let model_pts_clone = model_pts.clone();
            for pxy in model_pts_clone.map(|p| model_to_flat(*p - model_origin) * px_per_model) {
                let pxy = [pxy[0] - lx, pxy[1] - by].into();
                patch_img.draw_line(&xy0, &pxy, &c);
                xy0 = pxy;

    */
}
