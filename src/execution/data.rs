use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct DataAddr(Rc<RefCell<DataInst>>);
#[derive(Debug)]
pub struct DataInst {
    pub _data: Vec<u8>,
}

impl DataAddr {
    pub fn new(data: &Vec<u8>) -> DataAddr {
        DataAddr(Rc::new(RefCell::new(DataInst {
            _data: data.clone(),
        })))
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.0.borrow()._data.clone()
    }
}
