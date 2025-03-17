/*
This is stolen and modified from the as_bool crate.

https://crates.io/crates/as_bool
 */

//! Boolean conversion trait for various Rust types
//!
//! This module provides the `AsBool` trait which defines how different types
//! should behave in a boolean context. It's used internally by the Handlebars
//! templating engine for conditional rendering.
//!
//! The trait is based on the [as_bool crate](https://crates.io/crates/as_bool)
//! but has been modified to suit the needs of this project.
//!
//! # Examples
//!
//! ```rust
//! use rusty_handlebars::AsBool;
//!
//! // Numbers
//! assert!(1.as_bool());
//! assert!(!0.as_bool());
//!
//! // Strings
//! assert!("hello".as_bool());
//! assert!(!"".as_bool());
//!
//! // Collections
//! let vec = vec![1, 2, 3];
//! assert!(vec.as_bool());
//! assert!(!Vec::<i32>::new().as_bool());
//!
//! // Options
//! assert!(Some(1).as_bool());
//! assert!(!None::<i32>.as_bool());
//! ```

use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};

/// Trait for converting values to boolean
///
/// This trait defines how different types should behave in a boolean context.
/// It's used internally by the Handlebars templating engine for conditional rendering.
///
/// # Implementation Rules
///
/// - Numbers: `true` if non-zero, `false` if zero
/// - Strings: `true` if non-empty, `false` if empty
/// - Collections: `true` if non-empty, `false` if empty
/// - Options: `true` if Some and inner value is true, `false` otherwise
/// - Results: `true` if Ok and inner value is true, `false` otherwise
pub trait AsBool {
    /// Converts the value to a boolean
    fn as_bool(&self) -> bool;
}

// Booleans
impl AsBool for bool {
    fn as_bool(&self) -> bool {
        *self
    }
}

impl AsBool for &bool {
    fn as_bool(&self) -> bool {
        **self
    }
}

macro_rules! impl_int {
    ($($t:ty),*) => {
        $(
            impl AsBool for $t{
                fn as_bool(&self) -> bool {
                    *self != 0
                }
            }

            impl AsBool for &$t{
                fn as_bool(&self) -> bool {
                    **self != 0
                }
            }
        )*
    }
}

impl_int!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

macro_rules! impl_float {
    ($($t:ty),*) => {
        $(
            impl AsBool for $t{
                fn as_bool(&self) -> bool {
                    *self != 0.0
                }
            }

            impl AsBool for &$t{
                fn as_bool(&self) -> bool {
                    **self != 0.0
                }
            }
        )*
    }
}

impl_float!(f32, f64);

// Tuples
impl AsBool for () {
    fn as_bool(&self) -> bool {
        false
    }
}

// Arrays
impl<T> AsBool for [T] {
    fn as_bool(&self) -> bool {
        !self.is_empty()
    }
}

macro_rules! impl_list {
    ($($t:ty),*) => {
        $(
            impl<T> AsBool for $t{
                fn as_bool(&self) -> bool {
                    !self.is_empty()
                }
            }

            impl<T> AsBool for &$t{
                fn as_bool(&self) -> bool {
                    !self.is_empty()
                }
            }
        )*
    }
}

impl_list!(Vec<T>, VecDeque<T>, LinkedList<T>, BTreeSet<T>, BinaryHeap<T>);

macro_rules! impl_string {
    ($($t:ty),*) => {
        $(
            impl AsBool for $t{
                fn as_bool(&self) -> bool {
                    !self.is_empty()
                }
            }

            impl AsBool for &$t{
                fn as_bool(&self) -> bool {
                    !self.is_empty()
                }
            }
        )*
    }
}

impl_string!(String, &str);

macro_rules! impl_map {
    ($($t:ty),*) => {
        $(
            impl<K, V> AsBool for $t{
                fn as_bool(&self) -> bool {
                    !self.is_empty()
                }
            }

            impl<K, V> AsBool for &$t{
                fn as_bool(&self) -> bool {
                    !self.is_empty()
                }
            }
        )*
    }
}

impl_map!(HashMap<K, V>, BTreeMap<K, V>, HashSet<K, V>);

// Text
impl AsBool for char {
    fn as_bool(&self) -> bool {
        *self != '\0'
    }
}

impl AsBool for &char {
    fn as_bool(&self) -> bool {
        **self != '\0'
    }
}

