use std::collections::HashMap;

use crate::types::{self, QObject, QType};

pub struct QInterp {
    globals: QObject,
    locals: Vec<QObject>,
}

impl QInterp {
    pub fn init() -> Self {
        Self {
            globals: QObject::new(),
            locals: {
                let mut v = Vec::<QObject>::new();
                v.push(QObject::new());
                v
            },
        }
    }

    pub fn store_global(&mut self, name: String, value: QType) {
        _ = self.globals.insert(name, value);
    }

    pub fn fetch_global(&self, name: String) -> Option<QType> {
        let val = self.globals.get(&name);
        if let Some(o) = val {
            Some(o.clone())
        } else {
            None
        }
    }

    pub fn store_local(&mut self, name: String, value: QType) {
        let last = self.locals.len()-1;
        let mut curr_scope = self.locals.get_mut(last);
        if let Some(s) = curr_scope {
            _ = s.insert(name, value);
        } else { unreachable!() }
    }
}