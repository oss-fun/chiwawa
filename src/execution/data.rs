use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct DataAddr(Arc<RwLock<DataInst>>);
#[derive(Debug)]
pub struct DataInst {
    pub _data: Vec<u8>,
}

impl DataAddr {
    pub fn new(data: &Vec<u8>) -> DataAddr {
        DataAddr(Arc::new(RwLock::new(DataInst {
            _data: data.clone(),
        })))
    }
}
