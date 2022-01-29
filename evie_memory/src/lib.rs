//! Defines the data structures that are used across evie.
//! Also defines the memory management (Garbage Collection) for evie
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ptr::NonNull,
    rc::Rc,
};

use objects::{GCObjectOf, Object, ObjectType};

pub mod chunk;
pub mod objects;

type Mutable<T> = Rc<RefCell<T>>;

#[derive(Debug)]
struct InternedValue(GCObjectOf<Box<str>>, Option<GCObjectOf<Object>>);

/// A simple [objects::GCObjectOf] allocator.
/// Internally uses [Box] to create/destroy objects
pub struct ObjectAllocator {
    bytes_allocated: Cell<usize>,
    interned_strings: Mutable<HashMap<Box<str>, InternedValue>>,
}

impl ObjectAllocator {
    /// A new instance of [ObjectAllocator]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ObjectAllocator {
            bytes_allocated: Cell::new(0),
            interned_strings: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Creates an instance of GCObject
    pub fn alloc<T>(&self, object: T) -> GCObjectOf<T> {
        let v = Box::new(object);
        let bytes_allocated = std::mem::size_of::<T>();
        self.increment_allocated_bytes_by(bytes_allocated);
        #[cfg(feature = "trace_enabled")]
        evie_common::trace!(
            "Allocated {} bytes for {}",
            std::mem::size_of::<T>(),
            std::any::type_name::<T>()
        );
        let ptr = unsafe { NonNull::new_unchecked(Box::into_raw(v)) };
        GCObjectOf::new(ptr)
    }

    /// Creates an interned instance of GCObject<Box<str>>
    pub fn alloc_interned_str<T: AsRef<str>>(&self, object: T) -> GCObjectOf<Box<str>> {
        let object = object.as_ref().to_string().into_boxed_str();
        let v = self.interned_strings.borrow();
        if let Some(v) = v.get(&object) {
            (*v).0
        } else {
            drop(v);
            let string = self.alloc(object.clone());
            let mut v = (*self.interned_strings).borrow_mut();
            v.insert(object, InternedValue(string, None));
            string
        }
    }

    /// Creates an interned instance of GCObject<Object>
    pub fn alloc_interned_object(&self, object: GCObjectOf<Box<str>>) -> GCObjectOf<Object> {
        let mut v = self.interned_strings.borrow_mut();
        if let Some(v) = v.get_mut(object.as_ref()) {
            if let Some(v) = v.1 {
                v
            } else {
                let o = Object::new_gc_object(ObjectType::String(v.0), self);
                v.1 = Some(o);
                o
            }
        } else {
            panic!("BUG: String '{}' is not interned", object.as_ref());
        }
    }

    /// # Safety
    /// The caller should ensure that the object was note previously de allocated.
    /// This can cause double free.
    pub unsafe fn free<T>(&self, object_of: GCObjectOf<T>) {
        {
            // Gets freed when the object is dropped
            Box::from_raw(object_of.reference.as_ptr());
        }
        let bytes_to_deallocate = std::mem::size_of::<T>();
        #[cfg(feature = "trace_enabled")]
        evie_common::trace!(
            "Deallocated {} bytes for {}",
            std::mem::size_of::<T>(),
            std::any::type_name::<T>()
        );
        assert!(self.bytes_allocated.get() >= bytes_to_deallocate);
        self.decrement_allocated_bytes_by(bytes_to_deallocate);
    }

    /// Returns the number of bytes allocated so far
    pub fn bytes_allocated(&self) -> usize {
        self.bytes_allocated.get()
    }

    fn increment_allocated_bytes_by(&self, bytes_allocated: usize) {
        self.bytes_allocated
            .set(self.bytes_allocated() + bytes_allocated);
    }

    fn decrement_allocated_bytes_by(&self, bytes: usize) {
        self.bytes_allocated.set(self.bytes_allocated() - bytes);
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::{f64::EPSILON, time::Instant};

    use crate::{
        chunk::Chunk,
        objects::{Function, GCObjectOf, Object, ObjectType, Tag, UserDefinedFunction},
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
    fn timing_non_nan_boxed_value() {
        use crate::objects::non_nan_boxed::Value;

        #[inline(always)]
        fn value_equals(l: Value, r: Value) -> bool {
            if l.is_bool() && r.is_bool() {
                return l.as_bool() == r.as_bool();
            } else if l.is_nil() && r.is_nil() {
                return true;
            } else if l.is_number() && r.is_number() {
                return num_equals(l.as_number(), r.as_number());
            } else if l.is_object() && r.is_object() {
                match (l.as_object().object_type, r.as_object().object_type) {
                    (ObjectType::String(l), ObjectType::String(r)) => {
                        return std::ptr::eq(l.as_ptr(), r.as_ptr()) || l == r
                    }
                    _ => return false,
                }
            }
            false
        }
        #[inline(always)]
        fn num_equals(l: f64, r: f64) -> bool {
            (l - r).abs() < EPSILON
        }

        let mut objects = ObjectAllocator::new();
        let constants = vec![
            Value::number(1.0),
            Value::bool(true),
            Value::bool(false),
            Value::object(Object::new_gc_object(
                ObjectType::String(objects.alloc("str".into())),
                &objects,
            )),
            Value::object(Object::new_gc_object(
                ObjectType::String(objects.alloc("stru".into())),
                &objects,
            )),
        ];
        let mut stack = [Value::nil(); 10];

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
                    stack[0] = Value::bool(value_equals(a, b));
                    count += 1;
                }
            }
        }
        println!(
            "Time for non nan boxed; {} operations ={} ms",
            operations,
            start.elapsed().as_millis()
        )
    }

    #[test]
    fn timing_nan_boxed_value() {
        use crate::objects::nan_boxed::Value;
        #[inline(always)]
        fn value_equals(l: Value, r: Value) -> bool {
            l == r
        }

        let mut objects = ObjectAllocator::new();
        let str: GCObjectOf<Box<str>> = objects.alloc_interned_str("str");
        let stru: GCObjectOf<Box<str>> = objects.alloc_interned_str("stru");
        let constants = vec![
            Value::number(1.0),
            Value::bool(true),
            Value::bool(false),
            Value::object(objects.alloc_interned_object(str)),
            Value::object(objects.alloc_interned_object(stru)),
        ];
        let mut stack = [Value::nil(); 10];

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
                    stack[0] = Value::bool(value_equals(a, b));
                    count += 1;
                }
            }
        }
        println!(
            "Time for nan_boxed; {} operations ={} ms",
            operations,
            start.elapsed().as_millis()
        )
    }
}
