//a To do
//
// Add mapped line start/direction/error indication for each pms point
//
//
// From *nps* names, and a *pms* and *camera position* can generate rays with errors
//
// From multiple rays-with-errors can generate mode positions, errors and confidences
//
// A ray with error is a starting Point3D and a unit direction vector Point3D and a error tan ratio E
//
// The target area of the ray is a distance D from the starting point such that the error circle around the point of the target area has radius R such that E = R / D
//
// If the ray is generated from a picture without a model position then the error angle is perhaps more obvious
//

//a Documentation
/*! Documentation

  The calibration diagram is 4 points of X axis and 5 of Y axis with also point (1,1).

  Assume the diagram is in the Z=0 plane.

  A camera at P with quaternion Q will have camera-relative coordinates Q.(xy0+P) = xyz'

  This has a pitch/roll and hence view XY

  As a guess one has XY = fov_scale * xyz' / z' (This assumes a type of lens)

  We should have points (0,0), (0,1), (0,2), (0,3) ...

  These have coords
  xyz00' = Q.000+Q.P
  xyz01' = Q.010+Q.P = xyz00' + 1*Q.dx010
  xyz02' = Q.020+Q.P = xyz00' + 2*Q.dx010
  xyz03' = Q.030+Q.P = xyz00' + 3*Q.dx010

  Now if Q.dx010 = dx,dy,dz then we have
  XY00 = xyz00' * (scale/z00') hence xyz00' = XY00/(scale/z00')
  XY01 = ((XY00 / (scale/z00')) + (dx,dy)) * scale / (z00'+dz)
       = ((XY00 * z00' +   (dx,dy)*scale) / (z00'+dz)
  XY02 = ((XY00 * z00' + 2*(dx,dy)*scale) / (z00'+2*dz)
  XY03 = ((XY00 * z00' + 3*(dx,dy)*scale) / (z00'+3*dz)

  let z = z00' and (dx,dy)*scale=DXY and XY00=XY

  Hence:
  XY01-XY00 = ((XY * (z-z-dz) + dxysc) / (z+dz)
            = (DXY - dz * XY) / (z+dz)
  and
  XY03-XY02 = ((XY*z + 3DXY) / (z+3dz) - ((XY*z + 2DXY) / (z+2dz)
            = XY*z*(1/(z+3dz) - 1/(z+2dz)) + DXY*(3/(z+3dz)-2/(z+2dz))

  1/(z+3dz)-1/(z+2dz) = (z+2dz-z-3dz)/(z+3dz)/z+2dz) = -dz/(z+3dz)/z+2dz)
  3/(z+3dz)-2/(z+2dz) = (3z+6dz-2z-2dz)/(z+3dz)/z+2dz) = z/(z+3dz)/z+2dz)

  XY03-XY02 = ((XY*z + 3DXY) / (z+3dz) - ((XY*z + 2DXY) / (z+2dz)
            = (DXY-dz*XY) * z/(z+3dz)/(z+2dz)
  Now z/(z+3dz)/z+2dz) = z / (z**2 + 5z.dz + 6.dz**2)
  If dz<<z then this = 1 / (z + 5.dz)
  XY03-XY02 = (DXY-dz*XY) / (z+5dz)

  xyz00' = (z+0*dz) * (XY00,1) = P + 0*Q.dx010
  xyz01' = (z+1*dz) * (XY01,1) = P + 1*Q.dx010
  xyz02' = (z+2*dz) * (XY02,1) = P + 2*Q.dx010
  xyz03' = (z+3*dz) * (XY03,1) = P + 3*Q.dx010

  Q.dx010 = (z+3*dz) * (XY03,1) - (z+2*dz) * (XY02,1)

  To a first approximation this is

  Q.dx010 = (z+5/2*dz) * ((XY03,1)-(XY02,1))

C0, about 54cm from the origin on the screen (C1 is 46cm)

Y axis  (374.591667 300.550000 ) (374.120000 224.720000 ) (375.580000 156.230000 ) (375.598592 86.098592 ) (375.085366 21.048780 )
X axis  (231.333333 129.294118 ) (375.580000 156.230000 ) (504.053398 175.679612 ) (619.271084 195.301205 )

(54.591667   60.550000 ) (0,+76)
(54.120000  -15.280000 ) (0,+70)
(55.580000  -83.770000 ) (0,+70)
(55.598592 -153.910000 ) (0,+65)
(55.085366 -218.950000 )

(-89.67     -110.71 )
( 55.580000 -83.77 )
(184.053398 -64.32 )
(299.271084 -44.69 )

Another way to look at it is that each point on the calibration is on a line from the camera out.
i.e. xyz00' = k0 * Dir(XY00)
And we know that
xyz01' - xyz00' =   dxyz01 = k1 * Dir(XY01) - k0 * Dir(XY00)  (3 equations, 5 unknowns)
and
xyz02' - xyz00' = 2*dxyz01 = k2 * Dir(XY02) - k0 * Dir(XY00)  (6 equations, 6 unknowns)

If we assume that k0=1 then
xyz01' - xyz00' =   dxyz01 = k1 * Dir(XY01) - Dir(XY00)
xyz02' - xyz00' = 2*dxyz01 = k2 * Dir(XY02) - Dir(XY00)
xyz02' - xyz00' =   dxyz01 = k2/2 * Dir(XY02) - 1/2*Dir(XY00) = k1 * Dir(XY01) - Dir(XY00)
k2/2 * Dir(XY02) - k1 * Dir(XY01) = 1/2 Dir(XY00)

!*/

//a Modules
pub use ic_base::types::{Mat3x3, Point2D, Point3D, Point4D, Quat, RollYaw, TanXTanY};
pub use ic_base::utils::Rrc;
pub use ic_base::Ray;
pub use ic_base::{json, utils};

pub use ic_image as image;
pub use ic_image::{Color, Image, ImageGray16, ImageRgb8, Patch, Region};

pub use ic_camera::{
    CameraBody, CameraDatabase, CameraLens, CameraPolynomial, CameraPolynomialCalibrate,
    CameraPolynomialDesc,
};
// Rename these traits?
pub use ic_camera::CameraProjection;
// Don't expose this?
pub use ic_camera::polynomial;

pub use ic_mapping::BestMapping;
pub use ic_mapping::CameraShowMapping;
pub use ic_mapping::ModelLineSet;
pub use ic_mapping::{CameraAdjustMapping, CameraPtMapping};
pub use ic_mapping::{NamedPoint, NamedPointSet};
pub use ic_mapping::{PointMapping, PointMappingSet};

pub mod cmdline_args;

mod cip;
mod project;

pub use ic_kernel as kernel;
pub use ic_kernel::{Accelerate, KernelArgs, Kernels};

pub use cip::{Cip, CipDesc};
pub use project::Project;
// mod rotations;
// pub use rotations::Rotations;

pub mod http;
pub mod thread_pool;
