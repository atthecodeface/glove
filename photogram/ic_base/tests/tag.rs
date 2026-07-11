use ic_base::{Tag, TagData, TagMap};

struct Blah {
    tag: Tag,
    n: String,
}
impl TagData for Blah {
    fn tag(&self) -> &Tag {
        &self.tag
    }
    fn tag_mut(&mut self) -> &mut Tag {
        &mut self.tag
    }
}
#[test]
fn test_tagset() -> Result<(), Box<dyn std::error::Error>> {
    let mut tm = TagMap::default();
    tm.add_data(Blah {
        tag: Tag::owned("t0"),
        n: "data_t0".into(),
    });
    tm.add_data(Blah {
        tag: Tag::owned("t1"),
        n: "data_t1".into(),
    });

    let t = tm.get_data("t0").expect("Must be able to find tag t0");
    assert_eq!(t.n, "data_t0");
    // Tag use count is TagSet vec, TagSet index, TagMap data, and 't'.
    assert_eq!(t.tag.in_use_count(), Some(4));
    let _ = t;
    assert_eq!(tm.get_tag_use_count("t0"), Some(0));

    let t = tm.get_data("t0").unwrap().tag.clone();
    assert_eq!(tm.get_tag_use_count("t0"), Some(1));
    eprintln!("Cloned tag {t}");

    assert!(tm.remove_data("t0").is_some());
    assert!(tm.get_data("t0").is_none());

    Ok(())
}
