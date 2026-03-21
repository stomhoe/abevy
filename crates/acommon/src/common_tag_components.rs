

use bevy::prelude::*;
#[allow(unused_imports)]

use bevy::platform::collections::HashSet;
use std::hash::{Hash, };
use crate::common_components::{HashId, Tag};
use std::fmt::{Debug, };
use serde::{Deserialize, Serialize};
use delegate::delegate;

macro_rules! impl_tags_common_methods {
    ($collection_type_name:ty, $tag_type:ty, $collection_kind:ident) => {
        impl $collection_type_name {
            pub fn new_error_if_set_empty<S: AsRef<str>>(tags: impl IntoIterator<Item = S>) -> Result<Self, ()> {
                let collection: $collection_kind<$tag_type> = tags.into_iter()
                .filter_map(|id| {
                    let id_str = id.as_ref().trim();
                    if id_str.is_empty() { None } else { Some(<$tag_type>::from(id_str)) }

                })
                .collect();
                if collection.is_empty() {
                    Err(())
                } else {
                    Ok(Self(collection))
                }
            }
            pub fn new<S: AsRef<str>>(tags: impl IntoIterator<Item = S>) -> Self {
                let collection: $collection_kind<$tag_type> = tags.into_iter()
                .filter_map(|id| {
                    let id_str = id.as_ref().trim();
                    if id_str.is_empty() { None } else { Some(<$tag_type>::from(id_str)) }

                })
                .collect();
                Self(collection)
            }
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
            pub fn len(&self) -> usize {
                self.0.len()
            }
            pub fn iter(&self) -> impl Iterator<Item = &$tag_type> {
                self.0.iter()
            }
            pub fn intersects(&self, other: &$collection_type_name) -> bool {
                for tag in &self.0 {
                    if other.0.iter().any(|t| t == tag) {
                        return true;
                    }
                }
                false
            }
            pub fn clear(&mut self) {
                self.0.clear();
            }
            pub fn extend_from(&mut self, other: &Self)
            where
                $tag_type: Clone,
            {
                self.0.reserve(other.0.len());
                for tag in other.iter() {
                    self.insert(tag.clone());
                }
            }
        }
    };
}
macro_rules! impl_tag_vec_methods {
    ($collection_type_name:ty, $tag_type:ty) => {
        impl $collection_type_name {
            pub fn contains(&self, tag: impl Into<$tag_type>) -> bool {
                let tag = tag.into();
                self.0.iter().any(|t| t == &tag)
            }
            pub fn insert(&mut self, tag: impl Into<$tag_type>) {
                let tag = tag.into();
                if !self.0.iter().any(|t| t == &tag) {
                    self.0.push(tag);
                }
            }
            pub fn remove(&mut self, tag: impl Into<$tag_type>) {
                let tag = tag.into();
                self.0.retain(|t| t != &tag);
            }
        }
        impl_tags_common_methods!($collection_type_name, $tag_type, Vec);
    };
}
macro_rules! impl_tag_hashset_methods {
    ($collection_type_name:ty, $tag_type:ty) => {
        impl $collection_type_name {
            pub fn contains(&self, tag: impl Into<$tag_type>) -> bool {
                self.0.contains(&tag.into())
            }
            pub fn insert(&mut self, tag: impl Into<$tag_type>) -> bool {
                self.0.insert(tag.into())
            }
            pub fn remove(&mut self, tag: impl Into<$tag_type>) -> bool {
                self.0.remove(&tag.into())
            }
        }
        impl_tags_common_methods!($collection_type_name, $tag_type, HashSet);
    };
}
macro_rules! define_tag_hashset_and_impl_methods {
    ($name:ident, $tag_type:ty) => {
        #[derive(Component, Deserialize, Serialize, Clone, Default, PartialEq, Eq, TypePath, Asset)]
        pub struct $name(pub HashSet<$tag_type>);
        impl Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_list().entries(self.0.iter()).finish()
            }
        }
        impl_tag_hashset_methods!($name, $tag_type);
    };
}
#[allow(unused_macros)]
macro_rules! define_tag_vec_and_impl_methods {
    ($name:ident, $tag_type:ty) => {
        #[derive(Component, Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
        pub struct $name(pub Vec<$tag_type>);
        impl_tag_vec_methods!($name, $tag_type);
    };
}

