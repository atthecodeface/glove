//a Imports
use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use regex::RegexBuilder;
use serde::{Deserialize, Serialize};

use crate::Result;

/// A [Tag] is a [String] that is either Owned, Shared, or Unresolved
///
/// When created from Json by reading a [TagSet] it is 'Owned'; when it is resolved
/// by the [TagSet] it becomes 'Shared', and 'Shared' references are used by
/// different data sets to refer to the same name. 'Owned' tags should
/// not be part of the data structures after initialization has
/// completed.
///
/// A [Tag] serializes to a string. In a [TagSet] it deserializes to 'Owned',
/// but in a [TagMap] (which is expecting to use tags from its related [TagSet])
/// it deserializes to 'Unresolved'. When the [TagMap] is ready, it is then
/// 'linked' to the [TagSet] to resolve the tags to 'Shared'
///
/// Note that in some circumstances there is an explicit 'owner' of the data for
/// a TagMap; in those cases the tags for that tag map can be 'Owned' instead of
/// 'Unresolved' on deserialization (as it is with NamedPointSets)
#[derive(Debug)]
pub enum Tag {
    /// Owned name, but not in the official [TagSet] as yet (this is usually post-deserialization of the [TagSet])
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
//
// Needed to retrieve a Tag from a HashMap by a str
impl Borrow<str> for Tag {
    fn borrow(&self) -> &str {
        match self {
            Tag::Owned(s) => s,
            Tag::Shared(s) => s,
            Tag::Unresolved(s) => s,
        }
    }
}

//ip Borrow<Rc<String>> for Tag
//
// Required for HashMap<Tag, > for index in TagSet and in TagMap by Rc<String>
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
// Can we lose this?
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
    /// Clone an Owned or Shared tag; panic if this is unresolved
    // Must only be used by TagSet
    #[track_caller]
    fn clone_allow_owned(&self) -> Self {
        match self {
            Tag::Unresolved(_s) => {
                panic!(
                    "Attempt to clone an Unresolved tag for the TagSet which should only see Owned/Shared"
                );
            }
            Tag::Owned(s) => Tag::Owned(s.clone()),
            Tag::Shared(s) => Tag::Shared(s.clone()),
        }
    }

    /// Clone an Owned or Shared tag; panic if this is unresolved
    fn clone_with_new_name<S: Into<String>>(&self, name: S) -> Self {
        match self {
            Tag::Owned(_s) => Tag::Owned(name.into().into()),
            Tag::Shared(_s) => Tag::Shared(name.into().into()),
            _ => {
                panic!("Attempt to clone a tag that ia not clonable");
            }
        }
    }

    /// Create an *unresolved* tag
    pub fn make_unresolved<S: Into<String>>(name: S) -> Self {
        Tag::Unresolved(name.into())
    }

    /// Create an *owned* tag
    pub fn owned<S: Into<String>>(name: S) -> Self {
        Tag::Owned(Rc::new(name.into()))
    }

    /// Determins if a tag is Shared (i.e, unresolved has been mapped to a
    /// Shared tag, or owned has been successfully moved into Shared)
    pub fn is_resolved(&self) -> bool {
        matches!(self, Tag::Shared(_))
    }

    /// Take the name out of the tag
    pub fn take_name(self) -> Option<String> {
        match self {
            Tag::Owned(s) => Rc::into_inner(s),
            _ => None,
        }
    }

