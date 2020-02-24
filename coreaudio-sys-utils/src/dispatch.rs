use coreaudio_sys::*;

use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;

// Queue: A wrapper around `dispatch_queue_t`.
// ------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Queue(dispatch_queue_t);

impl Queue {
    pub fn new(label: &'static str) -> Self {
        Self(create_dispatch_queue(label, DISPATCH_QUEUE_SERIAL))
    }

    pub fn run_async<F>(&self, work: F)
    where
        F: Send + FnOnce(),
    {
        async_dispatch(self.0, work);
    }

    pub fn run_sync<F>(&self, work: F)
    where
        F: Send + FnOnce(),
    {
        sync_dispatch(self.0, work);
    }

    // This will release the inner `dispatch_queue_t` asynchronously.
    fn release(&self) {
        release_dispatch_queue(self.0);
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        self.release();
    }
}

impl Clone for Queue {
    fn clone(&self) -> Self {
        retain_dispatch_queue(self.0);
        Self(self.0)
    }
}

// Low-level Grand Central Dispatch (GCD) APIs
// ------------------------------------------------------------------------------------------------
const DISPATCH_QUEUE_SERIAL: dispatch_queue_attr_t = ptr::null_mut::<dispatch_queue_attr_s>();

fn create_dispatch_queue(
    label: &'static str,
    queue_attr: dispatch_queue_attr_t,
) -> dispatch_queue_t {
    let label = CString::new(label).unwrap();
    let c_string = label.as_ptr();
    unsafe { dispatch_queue_create(c_string, queue_attr) }
}

fn release_dispatch_queue(queue: dispatch_queue_t) {
    // TODO: This is incredibly unsafe. Find another way to release the queue.
    unsafe {
        dispatch_release(mem::transmute::<dispatch_queue_t, dispatch_object_t>(queue));
    }
}

fn retain_dispatch_queue(queue: dispatch_queue_t) {
    // TODO: This is incredibly unsafe. Find another way to retain the queue.
    unsafe {
        dispatch_retain(mem::transmute::<dispatch_queue_t, dispatch_object_t>(queue));
    }
}

fn async_dispatch<F>(queue: dispatch_queue_t, work: F)
where
    F: Send + FnOnce(),
{
    let (closure, executor) = create_closure_and_executor(work);
    unsafe {
        dispatch_async_f(queue, closure, executor);
    }
}

fn sync_dispatch<F>(queue: dispatch_queue_t, work: F)
where
    F: Send + FnOnce(),
{
    let (closure, executor) = create_closure_and_executor(work);
    unsafe {
        dispatch_sync_f(queue, closure, executor);
    }
}

// Return an raw pointer to a (unboxed) closure and an executor that
// will run the closure (after re-boxing the closure) when it's called.
fn create_closure_and_executor<F>(closure: F) -> (*mut c_void, dispatch_function_t)
where
    F: FnOnce(),
{
    extern "C" fn closure_executer<F>(unboxed_closure: *mut c_void)
    where
        F: FnOnce(),
    {
        // Retake the leaked closure.
        let closure = unsafe { Box::from_raw(unboxed_closure as *mut F) };
        // Execute the closure.
        (*closure)();
        // closure is released after finishing this function call.
    }

    let closure = Box::new(closure); // Allocate closure on heap.
    let executor: dispatch_function_t = Some(closure_executer::<F>);

    (
        Box::into_raw(closure) as *mut c_void, // Leak the closure.
        executor,
    )
}

#[test]
fn run_tasks_in_order() {
    let mut visited = Vec::<u32>::new();

    // Rust compilter doesn't allow a pointer to be passed across threads.
    // A hacky way to do that is to cast the pointer into a value, then
    // the value, which is actually an address, can be copied into threads.
    let ptr = &mut visited as *mut Vec<u32> as usize;

    fn visit(v: u32, visited_ptr: usize) {
        let visited = unsafe { &mut *(visited_ptr as *mut Vec<u32>) };
        visited.push(v);
    };

    let queue = Queue::new("Run tasks in order");

    queue.run_sync(move || visit(1, ptr));
    queue.run_sync(move || visit(2, ptr));
    queue.run_async(move || visit(3, ptr));
    queue.run_async(move || visit(4, ptr));
    // Call sync here to block the current thread and make sure all the tasks are done.
    queue.run_sync(move || visit(5, ptr));

    assert_eq!(visited, vec![1, 2, 3, 4, 5]);
}
