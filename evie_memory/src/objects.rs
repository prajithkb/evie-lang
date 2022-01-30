use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

#[cfg(feature = "nan_boxed")]
use crate::objects::nan_boxed::Value;
#[cfg(not(feature = "nan_boxed"))]
use crate::objects::non_nan_boxed::Value;
use crate::{cache::Cache, chunk::Chunk, ObjectAllocator};
use derive_new::new;
use evie_common::{bail, Writer};
pub mod nan_boxed {
    // Bit Flags
    pub(crate) const QNAN_BIT_FLAG: usize = 0x7ffc000000000000;
    pub(crate) const SIGN_BIT_FLAG: usize = 0x8000000000000000;
    pub(crate) const NIL_BIT_FLAG: usize = 1; // 01.
    pub(crate) const FALSE_BIT_FLAG: usize = 2; // 10.
    pub(crate) const TRUE_BIT_FLAG: usize = 3; // 11.

    // Values
    pub(crate) const NIL: Value = Value(QNAN_BIT_FLAG | NIL_BIT_FLAG);
    pub(crate) const FALSE: Value = Value(QNAN_BIT_FLAG | FALSE_BIT_FLAG);
    pub(crate) const TRUE: Value = Value(QNAN_BIT_FLAG | TRUE_BIT_FLAG);

    use super::{GCObjectOf, Object, ValueType};
    use std::fmt::Display;

    /// A word sized value
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Value(pub(crate) usize);

    impl std::fmt::Debug for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let v_type: ValueType = self.into();
            match v_type {
                ValueType::Nil => write!(f, "Nil"),
                ValueType::Boolean => f
                    .debug_struct("Boolean")
                    .field("value", &self.as_bool())
                    .field("binary_representation", &format!("{:#066b}", self.0))
                    .finish(),
                ValueType::Number => f
                    .debug_struct("Number")
                    .field("value", &self.as_number())
                    .field("binary_representation", &format!("{:#066b}", self.0))
                    .finish(),
                ValueType::Object => f
                    .debug_struct("Object")
                    .field("value", &self.as_object())
                    .field("binary_representation", &format!("{:#066b}", self.0))
                    .finish(),
            }
        }
    }

    impl Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let v_type: ValueType = self.into();
            match v_type {
                ValueType::Nil => f.write_str("nil"),
                ValueType::Boolean => f.write_str(&self.as_bool().to_string()),
                ValueType::Number => f.write_str(&self.as_number().to_string()),
                ValueType::Object => f.write_str(&self.as_object().to_string()),
            }
        }
    }

    impl Default for Value {
        fn default() -> Self {
            NIL
        }
    }

    impl Value {
        #[inline(always)]
        pub fn nil() -> Self {
            NIL
        }
        #[inline(always)]
        pub fn bool(b: bool) -> Self {
            if b {
                TRUE
            } else {
                FALSE
            }
        }
        #[inline(always)]
        pub fn number(n: f64) -> Self {
            Value(usize::from_be_bytes(n.to_be_bytes()))
        }
        #[inline(always)]
        pub fn object(o: GCObjectOf<Object>) -> Self {
            Value((o.as_ptr() as usize) | SIGN_BIT_FLAG | QNAN_BIT_FLAG)
        }
        #[inline(always)]
        pub fn to_type(&self) -> ValueType {
            self.into()
        }
        #[inline(always)]
        pub fn is_number(&self) -> bool {
            (self.0 & QNAN_BIT_FLAG) != QNAN_BIT_FLAG
        }
        #[inline(always)]
        pub fn is_bool(&self) -> bool {
            *self == TRUE || *self == FALSE
        }
        #[inline(always)]
        pub fn is_nil(&self) -> bool {
            *self == NIL
        }
        #[inline(always)]
        pub fn is_object(&self) -> bool {
            (self.0 & (QNAN_BIT_FLAG | SIGN_BIT_FLAG)) == (QNAN_BIT_FLAG | SIGN_BIT_FLAG)
        }
        #[inline(always)]
        pub fn as_nil(&self) -> Value {
            if self.is_nil() {
                NIL
            } else {
                panic!("Not nil")
            }
        }
        #[inline(always)]
        pub fn as_bool(&self) -> bool {
            if self.is_bool() {
                *self == TRUE
            } else {
                panic!("Not a boolean")
            }
        }

        #[inline(always)]
        pub fn as_number(&self) -> f64 {
            if self.is_number() {
                f64::from_be_bytes(self.0.to_be_bytes())
            } else {
                panic!("Not a number")
            }
        }
        #[inline(always)]
        pub fn as_object(&self) -> GCObjectOf<Object> {
            let object = self.0 & !(QNAN_BIT_FLAG | SIGN_BIT_FLAG);
            object.try_into().expect("Not an object")
        }
    }

    impl From<&Value> for ValueType {
        fn from(v: &Value) -> Self {
            if v.is_bool() {
                ValueType::Boolean
            } else if v.is_nil() {
                ValueType::Nil
            } else if v.is_number() {
                ValueType::Number
            } else {
                ValueType::Object
            }
        }
    }
}

