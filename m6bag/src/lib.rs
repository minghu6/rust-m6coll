use std::{
    borrow::Borrow,
    collections::{btree_map::Entry, BTreeMap}, fmt,
};


pub struct BTreeBag<T> {
    map: BTreeMap<T, usize>,
}


impl<T> BTreeBag<T> {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
}

impl<T: Ord> BTreeBag<T> {
    /// return same item number after insert
    pub fn insert(&mut self, item: T) -> usize {
        match self.map.entry(item) {
            Entry::Vacant(entry) => {
                entry.insert(1);
                1
            }
            Entry::Occupied(mut entry) => {
                let count = entry.get_mut();
                *count += 1;
                *count
            }
        }
    }

    pub fn count<Q: Ord>(&self, key: &Q) -> usize
    where
        T: Borrow<Q>,
    {
        self.map.get(&key).cloned().unwrap_or(0)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&T, usize)> {
        self.map.iter().map(|(k, v)| (k, *v))
    }
}

impl<T: fmt::Debug> fmt::Debug for BTreeBag<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.map, f)
    }
}

impl<T: Ord> FromIterator<T> for BTreeBag<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut bag = Self::new();

        for i in iter {
            bag.insert(i);
        }

        bag
    }
}


#[macro_export]
macro_rules! btree_bag {
    ( $($value:expr),* ) => {
        {
            let mut _bag = $crate::BTreeBag::new();

            $(
                _bag.insert($value);
            )*

            _bag
        }
    };
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_case1() {
        let bag = btree_bag! { 3, 2, 3, 2, 1, 3, 4, 2, 3 };

        assert_eq!(bag.count(&3), 4);
        assert_eq!(bag.count(&2), 3);
        assert_eq!(bag.count(&1), 1);
        assert_eq!(bag.count(&10), 0);

        let ents = bag.iter().collect::<Vec<_>>();

        assert_eq!(ents, vec![(&1, 1), (&2, 3), (&3, 4), (&4, 1)]);

        println!("{bag:?}");
    }
}
