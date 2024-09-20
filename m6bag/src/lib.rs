use std::{
    borrow::Borrow,
    collections::{btree_map::Entry, BTreeMap}, fmt,
};


pub struct BTreeBag<T> {
    map: BTreeMap<T, Vec<T>>,
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
    pub fn insert(&mut self, item: T) -> usize where T: Clone {
        match self.map.entry(item.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(vec![item]);
                1
            }
            Entry::Occupied(mut entry) => {
                let colls = entry.get_mut();
                colls.push(item);
                colls.len()
            }
        }
    }

    pub fn count<Q: Ord>(&self, key: &Q) -> usize
    where
        T: Borrow<Q>,
    {
        if let Some(coll) = self.map.get(&key) {
            coll.len()
        }
        else {
            0
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&T, usize)> {
        self.map.iter().map(|(k, v)| (k, v.len()))
    }

    pub fn flat_iter(&self) -> impl Iterator<Item = &T> {
        self.map.values().flatten()
    }

    
}

impl<T: fmt::Debug> fmt::Debug for BTreeBag<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.map, f)
    }
}

impl<T: Ord + Clone> FromIterator<T> for BTreeBag<T> {
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
    ( $($value:expr),* $(,)?) => {
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
    use crate::BTreeBag;

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

    #[test]
    fn test_case2() {
        use derive_where::derive_where;
        use derive_new::new;

        #[derive(new)]
        #[derive_where(PartialOrd, Ord, PartialEq, Eq)]
        #[derive(Clone, Debug)]
        struct A {
            a1: usize,
            a2: usize,
            #[derive_where(skip)]
            a3: usize
        }

        let colls = vec![
            A::new( 3, 1, 2),
            A::new( 3, 1, 3),
            A::new( 3, 1, 1),
            A::new( 4, 1, 1),
            A::new( 4, 1, 1),
            A::new( 4, 2, 1),
            A::new( 4, 2, 2),
        ];

        let bag = BTreeBag::from_iter(colls.iter().cloned()) ;

        assert_eq!(bag.count(&A::new( 3, 1, 5)), 3);
        assert_eq!(bag.count(&A::new( 4, 2, 2)), 2);

        let ents = bag.flat_iter().cloned().collect::<Vec<_>>();

        assert_eq!(ents, colls);

        println!("{bag:#?}");
    }
}
