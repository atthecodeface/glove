use super::Point2D;

/// Trait for a projection
pub trait Projection {
    fn centre_xy(&self) -> Point2D {
        Point2D::default()
    }
    fn screen_size(&self) -> Point2D;
    /// Ratio of width to height
    fn aspect_ratio(&self) -> f64;
    /// FOV of the camera (for half camera view width) in radians
    fn tan_fov_x(&self) -> f64;
    /// FOV of the camera (for half camera view height) in radians
    fn tan_fov_y(&self) -> f64 {
        self.tan_fov_x() / self.aspect_ratio()
    }
    fn as_spherical_xy(&self, screen_xy: &Point2D) -> Point2D {
        let rel_xy = *screen_xy - self.centre_xy();
        let wh = self.screen_size();
        [
            rel_xy[0] / wh[0] / self.tan_fov_x(),
            -rel_xy[1] / wh[1] / self.tan_fov_y(),
        ]
        .into()
    }
}
