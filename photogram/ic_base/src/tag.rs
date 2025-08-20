//a Imports
use std::borrow::Borrow;
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
/// A Tag (de)serializes to a string; it has to Deserialize to Unresolved.
#[derive(Debug)]
pub enum Tag {
    /// Owned name, but not in the official TagSet as yet
    ///
    /// This state should only be valid for the tags in use by the
    /// eventual owner of the TagSet, prior to that being provided.
    ///
    /// This may be borrowed as a &String or &str, but not as an Rc;
    /// for this it should have been added to a TagSet
    Owned(Rc<String>),
    /// Name in a TagSet
    Shared(Rc<String>),
    /// An unresolved reference, from a type that does not own the TagSet
    ///
    /// This will resolved into a Shared when the type is provided
    /// with the TagSet to resolve it with
    ///
    /// An Unresolved can *only* be resolved; no other operations are permitted
    Unresolved(String),
}

//ip Borrow<str> for Tag
impl Borrow<str> for Tag {
    fn borrow(&self) -> &str {
        match self {
            Tag::Owned(s) => s,
            Tag::Shared(s) => s,
            Tag::Unresolved(s) => s,
        }
    }
}

//ip Borrow<String> for Tag
impl Borrow<String> for Tag {
    fn borrow(&self) -> &String {
        match self {
            Tag::Owned(s) => s,
            Tag::Shared(s) => s,
            Tag::Unresolved(s) => s,
        }
    }
}

//ip Borrow<Rc<String>> for Tag
impl Borrow<Rc<String>> for Tag {
    #[track_caller]
    fn borrow(&self) -> &Rc<String> {
        match self {
            Tag::Owned(_s) => {
                panic!("Should not be borrowing a tag which is owned");
            }
            Tag::Unresolved(_s) => {
                panic!("Must not be borrow an unresolved tag");
            }
            Tag::Shared(s) => s,
        }
    }
}

//ip Clone for Tag
impl std::clone::Clone for Tag {
    #[track_caller]
    fn clone(&self) -> Self {
        match self {
            Tag::Owned(_s) => {
                panic!("Should not be cloning a tag which is owned");
            }
            Tag::Unresolved(_s) => {
                panic!("Must not be cloning a tag which is unresolved");
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
            Tag::Unresolved(_s) => {
                panic!("Must not hash an unresolved tag");
            }
        }
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
        Ok(Tag::Owned(s.into()))
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
            Tag::Unresolved(s) => s.serialize(serializer),
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
            Tag::Unresolved(s) => s,
        }
    }
}

//ip PartialOrd for Tag
impl std::cmp::PartialOrd for Tag {
    fn partial_cmp(&self, other: &Tag) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

//ip Cmp for Tag
impl std::cmp::Ord for Tag {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Borrow::<str>::borrow(self).cmp(Borrow::<str>::borrow(other))
    }
}

//ip PartialEq for Tag
impl std::cmp::PartialEq for Tag {
    fn eq(&self, other: &Tag) -> bool {
        match (self, other) {
            (Tag::Shared(s), Tag::Shared(o)) => Rc::ptr_eq(s, o),
            (s, o) => Borrow::<str>::borrow(s) == Borrow::<str>::borrow(o),
        }
    }
}

//ip Eq for Tag
impl std::cmp::Eq for Tag {}

//ip Default for Tag
impl std::default::Default for Tag {
    fn default() -> Self {
        Tag::Unresolved("".into())
    }
}

//ip Tag
impl Tag {
    // Must only be used by TagSet
    #[track_caller]
    fn clone_allow_owned(&self) -> Self {
        match self {
            Tag::Unresolved(_s) => {
                panic!("Attempt to clone an Unresolved tag for the TagSet which should only see Owned/Shared");
            }
            Tag::Owned(s) => Tag::Owned(s.clone()),
            Tag::Shared(s) => Tag::Shared(s.clone()),
        }
    }

    pub fn reference<S: Into<String>>(name: S) -> Self {
        Tag::Unresolved(name.into())
    }