    /// Clone the Rc<String>
    pub fn clone_rc_string(&self) -> Option<Rc<String>> {
        match self {
            Tag::Owned(s) => Some(s.clone()),
            Tag::Shared(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// If an *Owned* tag then resolve it by getting it from within the TagSet
    pub fn resolve_in(&mut self, tag_set: &TagSet) {
        if let Tag::Owned(s) = self {
            *self = tag_set.get_shared_or_make_owned_tag(s);
        }
    }

    /// Return the number of users of a *Shared* tag outside the [TagSet] - so the number of [TagMap] users, etc
    ///
    /// Usually this is used to determine that a tag may be deleted from a TagSet
    pub fn in_use_count(&self) -> Option<usize> {
        let Tag::Shared(s) = self else {
            return None;
        };
        Some(std::rc::Rc::strong_count(s))
    }
}

/// A [TagSet] contains an array of `Rc<String>` that are the actual tag
/// contents: shared strings that are used within the `Tag` and the index
///
/// It also contains an index which provides for finding the (offical source
/// string) for a given tag
///
/// Renaming a [Tag] requires it to be removed from the index, changing the
/// contents of its 'tags' entry in the [Vec], and then reinserting it with the
/// new hash into the index.
///
/// Adding a new [Tag] (derived from a string) to the [TagSet] requires creating
/// the `Rc<String>` for the tag, adding it to the [Vec], and then inserting a
/// new entry into the index.
///
/// Removing a [Tag] requires removing it from the index, removing it from the
/// [Vec], and the reindexing.
///
/// The [TagSet] has interior mutability, as it will be in use by many
/// structures (the whole point is to share the [Tag] with named points, point
/// mappings, etc). It is useful to be able to iterate over the entries, and a
/// [TagSetIter] can be generated which holds a [Ref] of the data while is is
/// alive.
///
/// A [TagSet] serializes to a list of strings, and deserializes to a list of
/// Owned tags; before real use the tags must be converted to Owned tags, and if
/// a 'user' is referencing the [TagSet] then their 'uses' have to be resolved
/// (i.e. turned from 'Unresolved')
#[derive(Debug, Default)]
pub struct TagSet {
    /// The *shared* names
    tags: RefCell<Vec<Rc<String>>>,

    /// Mapping from text to index into tags
    index: RefCell<HashMap<Tag, usize>>,
}

struct TagSetIter<'a> {
    x: Ref<'a, Vec<Rc<String>>>,
    n: usize,
    index: usize,
}

impl<'a> std::iter::Iterator for TagSetIter<'a> {
    type Item = Rc<String>;
    fn next(&mut self) -> Option<Rc<String>> {
        if self.index < self.n {
            let i = self.index;
            self.index += 1;
            Some(self.x[i].clone())
        } else {
            None
        }
    }
}

//ip TagSet
impl TagSet {
    /// Invoked only by [TagMap] and [TagSet], this inserts a new name into the [TagSet] as a `Shared` tag
    fn insert_name(&self, name: Rc<String>) -> Tag {
        let n = self.tags.borrow().len();
        self.tags.borrow_mut().push(name.clone());
        let tag = Tag::Shared(name);
        self.index.borrow_mut().insert(tag.clone(), n);
        tag
    }

    /// DEPRECATED
    ///
    /// Invoked when a [TagMap] is set to be associated with a specific [TagSet]
    /// - which might have been read from JSON
    ///
    /// This maps Unresolved tags to Shared tag that is already in the TagSet, or None
    ///
    /// It maps Owned tags to Shared tags if that is already in the TagSet, else
    /// it adds it to the TagSet and returns a Shared version of the tag
    pub fn resolve_tag(&self, tag: Tag) -> Option<Tag> {
        match tag {
            Tag::Unresolved(name) => self
                .index
                .borrow()
                .get(name.as_str())
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

    /// Invoked when a [TagMap] is set to be associated with a specific [TagSet]
    /// - which might have been read from JSON; indeed this merges a full set
    /// into an empty set
    ///
    /// This maps Unresolved tags to Shared tag that is already in the TagSet, or
    /// to Owned
    ///
    /// It maps Owned tags to Shared tags if that is already in the TagSet, else
    /// it adds it to the TagSet and returns a Shared version of the tag
    pub fn resolve_name(&self, name: &str) -> Tag {
        if let Some(index) = self.index.borrow().get(name) {
            Tag::Shared(self.tags.borrow()[*index].clone())
        } else {
            Tag::Shared(name.to_owned().into())
        }
    }

    /// Get the tag corresponding to the name, or create a new tag; the result *will* be Shared
    pub fn get_shared_or_make_owned_tag(&self, name: &str) -> Tag {
        if let Some(index) = self.index.borrow().get(name) {
            Tag::Shared(self.tags.borrow()[*index].clone())
        } else {
            self.insert_name(name.to_owned().into())
        }
    }

    //mp iter
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Rc<String>> + 'a {
        let n = self.tags.borrow().len();
        TagSetIter {
            x: self.tags.borrow(),
            n,
            index: 0,
        }
    }

    /// Rename the tag (which must be Shared in the set) to a new Shared Tag
    pub fn remove_tag(&self, name: &str) -> Option<Tag> {
        if let Some((old_tag, idx)) = self.index.borrow_mut().remove_entry(name) {
            self.tags.borrow_mut()[idx] = "<deleted>".to_string().into();
            Some(old_tag)
        } else {
            None
        }
    }

    fn has_name(&self, name: &str) -> bool {
        self.index.borrow().contains_key(name)
    }

    /// Rename the tag (which must be Shared in the set) to a new Shared Tag
    pub fn rename_tag(&self, old_name: &str, name: &str) -> Option<(Tag, Tag)> {
        if self.has_name(name) {
            return None;
        }

        if let Some((old_tag, idx)) = self.index.borrow_mut().remove_entry(old_name) {
            let new_tag = old_tag.clone_with_new_name(name);
            self.tags.borrow_mut()[idx] = new_tag.clone_rc_string().unwrap();
            self.index.borrow_mut().insert(new_tag.clone(), idx);
            Some((old_tag, new_tag))
        } else {
            None
        }
    }
    //zz All done
}

//a TagMap
//tp TagData
pub trait TagData {
    fn tag(&self) -> &RefCell<Tag>;
}

/// A [TagMap] extends a (shared) [TagSet] to provide a map from a copy of the string of a 'Tag' to the actual Tag data
///
/// It is not permitted to access the private [Vec] of tags within the TagSet
///
/// A [TagMap] serializes its tags into strings; they deserialize into
/// *Unresolved* tags, which require resolution with the [TagSet] (seperately
/// deserialized) the the [TagMap] should contain tags from.
///
/// A tag can be renamed only by:
///
///   1.  *removing* the string from the map with its data
///   2.  updating the data with the new tag
///   3.  reinserting the new data
#[derive(Debug)]
pub struct TagMap<V: TagData> {
    data: HashMap<String, Rc<V>>,
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
            seq.serialize_element(self.data.get(tag.as_str()).unwrap())?;
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
            tag_map.add_data_owned(data);
        }
        Ok(tag_map)
    }
}

//ip TagMap
impl<V> TagMap<V>
where
    V: TagData,
{
    //ap len
    pub fn len(&self) -> usize {
        self.data.len()
    }

    //ap is_empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Set the TagSet for this TagMap
    ///
    /// This turns all the Unresolved tags into Shared tags from the [TagSet], assuming that the TagMap is only using known tags.
    pub fn set_tag_set(&mut self, tags: Rc<TagSet>) {
        self.tags = tags;
        let old_data = std::mem::take(&mut self.data);
        for (t, mut v) in old_data.into_iter() {
            // assert!(!t.is_resolved());
            let t = self.tags.resolve_name(&t);
            let x = Rc::get_mut(&mut v).unwrap().tag();
            *x.borrow_mut() = t.clone();
            self.data.insert(t.as_str().into(), v);
        }
    }

    //mp map_sorted_tags
    pub fn map_sorted_tags<F, T>(&self, map: F) -> T
    where
        F: FnOnce(Vec<String>) -> T,
    {
        let mut order: Vec<_> = self.data.keys().cloned().collect();
        order.sort();
        map(order)
    }

    /// Return true if the name is in the TagMap
    pub fn has_name(&self, s: &str) -> bool {
        self.data.contains_key(&s.to_owned())
    }

    /// Add the data to the TagMap with an 'Unresolved' tag
    #[track_caller]
    pub fn add_data_owned(&mut self, data: V) -> Option<Rc<V>> {
        let tag = data.tag().borrow().as_str().into();
        self.data.insert(tag, Rc::new(data))
    }

    //mp add_data
    /// Requires np to not be in the name set already
    pub fn add_data(&mut self, data: V) -> Option<Rc<V>> {
        data.tag().borrow_mut().resolve_in(&self.tags);
        let tag = data.tag().borrow().as_str().into();
        self.data.insert(tag, Rc::new(data))
    }

    pub fn get_tag<'a, 'b>(&'a self, tag: &'b Tag) -> Option<&'a Rc<V>> {
        self.data.get(tag.as_str())
    }

