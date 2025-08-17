//a Imports
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use serde::{Deserialize, Serialize};

//a Tag
//tp Tag
#[derive(Debug)]
pub enum Tag {
    Owned(String),
    Shared(Rc<String>),
}

//ip Borrow<str> for Tag
impl std::borrow::Borrow<str> for Tag {
    fn borrow(&self) -> &str {
        match self {
            Tag::Owned(s) => &s,
            Tag::Shared(s) => &s,
        }
    }
}

//ip Borrow<String> for Tag
impl std::borrow::Borrow<String> for Tag {
    fn borrow(&self) -> &String {
        match self {
            Tag::Owned(s) => s,
            Tag::Shared(s) => s,
        }
    }
}

//ip Borrow<Rc<String>> for Tag
impl std::borrow::Borrow<Rc<String>> for Tag {
    fn borrow(&self) -> &Rc<String> {
        match self {
            Tag::Owned(s) => {
                panic!("Should not be borrowing a tag which is onwed");
            }
            Tag::Shared(s) => s,
        }
    }
}

//ip Clone for Tag
impl std::clone::Clone for Tag {
    fn clone(&self) -> Self {
        match self {
            Tag::Owned(s) => {
                panic!("Should not be cloning a tag which is onwed");
            }
            Tag::Shared(s) => Tag::Shared(s.clone()),
        }
    }
}

//ip Hash for Tag
impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Tag::Owned(s) => {
                (**s).hash(state);
            }
            Tag::Shared(s) => {
                (**s).hash(state);
            }
        }
    }
}

//ip From<String> for Tag
impl std::convert::From<String> for Tag {
    fn from(s: String) -> Self {
        Tag::Owned(s)
    }
}

//ip From<&str> for Tag
impl std::convert::From<&str> for Tag {
    fn from(s: &str) -> Self {
        Tag::Owned(s.to_owned())
    }
}

//ip From<&String> for Tag
impl std::convert::From<&String> for Tag {
    fn from(s: &String) -> Self {
        Tag::Owned(s.clone())
    }
}

//ip Display for Tag
impl std::fmt::Display for Tag {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.as_str())
    }
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

//ip PartialOrd for Tag
impl std::cmp::PartialOrd for Tag {
    fn partial_cmp(&self, other: &Tag) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Tag::Owned(s), Tag::Owned(o)) => (*s).partial_cmp(o),
            (Tag::Owned(s), Tag::Shared(o)) => s.partial_cmp(o),
            (Tag::Shared(s), Tag::Owned(o)) => (**s).partial_cmp(o),
            (Tag::Shared(s), Tag::Shared(o)) => s.partial_cmp(o),
        }
    }
}

//ip Cmp for Tag
impl std::cmp::Ord for Tag {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
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

//ip Eq for Tag
impl std::cmp::Eq for Tag {}

//ip Tag
impl Tag {
    pub fn as_str(&self) -> &str {
        use std::borrow::Borrow;
        self.borrow()
    }
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
    pub fn resolve_in(&mut self, tag_set: &TagSet) {
        match self {
            Tag::Owned(s) => {
                *self = tag_set.get_tag(&s);
            }
            _ => {}
        }
    }
}

//a TagSet
//tp TagSet
#[derive(Debug, Default)]
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

    //mp get_tag
    pub fn get_tag(&self, name: &str) -> Tag {
        if let Some(index) = self.index.borrow().get(name) {
            Tag::Shared(self.tags.borrow()[*index].clone())
        } else {
            self.insert_name(name.to_owned())
        }
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = Rc<String>> + 'a {
        Blah {
            tags: self,
            index: 0,
            length: self.tags.borrow().len(),
        }
    }

    //zz All done
}
pub struct Blah<'a> {
    tags: &'a TagSet,
    index: usize,
    length: usize,
}
impl<'a> std::iter::Iterator for Blah<'a> {
    type Item = Rc<String>;
    fn next(&mut self) -> Option<Rc<String>> {
        if self.index >= self.length {
            None
        } else {
            let n = self.index;
            let r = self.tags.tags.borrow()[n].clone();
            self.index += 1;
            Some(r)
        }
    }
}