/// The type of values. The types are named appropriately
pub enum ValueType {
    Nil,
    Boolean,
    Number,
    Object,
}

pub mod non_nan_boxed {
    use super::{GCObjectOf, Object, ValueType};
    use std::fmt::Display;

    /// A runtime 'Value' in Evie. This is the only data structure exposed to the runtime.
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
        Object(GCObjectOf<Object>),
    }

    impl PartialEq for Value {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
                (Self::Number(l0), Self::Number(r0)) => l0 == r0,
                (Self::Object(l0), Self::Object(r0)) => l0.reference == r0.reference,
                _ => core::mem::discriminant(self) == core::mem::discriminant(other),
            }
        }
    }

    impl Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Value::Nil => f.write_str("nil"),
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

    impl Value {
        #[inline(always)]
        pub fn nil() -> Self {
            Value::Nil
        }

        #[inline(always)]
        pub fn bool(b: bool) -> Self {
            Value::Boolean(b)
        }

        #[inline(always)]
        pub fn number(n: f64) -> Self {
            Value::Number(n)
        }

        #[inline(always)]
        pub fn object(o: GCObjectOf<Object>) -> Self {
            Value::Object(o)
        }

        #[inline(always)]
        pub fn to_type(&self) -> ValueType {
            self.into()
        }

        #[inline(always)]
        pub fn is_number(&self) -> bool {
            matches!(self, Value::Number(_))
        }

        #[inline(always)]
        pub fn is_bool(&self) -> bool {
            matches!(self, Value::Boolean(_))
        }

        #[inline(always)]
        pub fn is_nil(&self) -> bool {
            matches!(self, Value::Nil)
        }

        #[inline(always)]
        pub fn is_object(&self) -> bool {
            matches!(self, Value::Object(_))
        }

        #[inline(always)]
        pub fn as_nil(&self) -> Value {
            if self.is_nil() {
                Value::Nil
            } else {
                panic!("Not nil")
            }
        }

        #[inline(always)]
        pub fn as_bool(&self) -> bool {
            if let Value::Boolean(b) = self {
                *b
            } else {
                panic!("Not a boolean")
            }
        }

        #[inline(always)]
        pub fn as_number(&self) -> f64 {
            if let Value::Number(b) = self {
                *b
            } else {
                panic!("Not a number")
            }
        }

        #[inline(always)]
        pub fn as_object(&self) -> GCObjectOf<Object> {
            if let Value::Object(b) = self {
                *b
            } else {
                panic!("Not an Object")
            }
        }
    }
    impl From<&Value> for ValueType {
        fn from(v: &Value) -> Self {
            match v {
                Value::Nil => ValueType::Nil,
                Value::Boolean(_) => ValueType::Boolean,
                Value::Number(_) => ValueType::Number,
                Value::Object(_) => ValueType::Object,
            }
        }
    }
}
#[inline(always)]
pub fn print_value(value: &Value, writer: Writer) {
    write!(writer, "{}", value).expect("Write failed");
}

/// Objects are heap allocated and are garbage collected.
/// See [super::ObjectAllocator] for more details how to `alloc` and `free` objects
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Object {
    /// The [Tag] that helps with GC (mark and sweep algorithm)
    pub gc_tag: Tag,
    /// The [ObjectType] embedded in this Object
    pub object_type: ObjectType,
}

