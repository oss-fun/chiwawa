use std::{rc::Rc, cell::RefCell};

pub struct DataAddr(Rc<RefCell<DataInst>>);
pub struct DataInst {
    pub data: Vec<u8>,    
}