define_tag_hashset_and_impl_methods!(TagSet, Tag);

impl IntoIterator for TagSet {
    type Item = Tag;
    type IntoIter = <bevy::platform::collections::HashSet<Tag> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a TagSet {
    type Item = &'a Tag;
    type IntoIter = <&'a bevy::platform::collections::HashSet<Tag> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

#[derive(Component, Debug, Default, Copy, Clone)]
#[require(HashedTagsVec)]
pub struct AddSameHashedTags;

#[derive(Component, Debug, Default, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct HashedTagsVec(pub Vec<HashId>);
impl_tag_vec_methods!(HashedTagsVec, HashId);

impl From<&TagSet> for HashedTagsVec {
    fn from(tag: &TagSet) -> Self {
        Self(tag.0.iter().map(|t| HashId::from(t)).collect())
    }
}

impl IntoIterator for HashedTagsVec {
    type Item = HashId;
    type IntoIter = std::vec::IntoIter<HashId>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a HashedTagsVec {
    type Item = &'a HashId;
    type IntoIter = std::slice::Iter<'a, HashId>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WhitelistedTags(pub TagSet);

impl WhitelistedTags {
    pub fn new<S: AsRef<str>>(tags: impl IntoIterator<Item = S>) -> Self {
        Self(TagSet::new(tags))
    }
    delegate! {
        to self.0 {
            pub fn is_empty(&self) -> bool;
            pub fn len(&self) -> usize;
            pub fn clear(&mut self);
        }
    }

    pub fn contains(&self, tag: impl Into<Tag>) -> bool {
        self.0.contains(tag)
    }

    pub fn contains_ref(&self, tag: &Tag) -> bool {
        self.0.contains(tag.clone())
    }

    pub fn intersects(&self, other: &WhitelistedTags) -> bool {
        self.0.intersects(&other.0)
    }

    pub fn extend_from(&mut self, other: &WhitelistedTags) {
        for tag in other.0.iter() {
            self.0.0.insert(tag.clone());
        }
    }
}
//impl into tagset
impl From<&WhitelistedTags> for TagSet {
    fn from(tags: &WhitelistedTags) -> Self {
        tags.0.clone()
    }
}

impl IntoIterator for WhitelistedTags {
    type Item = Tag;
    type IntoIter = <bevy::platform::collections::HashSet<Tag> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a WhitelistedTags {
    type Item = &'a Tag;
    type IntoIter = <&'a bevy::platform::collections::HashSet<Tag> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0.0).into_iter()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlacklistedTags(pub TagSet);

impl BlacklistedTags {
    delegate! {
        to self.0 {
            pub fn is_empty(&self) -> bool;
            pub fn len(&self) -> usize;
            pub fn clear(&mut self);
        }
    }
    pub fn new<S: AsRef<str>>(tags: impl IntoIterator<Item = S>) -> Self {
        Self(TagSet::new(tags))
    }

    pub fn contains(&self, tag: impl Into<Tag>) -> bool {
        self.0.contains(tag)
    }

    pub fn contains_ref(&self, tag: &Tag) -> bool {
        self.0.contains(tag.clone())
    }

    pub fn intersects(&self, other: &BlacklistedTags) -> bool {
        self.0.intersects(&other.0)
    }

    pub fn extend_from(&mut self, other: &BlacklistedTags) {
        for tag in other.0.iter() {
            self.0.0.insert(tag.clone());
        }
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Tag) -> bool,
    {
        self.0.0.retain(f);
    }
}

impl From<&BlacklistedTags> for TagSet {
    fn from(tags: &BlacklistedTags) -> Self {
        tags.0.clone()
    }
}

impl IntoIterator for BlacklistedTags {
    type Item = Tag;
    type IntoIter = <bevy::platform::collections::HashSet<Tag> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a BlacklistedTags {
    type Item = &'a Tag;
    type IntoIter = <&'a bevy::platform::collections::HashSet<Tag> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0.0).into_iter()
    }
}