// Option
impl<T: AsBool> AsBool for Option<T> {
    fn as_bool(&self) -> bool {
        if let Some(t) = self {
            t.as_bool()
        } else {
            false
        }
    }
}

impl<T: AsBool> AsBool for &Option<T> {
    fn as_bool(&self) -> bool {
        if let Some(t) = self {
            t.as_bool()
        } else {
            false
        }
    }
}

// Result
impl<T: AsBool, E> AsBool for std::result::Result<T, E> {
    fn as_bool(&self) -> bool {
        if let Ok(t) = self {
            t.as_bool()
        } else {
            false
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use crate::AsBool;
    use std::collections::{
        BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque,
    };

    #[test]
    fn the_works() {
        assert!(true.as_bool());
        assert!(!false.as_bool());
        assert!((1 as usize).as_bool());
        assert!(!(0 as usize).as_bool());
        assert!((1 as u8).as_bool());
        assert!(!(0 as u8).as_bool());
        assert!((1 as u16).as_bool());
        assert!(!(0 as u16).as_bool());
        assert!((1 as u32).as_bool());
        assert!(!(0 as u32).as_bool());
        assert!((1 as u64).as_bool());
        assert!(!(0 as u64).as_bool());
        assert!((1 as u128).as_bool());
        assert!(!(0 as u128).as_bool());
        assert!((1 as isize).as_bool());
        assert!(!(0 as isize).as_bool());
        assert!((1 as i8).as_bool());
        assert!(!(0 as i8).as_bool());
        assert!((1 as i16).as_bool());
        assert!(!(0 as i16).as_bool());
        assert!((1 as i32).as_bool());
        assert!(!(0 as i32).as_bool());
        assert!((1 as i64).as_bool());
        assert!(!(0 as i64).as_bool());
        assert!((1 as i128).as_bool());
        assert!(!(0 as i128).as_bool());
        assert!((1.0 as f32).as_bool());
        assert!(!(0.0 as f32).as_bool());
        assert!((1.0 as f64).as_bool());
        assert!(!(0.0 as f64).as_bool());
        assert!(!(f32::NAN).as_bool());
        assert!(!(f64::NAN).as_bool());
        assert!('a'.as_bool());
        assert!(!'\0'.as_bool());
        assert!("a".as_bool());
        assert!(!"".as_bool());
        assert!("a".to_string().as_bool());
        assert!(!"".to_string().as_bool());

        assert!(!().as_bool());
        assert!(![true; 0].as_bool());
        assert!([true; 1].as_bool());

        let mut hm: HashMap<u8, bool> = HashMap::new();
        assert!(!hm.as_bool());
        hm.insert(1, true);
        assert!(hm.as_bool());

        let mut bm: BTreeMap<u8, bool> = BTreeMap::new();
        assert!(!bm.as_bool());
        bm.insert(1, true);
        assert!(bm.as_bool());

        let mut hs: HashSet<bool> = HashSet::new();
        assert!(!hs.as_bool());
        hs.insert(true);
        assert!(hs.as_bool());

        let mut bs: BTreeSet<bool> = BTreeSet::new();
        assert!(!bs.as_bool());
        bs.insert(true);
        assert!(bs.as_bool());

        let mut bh: BinaryHeap<bool> = BinaryHeap::new();
        assert!(!bh.as_bool());
        bh.push(true);
        assert!(bh.as_bool());

        let mut l: LinkedList<bool> = LinkedList::new();
        assert!(!l.as_bool());
        l.push_back(true);
        assert!(l.as_bool());

        let mut vd: VecDeque<bool> = VecDeque::new();
        assert!(!vd.as_bool());
        vd.push_back(true);
        assert!(vd.as_bool());

        let mut v: Vec<bool> = Vec::new();
        assert!(!v.as_bool());
        v.push(true);
        assert!(v.as_bool());

        assert!(Some(true).as_bool());
        assert!(!Some(false).as_bool());
        let n: Option<bool> = None;
        assert!(!n.as_bool());
        let mut o: Result<bool, bool> = Ok(true);
        let mut e: Result<bool, bool> = Err(true);
        assert!(o.as_bool());
        assert!(!e.as_bool());
        o = Ok(false);
        e = Err(false);
        assert!(!o.as_bool());
        assert!(!e.as_bool());
    }
}