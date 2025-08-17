//a Imports
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use serde::{Deserialize, Serialize};

//a Tag
//tp Tag
/// A [Tag] is a [String] that is either Owned or Shared
///
/// When created from Json or similar it is Owned; when it is resolved
/// into a TagSet it becomes Shared, and Shared references are used by
/// different data sets to refer to the same name. Owned tags should
/// not be part of the data structures after initialization has
/// completed.
///
/// A Tag (de)serializes to a string; it has to Deserialize to Owned. A
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
    pub fn xas_str(&self) -> &str {
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

//a Blah
//tp Blah
pub struct Blah<'a> {
    tags: &'a TagSet,
    index: usize,
    length: usize,
}

//ip Iterator for  Blah
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

//a TagMap
//tp TagData
pub trait TagData {
    fn tag(&self) -> &Tag;
    fn tag_mut(&mut self) -> &mut Tag;
}

//tp TagMap
#[derive(Debug)]
pub struct TagMap<V: TagData> {
    data: HashMap<Tag, Rc<V>>,
    tags: Rc<TagSet>,
}

//ip Default for TagMap
impl<V> std::default::Default for TagMap<V>
where
    V: TagData,
{
    fn default() -> Self {
        let data = HashMap::new();
        let tags = Rc::new(TagSet::default());
        Self { data, tags }
    }
}

//ip Serialize for TagMap
impl<V> Serialize for TagMap<V>
where
    V: TagData,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.data.len()))?;
        for tag in self.map_sorted_tags(|v| v) {
            seq.serialize_element(self.data.get(&tag).unwrap())?;
        }
        seq.end()
    }
}

//ip Deserialize for TagMap
impl<'de, V> Deserialize<'de> for TagMap<V>
where
    V: TagData,
    V: Deserialize<'de>,
{
    fn deserialize<DE>(deserializer: DE) -> std::result::Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let mut tag_map = TagMap::default();
        let array = Vec::<V>::deserialize(deserializer)?;
        for data in array {
            tag_map.add_data_unresolved(data);
        }
        Ok(tag_map)
    }
}

//ip TagMap
impl<V> TagMap<V>
where
    V: TagData,
{
    //mp set_tag_set
    /// Set the TagSet for this TagMap
    ///
    /// This turns all the Owned tags into Shared tags
    pub fn set_tag_set(&mut self, tags: Rc<TagSet>) {
        self.tags = tags;
        let old_data = std::mem::take(&mut self.data);
        for (t, v) in old_data.into_iter() {
            assert!(!t.is_resolved());
            let t = self.tags.resolve_tag(t);
            self.data.insert(t, v);
        }
    }

    //mp map_sorted_tags
    pub fn map_sorted_tags<F, T>(&self, map: F) -> T
    where
        F: FnOnce(Vec<Tag>) -> T,
    {
        let mut order: Vec<_> = self.data.keys().map(|s| s.clone()).collect();
        order.sort_by(|a, b| a.cmp(&b));
        map(order)
    }

    //mp has_tag
    pub fn has_tag(&self, t: &Tag) -> bool {
        self.data.contains_key(t)
    }

    //mp contains_data
    pub fn contains_data<A: AsRef<V>>(&self, v: A) -> bool {
        self.data.contains_key(v.as_ref().tag())
    }

    //mp add_data_unresolved
    /// Requires np to not be in the name set already
    pub fn add_data_unresolved(&mut self, mut data: V) -> Option<Rc<V>> {
        let tag = data.tag().clone();
        self.data.insert(tag, Rc::new(data))
    }

    //mp add_data
    /// Requires np to not be in the name set already
    pub fn add_data(&mut self, mut data: V) -> Option<Rc<V>> {
        data.tag_mut().resolve_in(&self.tags);
        let tag = data.tag().clone();
        self.data.insert(tag, Rc::new(data))
    }

    //mp get_data
    pub fn get_data(&self, name: &str) -> Option<Rc<V>> {
        self.data.get(name).cloned()
    }

    //mp iter
    pub fn iter(&self) -> impl Iterator<Item = &Rc<V>> {
        self.data.values()
    }

    //dp into_iter
    pub fn into_iter(self) -> impl Iterator<Item = Option<V>> {
        self.data.into_values().map(|v| Rc::into_inner(v))
    }
}
