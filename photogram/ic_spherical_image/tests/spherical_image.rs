use geo_nd::Vector;
use ic_base::{GCTriangle, Point3D};
use ic_image::ImageGray16;
use ic_spherical_image::{
    ImagePatch, SphericalImage, SphericalImageDescriptor, SphericalImageShape,
    SphericalPatchDescriptor,
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

#[test]
fn test_sph_image_tetra() -> Result<(), Box<dyn std::error::Error>> {
    let mut image = SphericalImage::<ImageGray16>::of_shape(SphericalImageShape::Octahedron);
    let image_file = image.add_new_image(512, 512);
    image.add_toplevel_patches(image_file, 256, 0)?;
    let ps: Vec<_> = image.iter_patch_indices().collect();
    fn pix_map(v: Point3D) -> Option<u16> {
        let d = (v[0] * v[0] + v[1] * v[1]).sqrt();
        let d = (d.atan2(v[2]) * 5.0).cos();
        Some(((d * 32000.0) + 32768.0) as u16)
    }
    for p in ps {
        image.fill_image_patch(p, &pix_map);
    }

    let mut t = std::env::temp_dir();
    t.push("tmp.png");
    eprintln!("{t:?}");
    image.set_image_path(image_file, &t);
    image.write_image(image_file)?;
    assert!(false, "Force fail");
    Ok(())
}