impl Object {
    pub fn new_gc_object(object_type: ObjectType, allocator: &ObjectAllocator) -> GCObjectOf<Self> {
        allocator.alloc(Object {
            gc_tag: Tag::default(),
            object_type,
        })
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.object_type.to_string())
    }
}

/// Objects are heap allocated and are garbage collected.
/// See [super::ObjectAllocator] for more details how to `alloc` and `free` objects
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum ObjectType {
    /// Strings
    String(GCObjectOf<Box<str>>),
    /// Functions
    Function(GCObjectOf<UserDefinedFunction>),
    /// Native Functions (File access socket access etc.)
    NativeFunction(GCObjectOf<NativeFunction>),
    /// A Closure
    Closure(GCObjectOf<Closure>),
    /// A Class
    Class(GCObjectOf<Class>),
    /// An Instance
    Instance(GCObjectOf<Instance>),
    /// A Bound Method with an instance as a receiver
    BoundMethod(GCObjectOf<BoundMethod>),
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::String(s) => f.write_str(&s.to_string()),
            ObjectType::Function(fun) => f.write_str(&fun.to_string()),
            ObjectType::Closure(c) => f.write_str(&c.to_string()),
            ObjectType::Class(c) => f.write_str(&c.to_string()),
            ObjectType::Instance(i) => f.write_str(&i.to_string()),
            ObjectType::BoundMethod(b) => f.write_str(&format!(
                "[{} bound to instance of {}]",
                *b.1.function.name.unwrap(),
                *b.0.class.name
            )),
            ObjectType::NativeFunction(u) => f.write_str(&u.to_string()),
        }
    }
}
impl std::hash::Hash for GCObjectOf<Box<str>> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.reference.hash(state)
    }
}

impl PartialEq for GCObjectOf<Box<str>> {
    fn eq(&self, other: &Self) -> bool {
        self.reference == other.reference
    }
}

impl Eq for GCObjectOf<Box<str>> {}

/// Closure Object for Evie
#[derive(Debug, Clone, Copy, new)]
pub struct Closure {
    pub function: GCObjectOf<UserDefinedFunction>,
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
    /// The name of the function. It is optional becuse the "Main" function does not have a name
    pub name: Option<GCObjectOf<Box<str>>>,
    /// The [Chunk] that holds instructions for this function
    pub chunk: GCObjectOf<Chunk>,
    /// The number of arguments
    pub arity: usize,
    /// The number of upvalues to be captured for this function
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
pub type NativeFn = fn(Vec<Value>, allocator: &ObjectAllocator) -> Value;

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
    pub fn call(&self, arguments: Vec<Value>, allocator: &ObjectAllocator) -> Value {
        let function = self.function;
        function(arguments, allocator)
    }
}

/// A Class in Evie
#[derive(Debug, Clone, Copy)]
pub struct Class {
    /// Name of the class
    pub name: GCObjectOf<Box<str>>,
    /// Methods defined by this class
    pub methods: GCObjectOf<Cache<GCObjectOf<Closure>>>,
}

impl Class {
    pub fn new(
        name: GCObjectOf<Box<str>>,
        methods: GCObjectOf<Cache<GCObjectOf<Closure>>>,
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
    pub fields: GCObjectOf<Cache<Value>>,
}

impl Instance {
    pub fn new(class: GCObjectOf<Class>, fields: GCObjectOf<Cache<Value>>) -> Self {
        Instance { class, fields }
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("<instance of {}>", &*self.class.name))
    }
}

#[derive(Debug)]
/// Struct for BoundMethod
pub struct BoundMethod(pub GCObjectOf<Instance>, pub GCObjectOf<Closure>);

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
pub struct Tag {
    /// Used in GC for mark and sweep
    pub is_marked: bool,
    /// Pointer to the next object
    pub next: Option<NonNull<Tag>>,
}

/// A Managed Object (garbage collected) in Evie. It contains the metadata and a pointer to the actual object.
/// This is created and destroyed using [super::ObjectAllocator]
pub struct GCObjectOf<T> {
    /// Pointer to the heap allocated object `T`
    pub reference: NonNull<T>,
}

