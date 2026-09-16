//! This is the toplevel library for photogram
//!
//! It exports everything (?!) that an external system may need...

pub use ic_base::{Tag, TagMap, TagSet};

pub use ic_base::WordXy;
pub use ic_base::{Error, Result, Rrc};
pub use ic_base::{JsonParsable, JsonSrc, PathGlob, PathSet};
pub use ic_base::{ModelData, NamedRayList, Ray};
pub use ic_base::{Point2D, Point3D, Quat, QuaternionDesc, RollYaw, TanXTanY};
pub use ic_base::{QtPath, Quadtree};

pub use ic_cache::{Cache, CacheRef, Cacheable};
pub use ic_camera::{CalibrationMapping, CameraProjection, CameraSensor};
pub use ic_camera::{
    CameraBody, CameraDatabase, CameraInstance, CameraInstanceDesc, CameraLens, LensPolys,
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
pub use ic_projections::{Cylinder, CylindricalProjection};
pub use ic_spherical_image::ImageFileIndex;
pub use ic_spherical_image::{SphericalImage, SphericalImageShape};
pub use ic_threads::ThreadPool;
pub use indexed::Idx;
