use std::fmt::Debug;
use std::ptr;
use std::slice;

trait AutoArrayWrapper {
    fn push(&mut self, data: *const (), elements: usize);
    fn pop(&mut self, elements: usize) -> bool;
    fn elements(&self) -> usize;
    fn data(&self) -> *const ();
}

#[derive(Debug)]
struct AutoArrayImpl<T: Clone> {
    ar: Vec<T>,
}

impl<T: Clone> AutoArrayImpl<T> {
    fn new(size: usize) -> Self {
        AutoArrayImpl {
            ar: Vec::<T>::with_capacity(size),
        }
    }
}

impl<T: Clone> AutoArrayWrapper for AutoArrayImpl<T> {
    fn push(&mut self, data: *const (), elements: usize) {
        let slice = unsafe { slice::from_raw_parts(data as *mut T, elements) };
        self.ar.extend_from_slice(slice);
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

    fn data(&self) -> *const () {
        if self.ar.is_empty() {
            return ptr::null();
        }
        self.ar.as_ptr() as *const ()
    }
}

#[cfg(test)]
fn test_auto_array_impl<T: Clone + Debug + PartialEq>(buf: &[T]) {
    let mut auto_array = AutoArrayImpl::<T>::new(5);
    assert_eq!(auto_array.elements(), 0);
    assert!(auto_array.data().is_null());

    // Check if push works.
    auto_array.push(buf.as_ptr() as *const (), buf.len());
    assert_eq!(auto_array.elements(), buf.len());

    let data = auto_array.data() as *const T;
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

    let data = auto_array.data() as *const T;
    for i in 0..buf.len() - POP {
        unsafe {
            assert_eq!(*data.add(i), buf[POP + i]);
        }
    }
}

#[cfg(test)]
fn test_auto_array_wrapper<T: Clone + Debug + PartialEq>(buf: &[T]) {
    let mut auto_array: Option<Box<AutoArrayWrapper>> = None;
    // println!("{:?}", auto_array);
    auto_array = Some(Box::new(AutoArrayImpl::<T>::new(5)));
    assert_eq!(auto_array.as_ref().unwrap().elements(), 0);
    assert!(auto_array.as_ref().unwrap().data().is_null());

    // Check if push works.
    auto_array
        .as_mut()
        .unwrap()
        .push(buf.as_ptr() as *const (), buf.len());
    assert_eq!(auto_array.as_ref().unwrap().elements(), buf.len());

    let data = auto_array.as_ref().unwrap().data() as *const T;
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

    let data = auto_array.as_ref().unwrap().data() as *const T;
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
