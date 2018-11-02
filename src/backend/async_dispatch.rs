// Reference:
// https://gist.github.com/ChunMinChang/8d13946ebc6c95b2622466c89a0c9bcc
// http://rustaudio.github.io/coreaudio-rs/coreaudio_sys/audio_unit/fn.dispatch_queue_create.html
// http://rustaudio.github.io/coreaudio-rs/coreaudio_sys/audio_unit/fn.dispatch_async_f.html
// https://github.com/phracker/MacOSX-SDKs/blob/9fc3ed0ad0345950ac25c28695b0427846eea966/MacOSX10.13.sdk/usr/include/dispatch/queue.h#L472

extern crate coreaudio_sys as sys;

use std::os::raw::{c_char, c_void};

pub const DISPATCH_QUEUE_SERIAL: sys::dispatch_queue_attr_t = 0 as sys::dispatch_queue_attr_t;

pub fn create_dispatch_queue(
    label: &'static str,
    queue_attr: sys::dispatch_queue_attr_t
) -> sys::dispatch_queue_t
{
    unsafe {
        sys::dispatch_queue_create(
            label.as_ptr() as *const c_char,
            queue_attr
        )
    }
}

// Send: Types that can be transferred across thread boundaries.
// FnOnce: One-time function.
pub fn async_dispatch<F>(queue: sys::dispatch_queue_t, work: F)
  where F: 'static + Send + FnOnce()
{
    let (closure, executor) = create_closure_and_executor(work);
    unsafe {
        sys::dispatch_async_f(queue, closure, executor);
    }
}

// Return an raw pointer to a (unboxed) closure and an executor that
// will run the closure (after re-boxing the closure) when it's called.
fn create_closure_and_executor<F>(
    closure: F
) -> (*mut c_void, sys::dispatch_function_t)
    where F: FnOnce()
{
    extern fn closure_executer<F>(unboxed_closure: *mut c_void)
        where F: FnOnce()
    {
        // Retake the leaked closure.
        let closure: Box<F> = unsafe {
            Box::from_raw(unboxed_closure as *mut F)
        };
        // Execute the closure.
        (*closure)();
        // closure is released after finishiing this function call.
    }

    let closure: Box<F> = Box::new(closure); // Allocate closure on heap.
    let executor: sys::dispatch_function_t = Some(closure_executer::<F>);

    (
        Box::into_raw(closure) as *mut c_void, // Leak the closure.
        executor
    )
}


#[test]
fn test_dispatch_async_f() {
    let label = "Run with native dispatch apis";

    let queue = unsafe {
        sys::dispatch_queue_create(
            label.as_ptr() as *const c_char,
            DISPATCH_QUEUE_SERIAL
        )
    };

    // Allocate the `context` on heap, otherwise the `context` will be
    // freed before `work` is fired and after program goes out of the
    // scope of the unsafe block.
    let context: Box<i32> = Box::new(123);

    extern fn work(leaked_ptr: *mut c_void) {
        let leaked_context = leaked_ptr as *mut i32;

        // Retake the leaked `context`.
        let context = unsafe { Box::from_raw(leaked_context) };
        assert_eq!(context.as_ref(), &123);
        // `context` is released after finishing this function call.
    }

    unsafe {
        sys::dispatch_async_f(
            queue,
            Box::into_raw(context) as *mut c_void, // Leak the `context`.
            Some(work)
        );
    }
}

#[test]
fn test_async_dispatch() {
    let label = "Run with dispatch api wrappers";

    let queue = create_dispatch_queue(
        label,
        DISPATCH_QUEUE_SERIAL
    );

    struct Resource {
        last_touched: Option<u32>,
        touched_count: u32,
    }

    impl Resource {
        fn new() -> Self {
            Resource {
                last_touched: None,
                touched_count: 0,
            }
        }
    }

    let mut resource = Resource::new();

    // Rust compilter doesn't allow a pointer to be passed across threads.
    // A hacky way to do that is to cast the pointer into a value, then
    // the value, which is actually an address, can be copied into threads.
    let resource_ptr = &mut resource as *mut Resource as usize;

    // The following two closures should be executed sequentially.
    async_dispatch(queue, move || {
        let res: &mut Resource = unsafe {
            let ptr = resource_ptr as *mut Resource;
            &mut (*ptr)
        };
        assert_eq!(res as *mut Resource as usize, resource_ptr);
        assert_eq!(res.last_touched, None);
        assert_eq!(res.touched_count, 0);

        res.last_touched = Some(1);
        res.touched_count += 1;
    });

    async_dispatch(queue, move || {
        let res: &mut Resource = unsafe {
            let ptr = resource_ptr as *mut Resource;
            &mut (*ptr)
        };
        assert_eq!(res as *mut Resource as usize, resource_ptr);
        assert!(res.last_touched.is_some());
        assert_eq!(res.last_touched.unwrap(), 1);
        assert_eq!(res.touched_count, 1);

        res.last_touched = Some(2);

        // Make sure the `res.touched_count += 1` is the last instruction of
        // the task since we use `res.touched_count` to check if whether
        // we should release the `resource` and should finish the
        // `test_async_dispatch`(see below). Any instructions after
        // `res.touched_count += 1` may be executed after `test_async_dispatch`.
        res.touched_count += 1;
        // e.g., the following code may cause crash since this instruction may be
        // executed after `resource` is freed.
        // println!("crash > {:?} @ {:p}", res, res);
    });

    // Make sure the resource won't be freed before the tasks are finished.
    while resource.touched_count < 2 {};
    assert!(resource.last_touched.is_some());
    assert_eq!(resource.last_touched.unwrap(), 2);
}
