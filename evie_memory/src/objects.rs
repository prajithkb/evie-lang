use std::{cell::RefCell, fmt::Display, ptr::NonNull, rc::Rc};

use derive_new::new;
use evie_common::Writer;

use crate::chunk::Chunk;

pub type Shared<T> = Rc<RefCell<T>>;
pub fn shared<T>(v: T) -> Shared<T> {
    Rc::new(RefCell::new(v))
}

/// A runtime 'Value' in Evie. This is only data structure exposed to the runtime.
/// It is a combination of primitives such as 'Boolean' and complex data structures like 'Object'
/// See [Object] for more about objects.
#[derive(Debug, Clone, Copy)]
pub enum Value {
    /// Nil value (nothing, null in other languages)
    Nil,
    /// Boolean as name suggests
    Boolean(bool),
    /// Numbers are represented as [f64]
    Number(f64),
    /// See [Object] for more about objects.
    Object(Object),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => f.write_str("Nil"),
            Value::Boolean(b) => f.write_str(&b.to_string()),
            Value::Number(n) => f.write_str(&n.to_string()),
            Value::Object(o) => f.write_str(&o.to_string()),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Nil
    }
}

pub fn print_value(value: &Value, writer: Writer) {
    write!(writer, "{}", value).expect("Write failed");
}

/// Objects are heap allocated and are garbage collected.
/// See [super::ObjectAllocator] for more details how to `alloc` and `free` objects
#[derive(Debug, Clone, Copy)]
pub enum Object {
    String(GCObjectOf<Box<str>>),
    Function(GCObjectOf<Function>),
    Closure(GCObjectOf<Closure>),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(s) => f.write_str(&s.as_ref().to_string()),
            Object::Function(fun) => f.write_str(&fun.as_ref().to_string()),
            Object::Closure(c) => f.write_str(&c.as_ref().to_string()),
        }
    }
}

impl std::hash::Hash for GCObjectOf<Box<str>> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.reference.hash(state);
    }
}

impl PartialEq for GCObjectOf<Box<str>> {
    fn eq(&self, other: &Self) -> bool {
        self.reference == other.reference
    }
}

impl Eq for GCObjectOf<Box<str>> {}

#[derive(Debug, Clone, Copy, new)]
pub struct Closure {
    pub function: GCObjectOf<Function>,
    pub upvalues: GCObjectOf<Vec<GCObjectOf<Upvalue>>>,
}

impl Display for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.function.as_ref().to_string())
    }
}
/// Functions in Evie
#[derive(Debug, Clone)]
pub enum Function {
    UserDefined(UserDefinedFunction),
    Native(String),
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::UserDefined(u) => f.write_str(&u.to_string()),
            Function::Native(_) => todo!(),
        }
    }
}
/// A user defined function
#[derive(Debug, Clone, new)]
pub struct UserDefinedFunction {
    pub name: Option<GCObjectOf<Box<str>>>,
    pub chunk: GCObjectOf<Chunk>,
    pub arity: usize,
    pub upvalue_count: usize,
}

impl Display for UserDefinedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.name {
            f.write_str(&format!("<fn {}>", name.as_ref()))
        } else {
            f.write_str("<fn script>")
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Upvalue {
    pub location: Location,
}

#[derive(Debug, Clone, Copy)]
pub enum Location {
    Stack(usize),
    Heap(GCObjectOf<Value>),
}

impl Upvalue {
    pub fn new_with_location(location: Location) -> Self {
        Upvalue { location }
    }
}

/// Metadata related to an [Object]. Used mainly for GC.
/// See
#[derive(Default, Debug, Clone, Copy, new)]
pub struct ObjectMetada {
    pub is_marked: bool,
    pub next: Option<NonNull<ObjectMetada>>,
}

/// A Managed Object (garbage collected) in Evie. It contains the metadata and a pointer to the actual object.
/// This is created and destroyed using [super::ObjectAllocator]
pub struct GCObjectOf<T> {
    //
    pub metadata: NonNull<ObjectMetada>,
    pub reference: NonNull<T>,
}

impl<T> std::fmt::Debug for GCObjectOf<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GCObjectOf")
            .field("metadata", &self.metadata)
            .field("reference", &self.reference)
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}

impl<T> GCObjectOf<T> {
    pub(crate) fn new(metadata: NonNull<ObjectMetada>, reference: NonNull<T>) -> Self {
        GCObjectOf {
            metadata,
            reference,
        }
    }

    pub fn as_ptr(&self) -> *const T {
        self.reference.as_ptr()
    }

    /// # Safety
    /// Caller should ensure that `orig` is a value GCObject
    pub unsafe fn map_ref<U, F>(orig: GCObjectOf<T>, f: F) -> GCObjectOf<U>
    where
        F: FnOnce(&T) -> &U,
    {
        let ptr: *const U = f(orig.reference.as_ref()) as *const U;
        GCObjectOf {
            metadata: orig.metadata,
            reference: NonNull::new(ptr as *mut U).expect("Null pointer"),
        }
    }

    /// # Safety
    /// Caller should ensure that `orig` is a value GCObject
    pub unsafe fn map_mut<U, F>(mut orig: GCObjectOf<T>, f: F) -> GCObjectOf<U>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        let ptr: *mut U = f(orig.reference.as_mut());
        GCObjectOf {
            metadata: orig.metadata,
            reference: NonNull::new(ptr).expect("Null pointer"),
        }
    }
}

impl<T> Clone for GCObjectOf<T> {
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata,
            reference: self.reference,
        }
    }
}

impl<T> AsRef<T> for GCObjectOf<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.reference.as_ref() }
    }
}

impl<T> AsMut<T> for GCObjectOf<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.reference.as_mut() }
    }
}

impl<T> Copy for GCObjectOf<T> {}

#[cfg(test)]
mod tests {
    use crate::objects::{Object, Value};

    #[test]
    fn value_size() {
        assert_eq!(32, std::mem::size_of::<Value>());
        assert_eq!(24, std::mem::size_of::<Object>());
    }
}
