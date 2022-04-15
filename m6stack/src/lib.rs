#![feature(box_syntax)]


use std::fmt;
use std::iter;
use std::iter::Rev;
use std::vec;

////////////////////////////////////////////////////////////////////////////////
/////// Stack

#[derive(Clone)]
pub struct Stack<T> {
    _value_vec: Vec<T>,
}

impl<T> Stack<T> {
    // staic method
    pub fn new() -> Self {
        Self { _value_vec: vec![] }
    }

    pub fn push(&mut self, item: T) {
        self._value_vec.push(item)
    }

    pub fn pop(&mut self) -> Option<T> {
        self._value_vec.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self._value_vec.last()
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self._value_vec.last_mut()
    }

    pub fn len(&self) -> usize {
        self._value_vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self._value_vec.len() == 0
    }

    /// FILO
    pub fn stack_iter<'a>(&'a self) -> impl Iterator<Item=&T> + 'a {
        let mut iter = self._value_vec.iter();

        std::iter::from_fn(move || {
            iter.next()
        })

        // AnIteratorWrapper {
        //     iter: box self._value_vec.iter().rev()
        // }
    }

    /// FIFO
    pub fn queue_iter(&self) -> AnIteratorWrapper<&T> {
        AnIteratorWrapper {
            iter: box self._value_vec.iter(),
        }
    }

    /// This method will move the content of stack
    pub fn extend_stack(&mut self, income_stack: Stack<T>) {
        for item in income_stack.into_iter().rev() {
            self.push(item);
        }
    }
}


impl<T: Clone> Stack<T> {
    /// Same order with iter (rev)
    pub fn to_vec(&self) -> Vec<T> {
        self._value_vec.iter().rev().cloned().collect::<Vec<T>>()
    }
}

impl<T> iter::IntoIterator for Stack<T> {
    type Item = T;
    type IntoIter = Rev<vec::IntoIter<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self._value_vec.into_iter().rev()
    }
}

/// rev order of Vec
impl<T> From<Vec<T>> for Stack<T> {
    fn from(income: Vec<T>) -> Self {
        Self {
            _value_vec: income.into_iter().rev().collect(),
        }
    }
}

impl<T> Extend<T> for Stack<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self._value_vec.push(item);
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return Ok(());
        }

        let msg
        = self._value_vec.iter().rev().map(|item| {
            if f.alternate() {
                format!("{:#?}", item)
            }
            else {
                format!("{:?}", item)
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

        write!(f, "{}", msg)
    }
}

impl<T: fmt::Display> fmt::Display for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return Ok(());
        }

        let msg
        = self._value_vec.iter().rev().map(|item| {
            format!("{}", item)
        })
        .collect::<Vec<String>>()
        .join(" ");

        write!(f, "{}", msg)
    }
}


pub struct AnIteratorWrapper<'a, T> {
    pub iter: Box<dyn Iterator<Item=T> + 'a>
}


impl<'a, T> Iterator for AnIteratorWrapper<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}




#[macro_export]
macro_rules! stack {
    ( $($value:expr),* ) => {
        {
            let mut _stack = $crate::Stack::new();

            $(
                _stack.push($value);
            )*

            _stack
        }
    };
}


#[cfg(test)]
mod test {
    use crate::*;

    #[derive(Debug, Default)]
    struct Foo {
        pub(crate) bar: usize,
    }


    #[test]
    fn test_stack_struct() {
        let mut stack = Stack::new();
        stack.extend(vec![1, 2, 3].into_iter());

        assert_eq!(stack.to_vec(), vec![3, 2, 1]);
        assert_eq!(stack![1, 2, 3].to_vec(), vec![3, 2, 1]);

        stack.extend_stack(stack![4, 5]);
        assert_eq!(stack.to_vec(), vec![5, 4, 3, 2, 1]);

        println!("{:?}", stack);
        println!("{:#?}", stack);
        println!("{}", stack);

        for e in stack.stack_iter() {
            println!("{}", e);
        }
    }

    #[test]
    fn test_stack_peek_mut() {
        let mut stack0 = stack![Foo::default()];

        assert_eq!(stack0.peek().unwrap().bar, 0);

        if let Some(foo_mut) = stack0.peek_mut() {
            foo_mut.bar += 1;
        }

        assert_eq!(stack0.peek().unwrap().bar, 1);
    }
}
