// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was
// not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.

use std;
use std::marker::PhantomData;

// Kinda important that you can't make values of these.
// But unstable, so we shim it with something else.
// extern { pub type Opaque; }
pub struct Opaque(());

struct BoxFnVtable<A: ?Sized, F: ?Sized = Opaque> {
	call: fn(&F, A),
	drop_box: unsafe fn(*mut F),
}

pub struct BoxFn<'a, A: 'a + ?Sized, F: 'a + ?Sized = Opaque> {
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

impl<'a, A, F: Fn(A)> From<Box<F>> for BoxFn<'a, A, F> {
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
	#[allow(dead_code)]
	pub fn call(&self, arg: A) {
		(self.vtable.call)(self.data, arg);
	}
}

impl<'a> BoxFn<'a, Opaque> {
	pub unsafe fn call_erased<A: 'a>(&self, arg: A) {
		std::mem::transmute::<
			fn(&Opaque, Opaque),
			fn(&Opaque, A),
		>(self.vtable.call)(self.data, arg);
	}
}
