use ic_base::WordXy;

//a Tests
#[test]
fn test_word_xy_0() -> Result<(), Box<dyn std::error::Error>> {
    for (test_xy, xy_value) in [
        ("adult.agent", (0_u8, 0_u8)),
        ("audio.album", (1_u8, 1_u8)),
        ("black.angle", (2_u8, 2_u8)),
    ] {
        eprintln!("Test {test_xy}");
        let xy: WordXy<100, 5> = test_xy.parse()?;
        assert_eq!(test_xy, &xy.to_string());
        assert_eq!(xy_value, xy.into());
    }
    Ok(())
}

/// Check the linear mapping is distinct
#[test]
fn test_word_xy_1() -> Result<(), Box<dyn std::error::Error>> {
    let mut words = vec![];
    for i in 0..100 * 100 {
        let wxy = WordXy::<100, 5>::of_n(i);
        words.push(wxy);
    }
    words.sort();
    for wp in words.windows(2) {
        assert!(wp[0] != wp[1], "The first N*N must be distinct {wp:?}");
    }
    let wxy = WordXy::<100, 5>::of_n(100 * 100);
    assert!(words.contains(&wxy));
    Ok(())
}
