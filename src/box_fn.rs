// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was
// not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.

//! Internal implementation of a boxed `Fn(T)` function pointer/closure that can have its argument
//! type (and its own type) erased.
//!
//! These types aren't really meant to be used outside this crate, as they rely on assumptions
//! based on the uses of `TypeId` in `HandlerMap`.

use std;
use std::marker::PhantomData;

/// Opaque handle type that represents an erased type parameter.
///
/// If extern types were stable, this could be implemented as `extern { pub type Opaque; }` but
/// until then we can use this.
///
/// Care should be taken that we don't use a concrete instance of this. It should only be used
/// through a reference, so we can maintain something else's lifetime.
pub(crate) struct Opaque(());

/// Collection of functions representing the operations we want to use on a boxed closure, namely,
/// calling it and dropping it.
struct BoxFnVtable<A: ?Sized, F: ?Sized = Opaque> {
    call: fn(&F, A),
    drop_box: unsafe fn(*mut F),
}

/// Custom handle to a boxed closure, allowing for preserving or erasing the closure or argument
/// types.
///
/// To create an instance of `BoxFn`, convert an instance of `Box<F: Fn(A)>` using
/// `From`/`Into`.
pub(crate) struct BoxFn<'a, A: 'a + ?Sized, F: 'a + ?Sized = Opaque> {
    data: &'a mut F,
    vtable: &'a BoxFnVtable<A, F>,
    _invariant: PhantomData<&'a mut &'a ()>,
}

impl<'a, A: ?Sized, F: ?Sized> Drop for BoxFn<'a, A, F> {
    fn drop(&mut self) {
        unsafe {
            (self.vtable.drop_box)(self.data);
        }
    }
}

impl<'a, A, F: Fn(A) + 'a> From<Box<F>> for BoxFn<'a, A, F> {
    fn from(f: Box<F>) -> Self {
        unsafe fn drop_box<F>(f: *mut F) {
            drop(Box::from_raw(f));
        }
        fn call<F: Fn(A), A>(f: &F, arg: A) {
            f(arg)
        }
        BoxFn {
            data: unsafe { &mut *Box::into_raw(f) },
            vtable: &BoxFnVtable {
                call,
                drop_box,
            },
            _invariant: PhantomData,
        }
    }
}

impl<'a, A, F> BoxFn<'a, A, F> {
    /// Erases the closure type, converting `BoxFn<'a, T, F>` to `BoxFn<'a, T, Opaque>`.
    pub fn erase(self) -> BoxFn<'a, A> {
        unsafe {
            let data = &mut *(self.data as *mut _ as *mut Opaque);
            let vtable = &*(self.vtable as *const _ as *const BoxFnVtable<A>);
            std::mem::forget(self);
            BoxFn {
                data,
                vtable,
                _invariant: PhantomData,
            }
        }
    }
}

impl<'a, A> BoxFn<'a, A> {
    /// Erases the argument type, converting `BoxFn<'a, A, Opaque>` to `BoxFn<'a, Opaque, Opaque>`.
    pub fn erase_arg(self) -> BoxFn<'a, Opaque> {
        unsafe {
            let data = &mut *(self.data as *mut _);
            let vtable = &*(self.vtable as *const _ as *const BoxFnVtable<Opaque>);
            std::mem::forget(self);
            BoxFn {
                data,
                vtable,
                _invariant: PhantomData,
            }
        }
    }
}

impl<'a, A, F: ?Sized> BoxFn<'a, A, F> {
    /// Calls the closure with the given argument.
    ///
    /// This is the equivalent of calling a `Box<Fn(T)>`, but since the `Fn` trait is unstable to
    /// implement, we have this function.
    #[allow(dead_code)] // not used in this crate, but added for completeness
    pub(crate) fn call(&self, arg: A) {
        (self.vtable.call)(self.data, arg);
    }
}

impl<'a> BoxFn<'a, Opaque> {
    /// Calls an erased closure with the given argument.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the argument type given to this function is actually the type that
    /// was used to originally create this `BoxFn` before its types were erased. Failure to uphold
    /// this constraint can cause the function to be called with invalid data.
    pub(crate) unsafe fn call_erased<A: 'a>(&self, arg: A) {
        std::mem::transmute::<
            fn(&Opaque, Opaque),
            fn(&Opaque, A),
        >(self.vtable.call)(self.data, arg);
    }
}
