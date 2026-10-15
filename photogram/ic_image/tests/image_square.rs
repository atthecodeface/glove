use ic_image::{ImageRgb8, ImageSquareSet};

#[test]
fn test_image_square_0() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let image = ImageRgb8::new(10 * 8, 20 * 8);
    let mut set = ImageSquareSet::create(image, 8)?;
    for _ in 0..199 {
        let x = set.allocate_squares(8, 8);
        assert!(x.is_some());
        let x = x.unwrap();
        eprintln!("{x:?}");
    }
    let x = set.allocate_squares(16, 16);
    eprintln!("{x:?}");
    assert!(x.is_none());
    let x = set.allocate_squares(8, 8);
    eprintln!("{x:?}");
    assert!(x.is_some());
    let x = set.allocate_squares(8, 8);
    eprintln!("{x:?}");
    assert!(x.is_none());
    Ok(())
}

#[test]
fn test_image_square_1() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let image = ImageRgb8::new(10 * 8, 20 * 8);
    let mut set = ImageSquareSet::create(image, 8)?;
    for _ in 0..50 {
        let x = set.allocate_squares(16, 16);
        assert!(x.is_some());
        let x = x.unwrap();
        eprintln!("{x:?}");
    }
    let x = set.allocate_squares(16, 16);
    eprintln!("{x:?}");
    assert!(x.is_none());
    let x = set.allocate_squares(8, 8);
    eprintln!("{x:?}");
    assert!(x.is_none());
    Ok(())
}

#[test]
fn test_image_square_2() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let image = ImageRgb8::new(10 * 8, 20 * 8);

    let mut set = ImageSquareSet::create(image, 8)?;
    let first = set.allocate_squares(16, 16).unwrap();
    for _ in 0..49 {
        let _x = set.allocate_squares(16, 16);
    }
    assert!(set.allocate_squares(8, 8).is_none());
    set.free_squares(first);
    assert!(set.allocate_squares(8, 24).is_none());
    assert!(set.allocate_squares(24, 8).is_none());
    let x = set.allocate_squares(8, 16);
    assert!(x.is_some());
    let x = x.unwrap();
    assert!(set.allocate_squares(16, 8).is_none());
    let y = set.allocate_squares(8, 8).unwrap();
    let _z = set.allocate_squares(8, 8).unwrap();
    set.free_squares(x);
    assert!(set.allocate_squares(16, 8).is_none());
    set.free_squares(y);
    assert!(set.allocate_squares(16, 8).is_some());
    Ok(())
}
