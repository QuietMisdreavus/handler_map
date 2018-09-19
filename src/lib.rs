//! Crate which contains a "handler map", a structure that maps message types with "handlers" that
//! can be called with them.
//!
//! The focal point of this crate is the `HandlerMap` type, which stores information about
//! functions which receive various types. This can be used to encode event handlers, message
//! handlers, or other situations where you want to dynamically select a function to call based on
//! the data it receives.
//!
//! To register a handler, pass it to `insert`:
//!
//! ```rust
//! use handler_map::HandlerMap;
//!
//! /// Message which prints to the console when received.
//! struct MyMessage;
//!
//! fn handle(_: MyMessage) {
//!     println!("got your message!");
//! }
//!
//! let mut map = HandlerMap::new();
//! map.insert(handle);
//! ```
//!
//! This adds the handler to the map so that it can be `call`ed later on:
//!
//! ```rust
//! # use handler_map::HandlerMap;
//!
//! # /// Message which prints to the console when received.
//! # struct MyMessage;
//!
//! # fn handle(_: MyMessage) {
//! #     println!("got your message!");
//! # }
//!
//! # let mut map = HandlerMap::new();
//! # map.insert(handle);
//! map.call(MyMessage);
//! ```
//!
//! The map can also take closures, as long as they implement `Fn` and don't capture any references
//! to their environment:
//!
//! ```rust
//! use handler_map::HandlerMap;
//! use std::rc::Rc;
//! use std::cell::Cell;
//!
//! /// Message which increments an accumulator when received.
//! struct MyMessage;
//!
//! let mut map = HandlerMap::new();
//! let acc = Rc::new(Cell::new(0));
//! {
//!     let acc = acc.clone();
//!     map.insert(move |_: MyMessage| {
//!         acc.set(acc.get() + 1);
//!     });
//! }
//!
//! // call the handler a few times to increment the counter
//! map.call(MyMessage);
//! map.call(MyMessage);
//! map.call(MyMessage);
//!
//! assert_eq!(acc.get(), 3);
//! ```
//!
//! `call` can take a message of any type, even if that type hasn't been registered. It returns a
//! `bool` representing whether a handler was called. If a handler for that type has been
//! registered in the map, it returns `true`; otherwise, it returns `false`.

mod boxfn;

use std::any::{Any, TypeId};
use std::collections::HashMap;

use boxfn::{BoxFn, Opaque};

/// Struct that maps types with functions or closures that can receive them.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Default)]
pub struct HandlerMap(HashMap<TypeId, BoxFn<'static, Opaque>>);

impl HandlerMap {
    /// Creates a new map with no handlers.
    pub fn new() -> HandlerMap {
        Self::default()
    }

    /// Insert a new callable into the map.
    pub fn insert<T: Any, F: Fn(T) + 'static>(&mut self, callable: F) {
        let ptr: BoxFn<'static, T, F> = Box::new(callable).into();
        let ptr: BoxFn<'static, Opaque> = ptr.erase().erase_arg();
        let id = TypeId::of::<T>();

        self.0.insert(id, ptr);
    }

    /// Calls the callable with the given message, returning whether the callable was registered.
    pub fn call<T: Any>(&self, msg: T) -> bool {
        let id = TypeId::of::<T>();
        if let Some(act) = self.0.get(&id) {
            unsafe { act.call_erased(msg); }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HandlerMap;

    #[test]
    fn it_works() {
        struct MyMessage;
        fn respond(_: MyMessage) {}

        let mut map = HandlerMap::new();
        map.insert(respond);

        assert!(map.call(MyMessage));
    }

    #[test]
    fn no_handler() {
        struct MyMessage;

        let map = HandlerMap::new();

        assert!(!map.call(MyMessage));
    }

    #[test]
    fn handler_is_called() {
        use std::sync::Arc;
        use std::sync::atomic::AtomicUsize;
        use std::sync::atomic::Ordering::SeqCst;

        let mut map = HandlerMap::new();

        struct FancyCaller;
        let acc = Arc::new(AtomicUsize::new(0));
        {
            let acc = acc.clone();
            map.insert(move |_: FancyCaller| {
                acc.fetch_add(1, SeqCst);
            });
        }

        map.call(FancyCaller);
        map.call(FancyCaller);
        map.call(FancyCaller);

        assert_eq!(acc.load(SeqCst), 3);
    }
}