    pub fn get_data(&self, name: &str) -> Option<&Rc<V>> {
        self.data.get(name)
    }

    pub fn get_tag_use_count(&self, name: &str) -> Option<usize> {
        if let Some(count) = self
            .data
            .get(name)
            .and_then(|v| v.tag().borrow().in_use_count())
        {
            debug_assert!(
                count >= 4,
                "tag must be in use by 'v', TagMap Data, TagSet vec, and TagSet index"
            );
            Some(count - 4)
        } else {
            None
        }
    }

    pub fn remove_data(&mut self, name: &str) -> Option<Rc<V>> {
        self.data.remove(name)
    }

    fn is_regex(s: &str) -> bool {
        s.chars().any(|c| "^[*?".contains(c))
    }

    //mp fold_search
    pub fn fold_search<F, T>(
        &self,
        search: &str,
        case_insensitive: bool,
        mut acc: T,
        fold: F,
    ) -> Result<T>
    where
        F: Fn(T, &Rc<V>) -> T,
    {
        if Self::is_regex(search) {
            let regex = RegexBuilder::new(search)
                .case_insensitive(case_insensitive)
                .build()
                .map_err(|e| format!("failed to compile regex '{search}': {e}"))?;
            for t in self.tags.iter() {
                if regex.is_match(t.as_str()) {
                    acc = fold(acc, self.data.get(t.as_str()).unwrap());
                }
            }
        } else {
            if let Some(data) = self.get_data(search) {
                acc = fold(acc, data);
            } else {
                return Err(format!("Could not find named point {search} in the set").into());
            };
        }
        Ok(acc)
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