impl<T> std::fmt::Debug for GCObjectOf<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GCObjectOf")
            // .field("metadata", &self.metadata)
            .field("reference", &self.reference)
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}

impl<T> TryFrom<usize> for GCObjectOf<T> {
    type Error = evie_common::ErrorKind;
    fn try_from(address: usize) -> std::result::Result<Self, evie_common::ErrorKind> {
        let address = address as *mut T;
        if let Some(ptr) = NonNull::new(address) {
            Ok(GCObjectOf::new(ptr))
        } else {
            bail!("Null pointer")
        }
    }
}

impl<T> GCObjectOf<T> {
    pub(crate) fn new(reference: NonNull<T>) -> Self {
        GCObjectOf { reference }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.reference.as_ptr()
    }
}

impl<T> Clone for GCObjectOf<T> {
    fn clone(&self) -> Self {
        Self {
            // metadata: self.metadata,
            reference: self.reference,
        }
    }
}

impl<T> AsRef<T> for GCObjectOf<T> {
    #[inline(always)]
    fn as_ref(&self) -> &T {
        unsafe { self.reference.as_ref() }
    }
}

impl<T> AsMut<T> for GCObjectOf<T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.reference.as_mut() }
    }
}

impl<T> Deref for GCObjectOf<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for GCObjectOf<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> Copy for GCObjectOf<T> {}

#[cfg(test)]
mod tests {
    use std::f64::EPSILON;

    use crate::{
        objects::{GCObjectOf, Object, ObjectType},
        ObjectAllocator,
    };

    #[test]
    fn value_size() {
        assert_eq!(
            16,
            std::mem::size_of::<crate::objects::non_nan_boxed::Value>()
        );
        assert_eq!(8, std::mem::size_of::<GCObjectOf<Object>>());
        assert_eq!(32, std::mem::size_of::<Object>());
    }

    #[test]
    fn nan_boxed_value_size() {
        assert_eq!(8, std::mem::size_of::<crate::objects::nan_boxed::Value>());
    }

    #[test]
    #[allow(clippy::bool_assert_comparison)]
    fn non_nan_boxed_value_types() {
        use crate::objects::non_nan_boxed::Value;
        assert_eq!(true, Value::bool(true).as_bool());
        assert_eq!(false, Value::bool(false).as_bool());
        assert_eq!(true, Value::bool(false).is_bool());

        assert_eq!(Value::nil(), Value::nil().as_nil());
        assert_eq!(true, Value::nil().is_nil());

        assert_eq!(true, Value::number(1f64).as_number() - 1f64 < EPSILON);
        assert_eq!(true, Value::number(1f64).is_number());

        let allocator = ObjectAllocator::new();

        let object = Object::new_gc_object(
            ObjectType::String(allocator.alloc("hi".to_string().into_boxed_str())),
            &allocator,
        );
        assert_eq!(Value::Object(object), Value::object(object));
        assert_eq!(true, Value::object(object).is_object());
    }

    #[test]
    #[allow(clippy::bool_assert_comparison)]
    fn nan_boxed_value_types() {
        use crate::objects::nan_boxed::Value;
        assert_eq!(true, Value::bool(true).as_bool());
        assert_eq!(false, Value::bool(false).as_bool());
        assert_eq!(true, Value::bool(false).is_bool());

        assert_eq!(Value::nil(), Value::nil().as_nil());
        assert_eq!(true, Value::nil().is_nil());

        assert_eq!(true, Value::number(124f64).is_number());
        assert_eq!(
            true,
            (Value::number(1.24f64).as_number() - 1.24f64).abs() < EPSILON
        );

        let allocator = ObjectAllocator::new();

        let gc_obj = Object::new_gc_object(
            ObjectType::String(allocator.alloc("hi".to_string().into_boxed_str())),
            &allocator,
        );
        assert_eq!(true, Value::object(gc_obj).is_object());
        let object = Value::object(gc_obj).as_object();
        if let ObjectType::String(s) = object.object_type {
            assert_eq!("hi", &**s);
        } else {
            panic!("test case failed!");
        }
    }
}
