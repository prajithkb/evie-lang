//! The memory crate
use core::fmt::Debug;
use std::{cell::Cell, ptr::NonNull};

use objects::{GCObjectOf, ObjectMetada};

pub mod chunk;
pub mod objects;

/// A simple [objects::GCObjectOf] allocator
#[derive(Default)]
pub struct ObjectAllocator {
    bytes_allocated: Cell<usize>,
}

impl ObjectAllocator {
    /// A new instance of [ObjectAllocator]
    pub fn new() -> Self {
        ObjectAllocator {
            bytes_allocated: Cell::new(0),
        }
    }

    /// Creates an instance of GCObject
    pub fn alloc<T>(&self, object: T) -> GCObjectOf<T>
    where
        T: Debug,
    {
        let v = Box::new(object);
        self.increment_allocated_bytes_by(std::mem::size_of::<T>());
        let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(v)) };
        GCObjectOf::new(ObjectMetada::default(), ptr)
    }

    /// # Safety
    /// The caller should ensure that the object was note previously de allocated.
    /// This can cause double free.
    pub unsafe fn free<T>(&self, object_of: GCObjectOf<T>)
    where
        T: Debug,
    {
        // Gets freed when the object is dropped
        let _object = Box::from_raw(object_of.reference.as_ptr());
        let bytes_to_deallocate = std::mem::size_of::<T>();
        debug_assert!(self.bytes_allocated.get() >= bytes_to_deallocate);
        self.decrement_allocated_bytes_by(bytes_to_deallocate);
    }

    /// Returns the number of bytes allocated so far
    pub fn bytes_allocated(&self) -> usize {
        self.bytes_allocated.get()
    }

    fn increment_allocated_bytes_by(&self, bytes: usize) {
        let prev_size = self.bytes_allocated.get();
        self.bytes_allocated.set(prev_size + bytes);
    }

    fn decrement_allocated_bytes_by(&self, bytes: usize) {
        let prev_size = self.bytes_allocated.get();
        self.bytes_allocated.set(prev_size - bytes);
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::{f64::EPSILON, time::Instant};

    use crate::{
        chunk::Chunk,
        objects::{Function, GCObjectOf, Object, UserDefinedFunction, Value},
        ObjectAllocator,
    };

    #[test]
    fn allocation_test() {
        let managed_objects = ObjectAllocator::new();
        let name: GCObjectOf<Box<str>> = managed_objects.alloc("object".into());
        assert_eq!(
            std::mem::size_of::<Box<str>>(),
            managed_objects.bytes_allocated()
        );
        let chunk = managed_objects.alloc(Chunk::new());
        let function = managed_objects.alloc(Function::UserDefined(UserDefinedFunction::new(
            Some(name),
            chunk,
            0,
            0,
        )));
        assert_eq!(
            std::mem::size_of::<Box<str>>()
                + std::mem::size_of::<Function>()
                + std::mem::size_of::<Chunk>(),
            managed_objects.bytes_allocated()
        );
        unsafe { managed_objects.free(function) };
        assert_eq!(
            std::mem::size_of::<Box<str>>() + std::mem::size_of::<Chunk>(),
            managed_objects.bytes_allocated()
        );
        unsafe { managed_objects.free(name) };
        unsafe { managed_objects.free(chunk) };
        assert_eq!(0, managed_objects.bytes_allocated());
    }

    #[test]
    fn timing() {
        let mut objects = ObjectAllocator::new();
        let constants = vec![
            Value::Nil,
            Value::Number(1.0),
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Object(Object::String(objects.alloc("str".into()))),
            Value::Object(Object::String(objects.alloc("stru".into()))),
        ];
        let mut stack = [
            Value::Nil,
            Value::Nil,
            Value::Nil,
            Value::Nil,
            Value::Nil,
            Value::Nil,
            Value::Nil,
            Value::Nil,
            Value::Nil,
            Value::Nil,
            Value::Nil,
            Value::Nil,
        ];

        let start = Instant::now();
        let mut count = 0;
        let operations = 10000000;
        while count < operations {
            for i in 0..constants.len() {
                for j in 0..constants.len() {
                    let a = constants[i];
                    let b = constants[j];
                    stack[0] = a;
                    stack[1] = b;
                    stack[0] = Value::Boolean(value_equals(a, b));
                    count += 1;
                }
            }
        }
        println!(
            "Time for {} operations ={} ms",
            operations,
            start.elapsed().as_millis()
        )
    }
    fn value_equals(l: Value, r: Value) -> bool {
        match (l, r) {
            (Value::Boolean(l), Value::Boolean(r)) => l == r,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(l), Value::Number(r)) => num_equals(l, r),
            (Value::Object(Object::String(l)), Value::Object(Object::String(r))) => {
                l.as_ref() == r.as_ref()
            }
            _ => false,
        }
    }
    fn num_equals(l: f64, r: f64) -> bool {
        (l - r).abs() < EPSILON
    }
}
