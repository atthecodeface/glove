use ic_base::JsonParsable;

use ic_spherical_image::{SphericalImageDescriptor, SphericalImageShape, SphericalPatchDescriptor};

#[test]
fn test_shape() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!(
        "{}",
        serde_json::to_string_pretty(&SphericalImageShape::Tetrahedron)?
    );
    // Parse known shapes correctly
    let _t = SphericalImageShape::load_json("\"Tetrahedron\"", &())?;
    let _t = SphericalImageShape::load_json("\"Octahedron\"", &())?;
    let _t = SphericalImageShape::load_json("\"Icosahedron\"", &())?;
    eprintln!("{_t:?}");
    // assert!(false, "Fore fail");
    Ok(())
}

#[test]
fn test_patch() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!(
        "{}",
        serde_json::to_string_pretty(&SphericalImageShape::Tetrahedron)?
    );
    // Parse known shapes correctly
    let sd = (SphericalImageShape::Tetrahedron).to_spherical_data()?;
    let t = SphericalPatchDescriptor::load_json(
        r##"{
        "patch_size":32,
        "img_xy": [0,0],
        "toplevel_t0": 0,
        "toplevel_t1": 0,
        "subdivision_to_patch": 0,
        "t0_subdivision_hierarchy": 0,
        "t1_subdivision_hierarchy": 0,
        "patch_subdivision": 0
        }"##,
        &sd,
    )?;
    eprintln!("{t:?}");
    // assert!(false, "Fore fail");
    Ok(())
}

#[test]
fn test_imagee_tetrahedron() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!(
        "{}",
        serde_json::to_string_pretty(&SphericalImageShape::Tetrahedron)?
    );
    // Parse known shapes correctly
    let sd = (SphericalImageShape::Tetrahedron).to_spherical_data()?;
    let t = SphericalImageDescriptor::load_json(
        r##"{
        "img_wh": [64,64],
        "shape": "Tetrahedron",
        "patches": [
        {
        "patch_size":32,
        "img_xy": [0,0],
        "toplevel_t0": 0,
        "toplevel_t1": 1,
        "subdivision_to_patch": 0,
        "t0_subdivision_hierarchy": 0,
        "t1_subdivision_hierarchy": 0,
        "patch_subdivision": 0
        },

        {
        "patch_size":32,
        "img_xy": [32,0],
        "toplevel_t0": 2,
        "toplevel_t1": 3,
        "subdivision_to_patch": 0,
        "t0_subdivision_hierarchy": 0,
        "t1_subdivision_hierarchy": 0,
        "patch_subdivision": 0
        }
        ]
        }
        "##,
        &(),
    )?;
    eprintln!("{t:?}");

    assert!(false, "Fore fail");
    Ok(())
}

#[test]
fn test_imagee_octahedron() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!(
        "{}",
        serde_json::to_string_pretty(&SphericalImageShape::Tetrahedron)?
    );
    // Parse known shapes correctly
    let sd = (SphericalImageShape::Tetrahedron).to_spherical_data()?;
    let t = SphericalImageDescriptor::load_json(
        r##"{
        "img_wh": [64,64],
        "shape": "Octahedron",
        "patches": [
        {
        "patch_size":32,
        "img_xy": [0,0],
        "toplevel_t0": 0,
        "toplevel_t1": 1,
        "subdivision_to_patch": 0,
        "t0_subdivision_hierarchy": 0,
        "t1_subdivision_hierarchy": 0,
        "patch_subdivision": 0
        },

        {
        "patch_size":32,
        "img_xy": [32,0],
        "toplevel_t0": 2,
        "toplevel_t1": 3,
        "subdivision_to_patch": 0,
        "t0_subdivision_hierarchy": 0,
        "t1_subdivision_hierarchy": 0,
        "patch_subdivision": 0
        }
        ]
        }
        "##,
        &(),
    )?;
    eprintln!("{t:?}");

    assert!(false, "Fore fail");
    Ok(())
}
