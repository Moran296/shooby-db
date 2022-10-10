use crate::ShoobyField;

pub trait ShoobyObserver {
    type ID;

    fn update(&self, field: &ShoobyField<Self::ID>);
}

// pub trait Oberved {
//     type ID;

//     fn add_observer(&mut self, observer: &dyn Observer<ID = Self::ID>);
// }
