//! All Native functions supported by Evie

#[cfg(feature = "trace_enabled")]
use evie_common::trace;
use evie_memory::objects::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn clock(_: Vec<Value>) -> Value {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64();
    #[cfg(feature = "trace_enabled")]
    trace!("native fn clock() -> {} ", since_the_epoch);
    Value::Number(since_the_epoch)
}
