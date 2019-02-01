// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=53180777432a468c4d0fce913ff69d19

use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::ptr;

#[derive(Debug)]
pub enum AutoArrayWrapper {
    Float(AutoArrayImpl<f32>),
    Short(AutoArrayImpl<i16>),
}

impl AutoArrayWrapper {
    pub fn push<T: Any>(&mut self, data: &[T]) {
        match self {
            AutoArrayWrapper::Float(array) => unsafe {
                assert_eq!(TypeId::of::<T>(), TypeId::of::<f32>());
                array.push(&*(data as *const [T] as *const [f32]));
            },
            AutoArrayWrapper::Short(array) => unsafe {
                assert_eq!(TypeId::of::<T>(), TypeId::of::<i16>());
                array.push(&*(data as *const [T] as *const [i16]));
            },
        }
    }

    pub fn pop(&mut self, elements: usize) -> bool {
        match self {
            AutoArrayWrapper::Float(array) => array.pop(elements),
            AutoArrayWrapper::Short(array) => array.pop(elements),
        }
    }

    pub fn elements(&self) -> usize {
        match self {
            AutoArrayWrapper::Float(array) => array.elements(),
            AutoArrayWrapper::Short(array) => array.elements(),
        }
    }

    pub fn as_ptr<T>(&self) -> *const T {
        match self {
            AutoArrayWrapper::Float(array) => array.as_ptr() as *const T,
            AutoArrayWrapper::Short(array) => array.as_ptr() as *const T,
        }
    }
}

#[derive(Debug)]
pub struct AutoArrayImpl<T: Clone> {
    ar: Vec<T>,
}

impl<T: Clone> AutoArrayImpl<T> {
    pub fn new(size: usize) -> Self {
        AutoArrayImpl {
            ar: Vec::<T>::with_capacity(size),
        }
    }

    fn push(&mut self, data: &[T]) {
        self.ar.extend_from_slice(data);
    }

    fn pop(&mut self, elements: usize) -> bool {
        if elements > self.ar.len() {
            return false;
        }
        self.ar.drain(0..elements);
        true
    }

    fn elements(&self) -> usize {
        self.ar.len()
    }

    fn as_ptr(&self) -> *const T {
        if self.ar.is_empty() {
            return ptr::null();
        }
        self.ar.as_ptr()
    }
}

#[cfg(test)]
fn test_auto_array_impl<T: Clone + Debug + PartialEq>(buf: &[T]) {
    let mut auto_array = AutoArrayImpl::<T>::new(5);
    assert_eq!(auto_array.elements(), 0);
    assert!(auto_array.as_ptr().is_null());

    // Check if push works.
    auto_array.push(buf);
    assert_eq!(auto_array.elements(), buf.len());

    let data = auto_array.as_ptr();
    for i in 0..buf.len() {
        unsafe {
            assert_eq!(*data.add(i), buf[i]);
        }
    }

    // Check if pop works.
    assert!(!auto_array.pop(buf.len() + 1));
    const POP: usize = 3;
    assert!(POP < buf.len());
    assert!(auto_array.pop(POP));
    assert_eq!(auto_array.elements(), buf.len() - POP);

    let data = auto_array.as_ptr();
    for i in 0..buf.len() - POP {
        unsafe {
            assert_eq!(*data.add(i), buf[POP + i]);
        }
    }
}

#[cfg(test)]
fn test_auto_array_wrapper<T: Any + Clone + Debug + PartialEq>(buf: &[T]) {
    let mut auto_array: Option<AutoArrayWrapper> = None;

    // Initialize the buffer based on the type.
    let type_id = TypeId::of::<T>();
    auto_array = Some(if type_id == TypeId::of::<f32>() {
        AutoArrayWrapper::Float(<AutoArrayImpl<f32>>::new(5))
    } else if type_id == TypeId::of::<i16>() {
        AutoArrayWrapper::Short(<AutoArrayImpl<i16>>::new(5))
    } else {
        panic!("Unsupported type!");
    });

    assert_eq!(auto_array.as_ref().unwrap().elements(), 0);
    let data = auto_array.as_ref().unwrap().as_ptr() as *const T;
    assert!(data.is_null());

    // Check if push works.
    auto_array.as_mut().unwrap().push(buf);
    assert_eq!(auto_array.as_ref().unwrap().elements(), buf.len());

    let data = auto_array.as_ref().unwrap().as_ptr() as *const T;
    for i in 0..buf.len() {
        unsafe {
            assert_eq!(*data.add(i), buf[i]);
        }
    }

    // Check if pop works.
    assert!(!auto_array.as_mut().unwrap().pop(buf.len() + 1));
    const POP: usize = 3;
    assert!(POP < buf.len());
    assert!(auto_array.as_mut().unwrap().pop(POP));
    assert_eq!(auto_array.as_ref().unwrap().elements(), buf.len() - POP);

    let data = auto_array.as_ref().unwrap().as_ptr() as *const T;
    for i in 0..buf.len() - POP {
        unsafe {
            assert_eq!(*data.add(i), buf[POP + i]);
        }
    }
}

#[test]
fn test_auto_array() {
    let buf_f32 = [1.0_f32, 2.1, 3.2, 4.3, 5.4];
    test_auto_array_impl(&buf_f32);
    test_auto_array_wrapper(&buf_f32);

    let buf_i16 = [5_i16, 8, 13, 21, 34, 55, 89, 144];
    test_auto_array_impl(&buf_i16);
    test_auto_array_wrapper(&buf_i16);
}

#[test]
#[should_panic]
fn test_auto_array_type_safe() {
    let mut array = AutoArrayWrapper::Float(<AutoArrayImpl<f32>>::new(5));
    array.push(&[5_i16, 8, 13, 21, 34, 55, 89, 144]);
    // let mut array = AutoArrayWrapper::Short(<AutoArrayImpl<i16>>::new(5));
    // array.push(&[1.0_f32, 2.1, 3.2, 4.3, 5.4]);
}
