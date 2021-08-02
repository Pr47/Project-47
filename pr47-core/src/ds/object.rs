use std::collections::HashMap;

use crate::data::Value;
use crate::data::traits::StaticBase;
use crate::util::mem::FatPointer;
use crate::util::void::Void;

pub struct Object {
    fields: HashMap<String, Value>
}

impl Object {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new()
        }
    }
}

impl StaticBase<Object> for Void {
    fn type_name() -> String { "object".into() }

    #[inline] fn children(
        vself: *const Object
    ) -> Option<Box<dyn Iterator<Item=FatPointer> + 'static>> {
        unsafe {
            let iter = Box::new((*vself).fields.iter().map(|x| x.1.ptr_repr.clone()));
            Some(iter)
        }
    }
}