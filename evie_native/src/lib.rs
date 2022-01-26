//! All Native functions supported by Evie.
//!
//! Currently only supports two [clock] & [to_string]

#[cfg(feature = "trace_enabled")]
use evie_common::trace;
#[cfg(feature = "nan_boxed")]
use evie_memory::objects::nan_boxed::Value;
#[cfg(not(feature = "nan_boxed"))]
use evie_memory::objects::non_nan_boxed::Value;
use evie_memory::{
    objects::{Object, ObjectType},
    ObjectAllocator,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Prints the current time as a [evie_memory::objects::Value::Number] (float)
pub fn clock(_: Vec<Value>, _: &ObjectAllocator) -> Value {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64();
    #[cfg(feature = "trace_enabled")]
    trace!("native fn clock() -> {} ", since_the_epoch);
    Value::number(since_the_epoch)
}

/// Converts the given [evie_memory::objects::Value]  into a [evie_memory::objects::ObjectType::String]
pub fn to_string(inputs: Vec<Value>, allocator: &ObjectAllocator) -> Value {
    let result = inputs[0].to_string();
    #[cfg(feature = "trace_enabled")]
    trace!("native fn to_string() -> {} ", result);
    let string = ObjectType::String(allocator.alloc(result.into_boxed_str()));
    Value::object(Object::new_gc_object(string, allocator))
}
