extern crate libc;

use self::libc::*;
use std::mem;

pub struct OwnedCriticalSection {
    mutex: pthread_mutex_t
}

impl OwnedCriticalSection {
    fn new() -> Self {
        OwnedCriticalSection {
            mutex: PTHREAD_MUTEX_INITIALIZER
        }
    }

    fn init(&mut self) {
        unsafe {
            let mut attr: pthread_mutexattr_t = mem::zeroed();
            let r = pthread_mutexattr_init(&mut attr);
            assert_eq!(r, 0);
            let r = pthread_mutexattr_settype(&mut attr, PTHREAD_MUTEX_ERRORCHECK);
            assert_eq!(r, 0);
            let r = pthread_mutex_init(&mut self.mutex, &attr);
            assert_eq!(r, 0);
            let r = pthread_mutexattr_destroy(&mut attr);
        }
    }

    fn destroy(&mut self) {
        unsafe {
            let r = pthread_mutex_destroy(&mut self.mutex);
            assert_eq!(r, 0);
        }
    }

    fn lock(&mut self) {
        unsafe {
            let r = pthread_mutex_lock(&mut self.mutex);
            assert_eq!(r, 0, "Deadlock");
        }
    }

    fn unlock(&mut self) {
        unsafe {
            let r = pthread_mutex_unlock(&mut self.mutex);
            assert_eq!(r, 0, "Unlocking unlocked mutex");
        }
    }

    fn assert_current_thread_owns(&mut self) {
        unsafe {
            let r = pthread_mutex_lock(&mut self.mutex);
            assert_eq!(r, EDEADLK);
        }
    }
}

impl Drop for OwnedCriticalSection {
    fn drop(&mut self) {
        self.destroy();
    }
}

#[test]
fn test_create_critical_section() {
    let mut section = OwnedCriticalSection::new();
    section.init();
    section.lock();
    section.assert_current_thread_owns();
    section.unlock();
}

#[test]
#[should_panic]
fn test_critical_section_destroy_without_unlocking_locked() {
    let mut section = OwnedCriticalSection::new();
    section.init();
    section.lock();
    section.assert_current_thread_owns();
    // Get EBUSY(16) since we destroy the object
    // referenced by mutex while it is locked.
}

#[test]
#[should_panic]
fn test_critical_section_unlock_without_locking() {
    let mut section = OwnedCriticalSection::new();
    section.init();
    section.unlock();
    // Get EPERM(1) since it has no privilege to
    // perform the operation.
}

// #[test]
// #[should_panic]
// fn test_critical_section_assert_without_locking() {
//     let mut section = OwnedCriticalSection::new();
//     section.init();
//     section.assert_current_thread_owns();
//     // Get 0 since calling assert_current_thread_owns is equal to
//     // call lock().
// }

#[test]
fn test_critical_section_multithread() {
    use std::thread;
    use std::time::Duration;

    struct Resource {
        value: u32,
        mutex: OwnedCriticalSection,
    }

    let mut resource = Resource {
        value: 0,
        mutex: OwnedCriticalSection::new(),
    };

    resource.mutex.init();

    // Make a vector to hold the children which are spawned.
    let mut children = vec![];

    println!("resource @ {:p}", &resource);
    // Rust compiler disallows the pointer to be passed into threads.
    // A hacky way to do so is to convert the pointer into a value
    // so it can copy the value(which is actually an address) to threads.
    let resource_ptr = &mut resource as *mut Resource as usize;

    for i in 0..10 {
        // Spin up another thread
        children.push(thread::spawn(move || {
            println!("resource address: {:x}", resource_ptr);
            let res = unsafe {
                let ptr = resource_ptr as *mut Resource;
                &mut (*ptr)
            };
            assert_eq!(res as *mut Resource as usize, resource_ptr);

            // Test fails after commenting res.mutex.lock() and
            // res.mutex.unlock() since the order to run the threads
            // is random.
            res.mutex.lock(); // ---------------------------------------+
                                                                     // |
            res.value = i;                                           // |
            thread::sleep(Duration::from_millis(1));                 // | critical
            println!("this is thread number {}, resource value: {}", // | section
                    i, res.value);                                   // |
            // assert_eq!(i, res.value);                             // |
                                                                     // |
            res.mutex.unlock(); // <------------------------------------+
            i == res.value
        }));
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let result = child.join().unwrap();
        assert!(result)
    }
}

#[test]
fn test_dummy_mutex_multithread() {
    use std::sync::Mutex;
    use std::thread;
    use std::time::Duration;

    struct Resource {
        value: u32,
        mutex: Mutex<()>,
    }

    let mut resource = Resource {
        value: 0,
        mutex: Mutex::new(()),
    };

    // Make a vector to hold the children which are spawned.
    let mut children = vec![];

    println!("resource @ {:p}", &resource);
    // Rust compiler disallows the pointer to be passed into threads.
    // A hacky way to do so is to convert the pointer into a value
    // so it can copy the value(which is actually an address) to threads.
    let resource_ptr = &mut resource as *mut Resource as usize;

    for i in 0..10 {
        // Spin up another thread
        children.push(thread::spawn(move || {
            println!("resource address: {:x}", resource_ptr);
            let res = unsafe {
                let ptr = resource_ptr as *mut Resource;
                &mut (*ptr)
            };
            assert_eq!(res as *mut Resource as usize, resource_ptr);

            // Test fails after commenting res.mutex.lock() since the order
            // to run the threads is random.
            // The scope of `guard` is a critical section.
            let mut _guard = res.mutex.lock().unwrap();  // ------------+
                                                                     // |
            res.value = i;                                           // |
            thread::sleep(Duration::from_millis(1));                 // | critical
            println!("this is thread number {}, resource value: {}", // | section
                    i, res.value);                                   // |
            // assert_eq!(i, res.value);                             // |
                                                                     // |
            i == res.value                                           // |
        })); // <-------------------------------------------------------+
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let result = child.join().unwrap();
        assert!(result)
    }
}
