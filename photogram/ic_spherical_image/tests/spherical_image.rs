use geo_nd::Vector;
use ic_base::{GCTriangle, Point3D};
use ic_spherical_image::{
    ImagePatch, SphericalImageDescriptor, SphericalImageShape, SphericalPatchDescriptor,
};

#[test]
fn test_sph_image_patch() -> Result<(), Box<dyn std::error::Error>> {
    /*
       let p0: Point3D = [1.0, 2.0, 3.0].into();
       let p1: Point3D = [3.0, 1.0, -3.0].into();
       let p2: Point3D = [4.0, 0.0, 1.0].into();
       let p3: Point3D = [-2.0, 5.0, 1.0].into();

       let gct0 = GCTriangle::of_points(&p0, &p1, &p2);
       let gct1 = GCTriangle::of_points(&p0, &p2, &p3);

       let p = ImagePatch::of_gc_triangles(&gct0, &gct1);
    */
    // assert!(false, "Force fail");
    Ok(())
}

#[test]
fn test_sph_image_desc() -> Result<(), Box<dyn std::error::Error>> {
    let p0 =
        SphericalImageDescriptor::of_shape_toplevel(SphericalImageShape::Tetrahedron, (64, 32), 32);
    Ok(())
}
