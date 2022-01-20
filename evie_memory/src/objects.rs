use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    rc::Rc,
};

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
    /// Strings
    String(GCObjectOf<Box<str>>),
    /// Functions
    Function(GCObjectOf<Function>),
    /// A Closure
    Closure(GCObjectOf<Closure>),
    /// A Class
    Class(GCObjectOf<Class>),
    /// An Instance
    Instance(GCObjectOf<Instance>),
    /// A Bound Method with an instance as a receiver
    BoundMethod(GCObjectOf<Instance>, GCObjectOf<Closure>),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(s) => f.write_str(&s.as_ref().to_string()),
            Object::Function(fun) => f.write_str(&fun.as_ref().to_string()),
            Object::Closure(c) => f.write_str(&c.as_ref().to_string()),
            Object::Class(c) => f.write_str(&c.as_ref().to_string()),
            Object::Instance(i) => f.write_str(&i.as_ref().to_string()),
            Object::BoundMethod(i, c) => f.write_str(&format!(
                "[{} bound to instance of {}]",
                *c.function, *i.class.name
            )),
        }
    }
}
impl std::hash::Hash for GCObjectOf<Box<str>> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { self.reference.as_ref().hash(state) }
    }
}

impl PartialEq for GCObjectOf<Box<str>> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.reference.as_ref() == other.reference.as_ref() }
    }
}

impl Eq for GCObjectOf<Box<str>> {}

/// Closure Object for Evie
#[derive(Debug, Clone, Copy, new)]
pub struct Closure {
    pub function: GCObjectOf<Function>,
    /// This is the magic that makes a closure work
    pub upvalues: GCObjectOf<Vec<GCObjectOf<Upvalue>>>,
}

impl Display for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.function.as_ref().to_string())
    }
}
/// Functions in Evie
#[derive(Debug, Clone, Copy)]
pub enum Function {
    /// User defined functions are the ones defined in Evie
    UserDefined(UserDefinedFunction),
    /// Native access, e.g. File/Socket access
    Native(NativeFunction),
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::UserDefined(u) => f.write_str(&u.to_string()),
            Function::Native(n) => f.write_str(&n.to_string()),
        }
    }
}
/// An User defined function
#[derive(Debug, Clone, new, Copy)]
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

/// Native function is  basically a function pointer
pub type NativeFn = fn(Vec<Value>) -> Value;

/// Native functions are functions implemented in Rust
#[derive(Clone, new, Copy)]
pub struct NativeFunction {
    pub name: GCObjectOf<Box<str>>,
    pub arity: usize,
    pub function: NativeFn,
}

impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeFunction")
            .field("name", self.name.as_ref())
            .finish()
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("native <fn {}>", self.name.as_ref()))
    }
}

impl NativeFunction {
    pub fn call(&self, arguments: Vec<Value>) -> Value {
        let function = self.function;
        function(arguments)
    }
}

/// A Class in Evie
#[derive(Debug, Clone, Copy)]
pub struct Class {
    /// Name of the class
    pub name: GCObjectOf<Box<str>>,
    /// Methods defined by this class
    pub methods: GCObjectOf<HashMap<GCObjectOf<Box<str>>, GCObjectOf<Closure>>>,
}

impl Class {
    pub fn new(
        name: GCObjectOf<Box<str>>,
        methods: GCObjectOf<HashMap<GCObjectOf<Box<str>>, GCObjectOf<Closure>>>,
    ) -> Self {
        Class { name, methods }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("<class {}>", self.name.as_ref()))
    }
}

/// An Instance in Evie
#[derive(Debug, Clone)]
pub struct Instance {
    /// Refers the class
    pub class: GCObjectOf<Class>,
    /// The fields held by this instance
    pub fields: GCObjectOf<HashMap<GCObjectOf<Box<str>>, Value>>,
}

impl Instance {
    pub fn new(
        class: GCObjectOf<Class>,
        fields: GCObjectOf<HashMap<GCObjectOf<Box<str>>, Value>>,
    ) -> Self {
        Instance { class, fields }
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("<instance of {}>", &*self.class.name))
    }
}

/// Captured value for a Closure (the magic that makes a Closure work)
#[derive(Debug, Clone, Copy)]
pub struct Upvalue {
    pub location: Location,
}

/// Holds the location of the captured value
#[derive(Debug, Clone, Copy)]
pub enum Location {
    /// Index on the stack
    Stack(usize),
    /// Allocated on the heap, when moved from the stack
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
    /// Used in GC for mark and sweep
    pub is_marked: bool,
    /// Pointer to the next object
    pub next: Option<NonNull<ObjectMetada>>,
}

/// A Managed Object (garbage collected) in Evie. It contains the metadata and a pointer to the actual object.
/// This is created and destroyed using [super::ObjectAllocator]
pub struct GCObjectOf<T> {
    /// Metadata for this object, used for mark and sweep
    pub metadata: NonNull<ObjectMetada>,
    /// Pointer to the heap allocated object `T`
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

impl<T> Deref for GCObjectOf<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for GCObjectOf<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> Copy for GCObjectOf<T> {}

#[cfg(test)]
mod tests {
    use crate::objects::{Object, Value};

    #[test]
    fn value_size() {
        assert_eq!(48, std::mem::size_of::<Value>());
        assert_eq!(40, std::mem::size_of::<Object>());
    }
}
