use std::{fmt::Display, ptr::NonNull};

use derive_new::new;
use evie_common::Writer;

use crate::chunk::Chunk;

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
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(s) => f.write_str(&s.as_ref().to_string()),
            Object::Function(fun) => f.write_str(&fun.as_ref().to_string()),
        }
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

/// Metadata related to an [Object]. Used mainly for GC.
/// See
#[derive(Default, Debug, Clone, Copy, new)]
pub struct ObjectMetada {
    pub is_marked: bool,
    pub next: Option<NonNull<ObjectMetada>>,
}

/// A Managed Object (garbage collected) in Evie. It contains the metadata and a pointer to the actual object.
/// This is created and destroyed using [super::ObjectAllocator]
#[derive(Debug)]
pub struct GCObjectOf<T> {
    //
    pub metadata: ObjectMetada,
    pub reference: NonNull<T>,
}

impl<T> GCObjectOf<T> {
    pub(crate) fn new(metadata: ObjectMetada, reference: NonNull<T>) -> Self {
        GCObjectOf {
            metadata,
            reference,
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
