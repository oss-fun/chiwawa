use std::{rc::Rc, cell::RefCell};

pub struct DataAddr(Rc<RefCell<DataInst>>);
pub struct DataInst {
    pub data: Vec<u8>,    
}

impl DataAddr {
    pub fn new(data: &Vec<u8>) -> DataAddr{
        DataAddr(
            Rc::new(RefCell::new(DataInst{
               data: data.clone(),
            }))
        )
    }
}