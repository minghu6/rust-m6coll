use std::cmp::Ordering;

/// KeyValue Pair
#[derive(Debug, Eq, Ord)]
pub struct Entry<K, V>(pub K, pub V);


impl<K: PartialEq, V> PartialEq for Entry<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K: PartialOrd, V> PartialOrd for Entry<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_entry() {
        use std::collections::BinaryHeap;
        use std::cmp::Reverse;

        let mut heap = BinaryHeap::new();

        heap.push(Reverse(Entry(2, 3)));
        heap.push(Reverse(Entry(1, 1)));
        heap.push(Reverse(Entry(4, 16)));
        heap.push(Reverse(Entry(2, 1)));


        assert_eq!(heap.pop(), Some(Reverse(Entry(1, 1))));
        assert_eq!(heap.pop(), Some(Reverse(Entry(2, 3))));
        assert_eq!(heap.pop(), Some(Reverse(Entry(2, 1))));
        assert_eq!(heap.pop(), Some(Reverse(Entry(4, 16))));

        assert_eq!(heap.pop(), None);

    }


}