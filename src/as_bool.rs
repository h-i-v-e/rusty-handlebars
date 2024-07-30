/*! as_bool provides an expanded notion of what is *true* and what is *false*.

    Specifically with the AsBool trait, which an implementing type can
    use to express how it should be represented in a boolean context.

    This crate also provides implementations of AsBool for Rust's builtin types
    and collections from the Standard Library. These implementations provide a
    truth table similar to the *Groovy Truth* implemented in the Groovy
    programming language. The truth table can be described as follow:

    * booleans behave as expected.
    * all non-zero numbers are `true`.
    * `0` , `0.0` , `f32::NAN`, `f64::NAN`, and `'\0'` are `false`.
    * non-empty strings are `true`.
    * empty strings are `false`.
    * non-empty collections are `true`.
    * empty collections are `false`.
    * `None` is always `false`.
    * `Err` is always `false`.
    * `Ok` and `Some` are unwrapped and the contained item is evaluated according
    to the preceding rules.
**/
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};

/// `AsBool` defines a type's behavior in a boolean context. Basically, it converts
/// the implementing type to `bool`.
pub trait AsBool {
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