//! Data segment instances for memory initialization.

use std::cell::RefCell;
use std::rc::Rc;

/// Reference-counted handle to a data segment instance.
#[derive(Clone, Debug)]
pub struct DataAddr(Rc<RefCell<DataInst>>);

/// Data segment instance.
#[derive(Debug)]
pub struct DataInst {
    pub _data: Vec<u8>,
}

impl DataAddr {
    /// Creates a new data segment from byte slice.
    pub fn new(data: &Vec<u8>) -> DataAddr {
        DataAddr(Rc::new(RefCell::new(DataInst {
            _data: data.clone(),
        })))
    }

    /// Returns a copy of the data bytes.
    pub fn get_data(&self) -> Vec<u8> {
        self.0.borrow()._data.clone()
    }

    /// Clears the data (for data.drop instruction).
    pub fn drop_data(&self) {
        self.0.borrow_mut()._data.clear();
    }
}
