// Copyright (c) 2016 multimap developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::collections::hash_map::OccupiedEntry as HashMapOccupiedEntry;
use std::collections::hash_map::VacantEntry as HashMapVacantEntry;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::slice;

pub enum OneOrAny<T> {
    One(T),
    Any(Vec<T>),
}

impl<T> Default for OneOrAny<T> {
    fn default() -> Self {
        OneOrAny::Any(Vec::new())
    }
}

impl<T> From<T> for OneOrAny<T> {
    fn from(one: T) -> Self {
        OneOrAny::One(one)
    }
}

impl<T> From<Vec<T>> for OneOrAny<T> {
    fn from(any: Vec<T>) -> Self {
        OneOrAny::Any(any)
    }
}

impl<T: Clone> Clone for OneOrAny<T> {
    fn clone(&self) -> Self {
        match self {
            OneOrAny::One(ref one) => OneOrAny::One(one.clone()),
            OneOrAny::Any(any) if any.len() == 1 => OneOrAny::One(any[0].clone()),
            OneOrAny::Any(any) => OneOrAny::Any(any.clone()),
        }
    }
}

impl<T> OneOrAny<T> {
    pub fn retain<F: FnMut(&T)->bool>(&mut self,  mut f: F) {
        match self {
            OneOrAny::One(ref mut value) if f(value) => {},
            OneOrAny::One(_) => *self = Self::default(),
            OneOrAny::Any(ref mut values) => values.retain(f),
        }
    }

    pub fn as_slice(&self) -> &[T] {
        match self {
            OneOrAny::One(ref value) => slice::from_ref(value),
            OneOrAny::Any(ref values) => values,
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut[T] {
        match self {
            OneOrAny::One(ref mut value) => slice::from_mut(value),
            OneOrAny::Any(ref mut values) => values,
        }
    }

    pub fn as_mut_vec(&mut self) -> &mut Vec<T> {
        if let OneOrAny::Any(any) = self {
            any
        } else {
            let moved = mem::replace(self, OneOrAny::Any(Vec::with_capacity(1)));
            let OneOrAny::One(value) = moved else {unreachable!()};
            let OneOrAny::Any(slot) = self else {unreachable!()};
            slot.push(value);
            slot
        }
    }
}

impl<T> Deref for OneOrAny<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> DerefMut for OneOrAny<T> {
    fn deref_mut(&mut self) -> &mut[T] {
        self.as_mut_slice()
    }
}

impl<T> From<OneOrAny<T>> for Vec<T> {
    fn from(values: OneOrAny<T>) -> Vec<T> {
        match values {
            OneOrAny::One(value) => vec![value],
            OneOrAny::Any(values) => values,
        }
    }
}

/// A view into a single occupied location in a MultiMap.
pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    #[doc(hidden)]
    pub inner: HashMapOccupiedEntry<'a, K, OneOrAny<V>>,
}

/// A view into a single empty location in a MultiMap.
pub struct VacantEntry<'a, K: 'a, V: 'a> {
    #[doc(hidden)]
    pub inner: HashMapVacantEntry<'a, K, OneOrAny<V>>,
}

/// A view into a single location in a map, which may be vacant or occupied.
pub enum Entry<'a, K: 'a, V: 'a> {
    /// An occupied Entry.
    Occupied(OccupiedEntry<'a, K, V>),

    /// A vacant Entry.
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K: 'a, V: 'a> OccupiedEntry<'a, K, V> {
    /// Gets a reference to the first item in value in the vector corresponding to entry.
    pub fn get(&self) -> &V {
        &self.inner.get()[0]
    }

    /// Gets a reference to the values (vector) corresponding to entry.
    pub fn get_slice(&self) -> &[V] {
        self.inner.get()
    }

    /// Gets a mut reference to the first item in value in the vector corresponding to entry.
    pub fn get_mut(&mut self) -> &mut V {
        &mut self.inner.get_mut()[0]
    }

    /// Gets a mut reference to the values (vector) corresponding to entry.
    pub fn get_vec_mut(&mut self) -> &mut Vec<V> {
        self.inner.get_mut().as_mut_vec()
    }

    /// Converts the OccupiedEntry into a mutable reference to the first item in value in the entry
    /// with a lifetime bound to the map itself
    pub fn into_mut(self) -> &'a mut V {
        &mut self.inner.into_mut()[0]
    }

    /// Converts the OccupiedEntry into a mutable reference to the values (vector) in the entry
    /// with a lifetime bound to the map itself
    pub fn into_vec_mut(self) -> &'a mut Vec<V> {
        self.inner.into_mut().as_mut_vec()
    }

    /// Inserts a new value onto the vector of the entry.
    pub fn insert(&mut self, value: V) {
        self.get_vec_mut().push(value);
    }

    /// Extends the existing vector with the specified values.
    pub fn insert_vec(&mut self, values: Vec<V>) {
        self.get_vec_mut().extend(values);
    }

    /// Takes the values (vector) out of the entry, and returns it
    pub fn remove(self) -> Vec<V> {
        self.inner.remove().into()
    }
}

impl<'a, K: 'a, V: 'a> VacantEntry<'a, K, V> {
    /// Sets the first value in the vector of the entry with the VacantEntry's key,
    /// and returns a mutable reference to it.
    pub fn insert(self, value: V) -> &'a mut V {
        &mut self.inner.insert(OneOrAny::One(value))[0]
    }

    /// Sets values in the entry with the VacantEntry's key,
    /// and returns a mutable reference to it.
    pub fn insert_vec(self, values: Vec<V>) -> &'a mut Vec<V> {
        self.inner.insert(OneOrAny::Any(values)).as_mut_vec()
    }
}

impl<'a, K: 'a, V: 'a> Entry<'a, K, V> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry. This will return a mutable reference to the
    /// first value in the vector corresponding to the specified key.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the default values if empty, and returns
    /// a mutable reference to the values (the corresponding vector to the specified key) in
    /// the entry.
    pub fn or_insert_vec(self, defaults: Vec<V>) -> &'a mut Vec<V> {
        match self {
            Entry::Occupied(entry) => entry.into_vec_mut(),
            Entry::Vacant(entry) => entry.insert_vec(defaults),
        }
    }
}