    pub fn owned<S: Into<String>>(name: S) -> Self {
        Tag::Owned(Rc::new(name.into()))
    }

    pub fn is_resolved(&self) -> bool {
        matches!(self, Tag::Shared(_))
    }
    pub fn take_name(self) -> Option<String> {
        match self {
            Tag::Owned(s) => Rc::into_inner(s),
            _ => None,
        }
    }
    pub fn resolve_in(&mut self, tag_set: &TagSet) {
        if let Tag::Owned(s) = self {
            *self = tag_set.get_tag(s);
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
    index: RefCell<HashMap<Tag, usize>>,
}

//ip TagSet
impl TagSet {
    //mi insert_name
    fn insert_name(&self, name: Rc<String>) -> Tag {
        let n = self.tags.borrow().len();
        self.tags.borrow_mut().push(name.clone());
        let tag = Tag::Shared(name);
        self.index.borrow_mut().insert(tag.clone(), n);
        tag
    }

    //mp resolve_tag
    pub fn resolve_tag(&self, tag: Tag) -> Option<Tag> {
        match tag {
            Tag::Unresolved(name) => self
                .index
                .borrow()
                .get(&name)
                .map(|index| Tag::Shared(self.tags.borrow()[*index].clone())),
            Tag::Owned(name) => {
                if let Some(index) = self.index.borrow().get(&name) {
                    Some(Tag::Shared(self.tags.borrow()[*index].clone()))
                } else {
                    Some(self.insert_name(name))
                }
            }
            tag => Some(tag),
        }
    }

    //mp get_tag
    pub fn get_tag(&self, name: &str) -> Tag {
        if let Some(index) = self.index.borrow().get(name) {
            Tag::Shared(self.tags.borrow()[*index].clone())
        } else {
            self.insert_name(name.to_owned().into())
        }
    }

    //zz All done
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
        for (t, mut v) in old_data.into_iter() {
            assert!(!t.is_resolved());
            let Some(t) = self.tags.resolve_tag(t) else {
                panic!("Attempt to set TagSet for a TagMap that has *Unresolved* tags; as these are tags in the TagMap, these tags should either be owned or shared");
            };
            *Rc::get_mut(&mut v).unwrap().tag_mut() = t.clone();
            self.data.insert(t, v);
        }
    }

    //mp map_sorted_tags
    pub fn map_sorted_tags<F, T>(&self, map: F) -> T
    where
        F: FnOnce(Vec<Tag>) -> T,
    {
        let mut order: Vec<_> = self.data.keys().cloned().collect();
        order.sort();
        map(order)
    }

    //mp has_tag
    pub fn has_tag(&self, t: &Tag) -> bool {
        self.data.contains_key(t)
    }

    //mp has_name
    pub fn has_name(&self, s: &str) -> bool {
        self.data.contains_key(s)
    }

    //mp add_data_unresolved
    /// Requires np to not be in the name set already
    #[track_caller]
    pub fn add_data_unresolved(&mut self, data: V) -> Option<Rc<V>> {
        let tag = data.tag().clone_allow_owned();
        self.data.insert(tag, Rc::new(data))
    }

    //mp add_data
    /// Requires np to not be in the name set already
    pub fn add_data(&mut self, mut data: V) -> Option<Rc<V>> {
        data.tag_mut().resolve_in(&self.tags);
        let tag = data.tag().clone();
        self.data.insert(tag, Rc::new(data))
    }

    //mp get_tag
    pub fn get_tag(&self, t: &Tag) -> Option<&Rc<V>> {
        self.data.get(t)
    }

    //mp get_data
    pub fn get_data(&self, name: &str) -> Option<&Rc<V>> {
        self.data.get(name)
    }

    //mp iter
    pub fn iter(&self) -> impl Iterator<Item = &Rc<V>> {
        self.data.values()
    }

    //dp into_values
    pub fn into_values(self) -> impl Iterator<Item = Rc<V>> {
        self.data.into_values()
    }
}
