use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use super::*;

#[derive(Clone)]
pub struct TypeTab {
    pub parent: Option<Rc<TypeTab>>,
    pub types:  RefCell<Vec<Type>>,
}

impl TypeTab {
    pub fn new(parent: Rc<TypeTab>, types: &Vec<Type>) -> TypeTab {
        TypeTab {
            parent: Some(parent),
            types:  RefCell::new(types.clone()),
        }
    }

    pub fn new_global() -> TypeTab {
        TypeTab {
            parent: None,
            types: RefCell::new(Vec::new()),
        }
    }

    pub fn set_type(&self, index: usize, env_index: usize, t: Type) -> Result<(), Response> {
        if env_index == 0 {
            let mut types = self.types.borrow_mut();
            match types.get_mut(index) {
                Some(v) => {
                    *v = t;
                    Ok(())
                },
                None => Err(Response::error(None, format!("invalid type env index: {}", env_index))),
            }
        } else {
            match self.parent {
                Some(ref p) => p.set_type(index, env_index - 1, t),
                None => Err(Response::error(None, format!("invalid type env index: {}", env_index)))
            }
        }
    }

    pub fn get_type(&self, index: usize, env_index: usize) -> Result<Type, Response> {
        if env_index == 0 {
            match self.types.borrow().get(index) {
                Some(v) => Ok(v.clone()),
                None => Err(Response::error(None, format!("invalid type index: {}", index)))
            }
        } else {
            match self.parent {
                Some(ref p) => p.get_type(index, env_index - 1),
                None => Err(Response::error(None, format!("invalid type index: {}", index)))
            }
        }
    }

    pub fn visualize(&self, env_index: usize) {
        if env_index > 0 {
            if let Some(ref p) = self.parent {
                p.visualize(env_index - 1);
                println!("------------------------------");
            }
        }

        for (i, v) in self.types.borrow().iter().enumerate() {
            println!("({} : {}) = {:?}", i, env_index, v)
        }
    }

    fn dump(&self, f: &mut fmt::Formatter, env_index: usize) -> fmt::Result {
        if env_index > 0 {
            if let Some(ref p) = self.parent {
                try!(p.dump(f, env_index - 1));
                try!(writeln!(f, "------------------------------"));
            }
        }

        for (i, v) in self.types.borrow().iter().enumerate() {
            try!(writeln!(f, "({} : {}) = {:?}", i, env_index, v))
        }

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.types.borrow().len()
    }

    pub fn grow(&self) {
        self.types.borrow_mut().push(Type::Undefined)
    }
}

impl fmt::Debug for TypeTab {
    fn fmt(&self, f : &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(self.dump(f, 0));
        Ok(())
    }
}
