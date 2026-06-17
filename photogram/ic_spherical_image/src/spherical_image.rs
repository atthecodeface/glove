use ic_base::{Point2D, Point3D, Triangle3D};
use std::marker::PhantomData;

trait Color: Default {}
trait ImagePatch<C>: Default {
    fn pixel_wh(&self) -> u32;
}

/// A pair of triangles (p[0], p[1], p[2]; p[1], p[2], p[3]) and the image data associated with them
pub struct TrianglePatchPair<C: Color, I: ImagePatch<C>> {
    /// The four coordinates *on the sphere* that the two triangles making up this patch correspond to
    sphere_corners: [Point3D; 4],
    /// The Plane consisting of points 0, 1, 2 - which should be
    /// counterclockwise as a surface when viewed from *outside* the solid
    plane012: Triangle3D,
    /// The Plane consisting of points 3, 1, 2 - which should be
    /// clockwise as a surface when viewed from *outside* the solid (i.e. the opposite orientation to plane012)
    plane312: Triangle3D,
    /// The image data for the square made up of the two triangles, of dimension pixel_wh by pixel_wh
    image_data: I,
    phantom: PhantomData<C>,
}

impl<C: Color, I: ImagePatch<C>> TrianglePatchPair<C, I> {
    fn new(sphere_corners: [Point3D; 4]) -> Option<Self> {
        let image_data = I::default();
        let plane012 =
            Triangle3D::of_points(&sphere_corners[0], &sphere_corners[1], &sphere_corners[2])?;

        let plane312 =
            Triangle3D::of_points(&sphere_corners[3], &sphere_corners[1], &sphere_corners[2])?;

        Some(Self {
            sphere_corners,
            plane012,
            plane312,
            image_data,
            phantom: PhantomData,
        })
    }
    fn coord_in_patch(&self, _pt: &Point3D) -> Point2D {
        Point2D::default()
    }
    fn contains_point(&self, pt: &Point3D) -> bool {
        if self.plane012.contains_point(pt) {
            true
        } else if self.plane312.contains_point(pt) {
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
pub enum QuadTreeImagePatch<C, I> {
    #[default]
    Empty,
    Uniform(C),
    Data(I),
}

pub struct SphericalImageOctahedron<C: Color, I: ImagePatch<C>> {
    patches: [TrianglePatchPair<C, I>; 4],
}

impl<C: Color, I: ImagePatch<C>> SphericalImageOctahedron<C, I> {
    fn new() -> Self {
        let patches = [
            TrianglePatchPair::new([
                [0.0, 0.0, 1.0].into(),
                [1.0, 0.0, 0.0].into(),
                [0.0, 1.0, 0.0].into(),
                [0.0, 0.0, -1.0].into(),
            ])
            .unwrap(),
            TrianglePatchPair::new([
                [0.0, 0.0, 1.0].into(),
                [0.0, 1.0, 0.0].into(),
                [-1.0, 0.0, 0.0].into(),
                [0.0, 0.0, -1.0].into(),
            ])
            .unwrap(),
            TrianglePatchPair::new([
                [0.0, 0.0, 1.0].into(),
                [-1.0, 0.0, 0.0].into(),
                [0.0, -1.0, 0.0].into(),
                [0.0, 0.0, -1.0].into(),
            ])
            .unwrap(),
            TrianglePatchPair::new([
                [0.0, 0.0, 1.0].into(),
                [0.0, -1.0, 0.0].into(),
                [1.0, 0.0, 0.0].into(),
                [0.0, 0.0, -1.0].into(),
            ])
            .unwrap(),
        ];
        Self { patches }
    }
    fn find_patch_of_point(&self, p: Point3D, ignore_mask: u32) -> Option<usize> {
        None
    }
    /// Iterate through the pixels with centers at x0, x1 with n pixels precisely; n must be 2 or more
    fn pixel_iter(x0: Point3D, x1: Point3D, n: u32) // -> Option<impl ExactSizeIterator<Item = C>> {
    {
    }
}
