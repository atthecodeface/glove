//a Imports
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

//a Tag
//tp Tag
#[derive(Debug, Clone)]
pub enum Tag {
    Owned(String),
    Shared(Rc<String>),
}

//ip Deserialize for Tag
impl<'de> Deserialize<'de> for Tag {
    fn deserialize<DE>(deserializer: DE) -> std::result::Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Tag::Owned(s))
    }
}

//ip Serialize for Tag
impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Tag::Owned(s) => s.serialize(serializer),
            Tag::Shared(s) => s.serialize(serializer),
        }
    }
}

//ip Deref for Tag
impl std::ops::Deref for Tag {
    type Target = String;
    fn deref(&self) -> &String {
        match self {
            Tag::Owned(s) => s,
            Tag::Shared(s) => s,
        }
    }
}

//ip PartialEq for Tag
impl std::cmp::PartialEq for Tag {
    fn eq(&self, other: &Tag) -> bool {
        match (self, other) {
            (Tag::Owned(s), Tag::Owned(o)) => s == o,
            (Tag::Owned(s), Tag::Shared(o)) => s == &**o,
            (Tag::Shared(s), Tag::Owned(o)) => &**s == o,
            (Tag::Shared(s), Tag::Shared(o)) => Rc::ptr_eq(s, o),
        }
    }
}

//ip Tag
impl Tag {
    pub fn is_resolved(&self) -> bool {
        match self {
            Tag::Shared(_) => true,
            _ => false,
        }
    }
    pub fn take_name(self) -> Option<String> {
        match self {
            Tag::Owned(s) => Some(s),
            _ => None,
        }
    }
}

//a TagSet
//tp TagSet
pub struct TagSet {
    /// The *shared* names
    tags: RefCell<Vec<Rc<String>>>,
    /// Mapping from text to index into tags
    index: RefCell<HashMap<String, usize>>,
}

//ip TagSet
impl TagSet {
    //mi insert_name
    fn insert_name(&self, name: String) -> Tag {
        let shared_name = Rc::new(name.clone());
        let n = self.tags.borrow().len();
        self.tags.borrow_mut().push(shared_name.clone());
        self.index.borrow_mut().insert(name, n);
        Tag::Shared(shared_name)
    }

    //mp resolve_tag
    pub fn resolve_tag(&self, tag: Tag) -> Tag {
        if tag.is_resolved() {
            tag
        } else {
            let name = tag.take_name().unwrap();
            if let Some(index) = self.index.borrow().get(&name) {
                Tag::Shared(self.tags.borrow()[*index].clone())
            } else {
                self.insert_name(name)
            }
        }
    }

    //zz All done
}
