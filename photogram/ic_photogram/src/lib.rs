/*! This is the toplevel library for photogram

It exports everything (?!) that an external system may need...

This libary internally has the following structure:

* ic_base

* ic_camera
* ic_image
* ic_mesh
* ic_kernel
* ic_cache
* ic_projections

* ic_mapping

* ic_spherical_image
* ic_project
* ic_http

*/

pub use ic_base::{Tag, TagMap, TagSet};

pub use ic_base::WordXy;
pub use ic_base::{Error, Result, Rrc};
pub use ic_base::{JsonParsable, JsonSrc, PathGlob, PathSet};
pub use ic_base::{ModelData, NamedRayList, Ray};
pub use ic_base::{Point2D, Point3D, Quat, QuaternionDesc, RollYaw, TanXTanY};
pub use ic_base::{QtPath, Quadtree};

pub use ic_cache::{Cache, CacheRef, Cacheable};
// pub use ic_camera::CalibrationMapping;
pub use ic_camera::{
    AdjustableCameraProjection, CameraLensProjection, CameraProjection, CameraSensor,
    CylindricalProjection, LensProjection,
};
pub use ic_camera::{
    BaseCamera, CameraBody, CameraDatabase, CameraInstance, CameraInstanceDesc, CameraLens,
    CylindricalLens, LensPolys, RectilinearLens, SizedSensor,
};
pub use ic_http::{
    HttpRequest, HttpRequestType, HttpResponse, HttpResponseType, HttpServer, HttpServerExt,
};

pub use ic_image::{
    Color8, Gray16, Image, ImageColor, ImageDrawable, ImageGray16, ImagePt, ImageRgb8,
    ImageSquareSet, Region, read_image,
};
pub use ic_kernel::*;
pub use ic_mapping::*;
pub use ic_mesh::Mesh;
pub use ic_project::*;
pub use ic_spherical_image::ImageFileIndex;
pub use ic_spherical_image::{SphericalImage, SphericalImageShape};
pub use ic_threads::ThreadPool;
pub use indexed::Idx;
