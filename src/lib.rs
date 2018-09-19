use std::any::{Any, TypeId};
use std::collections::HashMap;

mod boxfn;

use boxfn::{BoxFn, Opaque};

/// Struct that maps types with functions or closures that can receive them.
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
