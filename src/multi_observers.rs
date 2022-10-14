use crate::errors::ShoobyError;
use crate::traits::*;
use crate::ShoobyField;
use heapless::Vec as HeaplessVec;

pub struct MultiObserver<ID, T: ShoobyObserver<ID = ID>, const N: usize> {
    observers: HeaplessVec<T, N>,
}

impl<ID, T: ShoobyObserver<ID = ID>, const N: usize> MultiObserver<ID, T, N> {
    pub fn new() -> Self {
        Self {
            observers: HeaplessVec::new(),
        }
    }

    pub fn add(&mut self, observer: T) -> Result<(), ShoobyError> {
        self.observers
            .push(observer)
            .map_err(|_| ShoobyError::OutOfBounds)
    }
}

impl<ID, T: ShoobyObserver<ID = ID>, const N: usize> ShoobyObserver for MultiObserver<ID, T, N> {
    type ID = ID;

    fn update(&self, item: &ShoobyField<Self::ID>) {
        for observer in self.observers.iter() {
            observer.update(item);
        }
    }
}
