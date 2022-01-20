use std::collections::{LinkedList, HashMap};
use std::f64::EPSILON;
use std::io::{stdout, Write};
use std::mem::{self, MaybeUninit};
use std::ops::Range;
use std::panic;
use std::ptr::NonNull;
use std::time::{Instant};
use evie_common::{errors::*, info, ByteUnit, bail,  utf8_to_string, error};
#[cfg(feature="trace_enabled")]
use evie_common::{log_enabled, Level};
#[cfg(feature="trace_enabled")]
use evie_common::trace;
#[cfg(feature="trace_enabled")]
use evie_frontend::tokens::pretty_print;
use evie_common::Writer;
use evie_compiler::compiler::Compiler;
use evie_frontend::scanner::Scanner;
use evie_instructions::opcodes::{self, Opcode};
use evie_memory::{ObjectAllocator};
use evie_memory::chunk::Chunk;
use evie_memory::objects::{Closure, Location, NativeFunction, NativeFn, Class, Instance, UserDefinedFunction};
use evie_memory::objects::{Value, Object, GCObjectOf, Upvalue};

use crate::runtime_memory::Values;


const STACK_SIZE: usize = 1024;

#[derive(Debug)]
struct CallFrame {
    fn_start_stack_index: usize,
    closure: GCObjectOf<Closure>,
    ip: usize,
}

impl CallFrame {
    fn new(fn_start_stack_index: usize, closure: GCObjectOf<Closure>) -> Self {
        CallFrame {
            fn_start_stack_index,
            closure,
            ip: 0,
        }
    }

    fn ip_ptr(&self) -> *mut usize {
        &self.ip as *const usize as *mut usize
    }

    fn non_null_ptr(&self) -> NonNull<usize> {
        NonNull::new(self.ip_ptr()).unwrap()
    }

}

pub fn define_native_fn(name: &str, arity: usize, vm: &mut VirtualMachine, native_fn: NativeFn) {
    let box_str =name.to_string().into_boxed_str();
    let name = vm.allocator.alloc(box_str.clone());
    let native_function = vm.allocator.alloc(NativeFunction::new(name, arity, native_fn));
    vm.runtime_values.insert(box_str, Value::Object(Object::NativeFunction(native_function)));
}

#[derive(Default)]
pub struct Args {
    _timing_per_instruction: bool,

}

pub struct VirtualMachine<'a> {
    stack: [Value; STACK_SIZE],
    stack_top: usize,
    call_frames: Vec<CallFrame>,
    runtime_values: Values,
    up_values: LinkedList<GCObjectOf<Upvalue>>,
    custom_writer: Option<Writer<'a>>,
    allocator: ObjectAllocator,
    // unused for now
    optional_args: Option<Args>,
    ip: NonNull<usize>
}

impl<'a> std::fmt::Debug for VirtualMachine<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualMachine")
            .field("stack", &self.stack)
            .field("runtime_values", &self.runtime_values)
            .finish()
    }
}

fn init_stack() -> [Value; STACK_SIZE] {
    let data = {
        // Create an uninitialized array of `MaybeUninit`. The `assume_init` is
        // safe because the type we are claiming to have initialized here is a
        // bunch of `MaybeUninit`s, which do not require initialization.
        let mut data: [MaybeUninit<Value>; STACK_SIZE] =
            unsafe { MaybeUninit::uninit().assume_init() };

        // Dropping a `MaybeUninit` does nothing. Thus using raw pointer
        // assignment instead of `ptr::write` does not cause the old
        // uninitialized value to be dropped. Also if there is a panic during
        // this loop, we have a memory leak, but there is no memory safety
        // issue.
        for elem in &mut data[..] {
            elem.write(Value::default());
        }

        // Everything is initialized. Transmute the array to the
        // initialized type.
        unsafe { mem::transmute::<_, [Value; STACK_SIZE]>(data) }
    };
    data
}

impl<'a> VirtualMachine<'a> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        VirtualMachine::new_with_writer(None)
    }

    pub fn new_with_writer(custom_writer: Option<Writer<'a>>) -> Self {
        VirtualMachine {
            stack: init_stack(),
            stack_top: 0,
            call_frames: Vec::new(),
            runtime_values: Values::new(),
            up_values: LinkedList::new(),
            custom_writer,
            allocator: ObjectAllocator::new(),
            optional_args: None,
            ip: NonNull::new(&mut 0usize as *mut usize).expect("Null pointer"),
        }
    }

    pub fn interpret(&mut self, source: String, optional_args: Option<Args>) -> Result<()> {
        self.reset_vm();
        self.optional_args = optional_args;
        let mut scanner = Scanner::new(source);
        let start_time = Instant::now();
        let tokens = scanner.scan_tokens()?;
        info!("Tokens created in {} us", start_time.elapsed().as_micros());
        #[cfg(feature = "trace_enabled")]
        if log_enabled!(Level::Trace) {
            pretty_print(tokens, &mut stdout());
        }
        let start_time = Instant::now();
        let compiler = Compiler::new(tokens, &self.allocator);
        let main_function = compiler.compile()?;
        let upvalues = self.allocator.alloc(Vec::<GCObjectOf<Upvalue>>::new());
        info!("Compiled in {} us", start_time.elapsed().as_micros());
        self.check_arguments("", 0, 0)?;
        let closure = self.allocator.alloc(Closure::new(main_function, upvalues));
        let script = Object::Closure(closure);
        self.push_to_call_frame(CallFrame::new(0, closure));
        self.push_to_stack(Value::Object(script));
        let start_time = Instant::now();
        let result = self.run();
        info!("Ran in {} us, Total bytes allocated: {}", start_time.elapsed().as_micros(), self.allocator.bytes_allocated());
        result
    }

    fn push_to_call_frame(&mut self, c: CallFrame) {
        self.call_frames.push(c);
        self.ip = self.call_frame().non_null_ptr();
    }

    fn reset_vm(&mut self) {
        self.call_frames.clear();
        self.stack_top = 0;
    }

    #[inline(always)]
    fn call_frame(&self) -> &CallFrame {
        self.call_frame_peek_at(0)
    }
    
    #[inline(always)]
    fn call_frame_peek_at(&self, index: usize) -> &CallFrame {
        let len = self.call_frames.len();
        let index = len - 1 - index;
        assert!(index < self.call_frames.len());
        &self.call_frames[index]
    }

    #[inline(always)]
    fn ip(&self) -> usize {
        // Safety ip is set in the run method
        unsafe  {*self.ip.as_ref()}
    }

    #[inline(always)]
    fn read_byte(&mut self, chunk: &Chunk, ip: &mut usize) -> ByteUnit{
        let v =  chunk.code.read_item_at(*ip);
        *ip += 1;
        v
    }

    #[inline(always)]
    fn read_constant(&mut self, chunk: &Chunk, ip: &mut usize) -> Result<Value> {
        let v = chunk.read_constant_at(*ip);
        *ip += 1;
        Ok(v)
    }

    #[inline(always)]
    fn current_chunk(&self) -> GCObjectOf<Chunk> {
        self.current_function().chunk
    }

    #[inline(always)]
    fn current_function(&self) -> GCObjectOf<UserDefinedFunction> {
         self.current_closure().function
    }

    #[inline(always)]
    fn current_closure(&self) -> GCObjectOf<Closure> {
        self.call_frame().closure
    }


    #[inline(always)]
    fn get_value_from_stack(&self, index: usize) -> Value {
        assert!(index < STACK_SIZE, "{}", self.runtime_error(&format!("VM BUG Access out of bounds, stack size = {}, index = {}", STACK_SIZE, index)));
        self.stack[index]
    }

    #[inline(always)]
    fn set_stack_mut(&mut self, index: usize, v: Value) {
        assert!(index< STACK_SIZE, "{}", self.runtime_error(&format!("VM BUG: Stack overflow, stack size = {}, index = {}", STACK_SIZE, index)));
        self.stack[index] = v;
    }

    #[inline(always)]
    fn read_short(&mut self, chunk: &Chunk, ip: &mut usize) -> u16 {
        let first = self.read_byte(chunk, ip) as u16;
        let second = self.read_byte(chunk, ip) as u16;
        first << 8 | second
    }

    #[inline(always)]
    fn set_ip_for_run_method(&mut self, ip_ref: &mut &mut usize) {
        // Safety the call sites ensure that both unsafe access are safe
        unsafe {
            *ip_ref = self.ip.as_mut();
        }
    }

    fn run(&mut self) -> Result<()> {
        let mut chunk_obj  = self.current_chunk();
        let mut chunk = &chunk_obj;
        let mut current_ip = &mut 0;
        self.set_ip_for_run_method(&mut current_ip);
        info!("Running VM, {} Bytes allocated by by compiler", self.allocator.bytes_allocated());
        loop {
            let byte = self.read_byte(chunk, current_ip);
            let instruction = Opcode::from(byte);
            #[cfg(feature ="trace_enabled")]
            if log_enabled!(Level::Trace) {
                let mut buf = Vec::new();
                let fun_name = self.current_function().as_ref().to_string();
                opcodes::disassemble_instruction_with_writer_with_out_line_num(chunk, *current_ip -1, &mut buf, false);
                trace!(
                    "ip: {},function {}, stack: {:?}, next instruction: [{}]",
                    *current_ip,
                    fun_name,
                    self.sanitized_stack(0..self.stack_top, false),
                    &utf8_to_string(&buf).trim()
                );
            }
            match instruction {
                Opcode::Constant => {
                    let constant = self.read_constant(chunk, current_ip)?;
                    self.push_to_stack(constant);
                }
                Opcode::Return => {
                    let fn_starting_pointer = self.call_frame().fn_start_stack_index;
                    let result = self.pop_from_stack();
                    self.close_upvalues(fn_starting_pointer);
                    if self.call_frames.len() == 1 {
                        return Ok(());
                    }
                    self.call_frames.pop();
                    self.ip = self.call_frame().non_null_ptr();
                    self.set_ip_for_run_method(&mut current_ip);
                    chunk_obj = self.current_chunk();
                    chunk = &chunk_obj;
                    // drop all the local values for the last function
                    self.stack_top = fn_starting_pointer;
                    // push the return result
                    self.push_to_stack(result);
                }
                Opcode::Negate => {
                    if let Value::Number(v) = self.peek_at(0) {
                        let result = Value::Number(-v);
                        self.pop_from_stack();
                        self.push_to_stack(result);
                    } else {
                        bail!(self.runtime_error("Can only negate numbers."));
                    }
                }
                Opcode::Add => self.add()?,
                Opcode::Subtract => self.binary_op(|a, b| Value::Number(a - b))?,
                Opcode::Multiply => self.binary_op(|a, b| Value::Number(a * b))?,
                Opcode::Divide => self.binary_op(|a, b| Value::Number(a / b))?,
                Opcode::Nil => self.push_to_stack(Value::Nil),
                Opcode::True => self.push_to_stack(Value::Boolean(true)),
                Opcode::False => self.push_to_stack(Value::Boolean(false)),
                Opcode::Not => {
                    let v = self.pop_from_stack();
                    self.push_to_stack(Value::Boolean(is_falsey(&v)))
                }
                Opcode::BangEqual => {
                    let v = self.equals();
                    self.push_to_stack(Value::Boolean(!v))
                }
                Opcode::Greater => self.binary_op(|a, b| Value::Boolean(a > b))?,
                Opcode::GreaterEqual => self.binary_op(|a, b| Value::Boolean(a >= b))?,
                Opcode::Less => self.binary_op(|a, b| Value::Boolean(a < b))?,
                Opcode::LessEqual => self.binary_op(|a, b| Value::Boolean(a <= b))?,
                Opcode::EqualEqual => {
                    let v = self.equals();
                    self.push_to_stack(Value::Boolean(v))
                }
                Opcode::Print => {
                    let v = self.pop_from_stack();
                    self.print_stack_value(v);
                    self.new_line();
                }
                Opcode::Pop => {
                    self.pop_from_stack();
                }
                Opcode::DefineGlobal => {
                    let value = self.pop_from_stack();
                    let name = self.read_string(chunk, current_ip)?;
                    self.runtime_values.insert(name.as_ref().clone(), value);
                }
                Opcode::GetGlobal => {
                    let name = self.read_string(chunk, current_ip)?;
                    let value = self.runtime_values.get(name.as_ref());
                    if let Some(v) = value {
                        let v = *v;
                        self.push_to_stack(v)
                    } else {
                        bail!(self.runtime_error(&format!("Undefined variable '{}'", name.as_ref())))
                    }
                }
                Opcode::SetGlobal => {
                    let name = self.read_string(chunk, current_ip)?;
                    let value = self.peek_at(0);
                    let v = self.runtime_values.get_mut(name.as_ref());
                    match v {
                        Some(e) => {
                            *e = value;
                        }
                        None => {
                            drop(v);
                            bail!(self.runtime_error(&format!("Undefined variable '{}'", name.as_ref())))
                        }
                    }
                }
                Opcode::GetLocal => {
                    let index = self.read_byte(chunk, current_ip) as usize;
                    let fn_start_pointer = self.call_frame().fn_start_stack_index;
                    let v = self.get_value_from_stack(fn_start_pointer + index);
                    self.push_to_stack(v);
                }
                Opcode::SetLocal => {
                    let index = self.read_byte(chunk, current_ip);
                    let fn_start_pointer = self.call_frame().fn_start_stack_index;
                    self.stack[fn_start_pointer + index as usize] = self.peek_at(0);
                }
                Opcode::JumpIfFalse => {
                    let offset = self.read_short(chunk, current_ip);
                    if is_falsey(&self.peek_at(0)) {
                        *current_ip += offset as usize;
                    }
                }
                Opcode::Jump => {
                    let offset = self.read_short(chunk, current_ip);
                    *current_ip += offset as usize;
                }
                Opcode::JumpIfTrue => {
                    let offset = self.read_short(chunk, current_ip);
                    if !is_falsey(&self.peek_at(0)) {
                        *current_ip +=  offset as usize;
                    }
                }
                Opcode::Loop => {
                    let offset = self.read_short(chunk, current_ip);
                    *current_ip -= offset as usize;
                }
                Opcode::Call => {
                    let arg_count = self.read_byte(chunk,current_ip) as usize;
                    self.call_value(arg_count, self.peek_at(arg_count))?;
                    chunk_obj = self.current_chunk();
                    chunk = &chunk_obj;
                    self.set_ip_for_run_method(&mut current_ip);
                }
                Opcode::Closure => {
                    let function = self.read_function(chunk, current_ip)?;
                    let current_fn_stack_ptr = self.call_frame().fn_start_stack_index;
                    let upvalues = self.allocator.alloc(Vec::<GCObjectOf<Upvalue>>::new());
                    let mut closure = Closure::new(function, upvalues);
                    for _ in 0..function.upvalue_count {
                        let is_local = self.read_byte(chunk, current_ip) > 0;
                        let index = self.read_byte(chunk, current_ip);
                        if is_local {
                            let upvalue_index_on_stack =
                                current_fn_stack_ptr + index as usize;
                            let captured_upvalue =
                                self.capture_upvalue(upvalue_index_on_stack);
                            let upvalues = closure.upvalues.as_mut();
                            upvalues.push(captured_upvalue);
                        } else {
                            let current_closure = self.current_closure();
                            let upvalue = current_closure.upvalues[index as usize];
                            closure.upvalues.as_mut().push(upvalue);
                        }
                    }
                    let object = self.allocator.alloc(closure);
                    let stack_value = Value::Object(Object::Closure(object));
                    self.push_to_stack(stack_value);
                }
                Opcode::GetUpvalue => {
                    let slot = self.read_byte(chunk, current_ip) as usize;
                    let closure = self.current_closure();
                    let value = {
                        let upvalues = closure.upvalues;
                        assert!(slot < upvalues.len(), "{}", self.runtime_error("VM BUG: Invalid up value index"));
                        let upvalue = upvalues[slot];
                        match upvalue.location {
                            Location::Stack(index) => self.get_value_from_stack(index),
                            Location::Heap(shared_value) => *shared_value,
                        }
                    };
                    self.push_to_stack(value);
                }
                Opcode::SetUpvalue => {
                    let slot = self.read_byte(chunk, current_ip) as usize;
                    let value = self.peek_at(slot as usize);
                    let closure = self.current_closure();
                    let upvalues = closure.upvalues;
                    assert!(slot < upvalues.len(), "{}", self.runtime_error("VM BUG: Invalid up value index"));
                    let mut upvalue = upvalues[slot];
                    let location = &mut upvalue.as_mut().location;
                    match location {
                        Location::Stack(index) => {
                            let i = *index;
                            self.set_stack_mut(i, value);
                        }
                        Location::Heap(shared_value) => {
                            *shared_value.as_mut() = value
                        }
                    }
                }
                Opcode::CloseUpvalue => {
                    self.close_upvalues(self.stack_top - 1);
                    self.pop_from_stack();
                }
                Opcode::Class => {
                    let class = self.read_string(chunk, current_ip)?;
                    let methods= self.allocator.alloc(HashMap::<GCObjectOf<Box<str>>, GCObjectOf<Closure>>::new());
                    let class_obj = self.allocator.alloc(Class::new(class, methods));
                    let value = Value::Object(Object::Class(class_obj));
                    self.push_to_stack(value);
                }
                Opcode::SetProperty => {
                    let property = self.read_string(chunk, current_ip)?;
                    let value = self.peek_at(0);
                    let mut instance = self.peek_at(1);
                    if let Value::Object(Object::Instance(i)) = &mut instance {
                        self.set_property(i, property, value)?;
                        let value = self.pop_from_stack();
                        self.pop_from_stack();
                        // a.b = '2' evaluates to '2'
                        self.push_to_stack(value);
                    } else {
                        bail!(self.runtime_error(&format!("Only instances can have properties got {} instead", instance)))
                    }
                }
                Opcode::GetProperty => {
                    let property = self.read_string(chunk, current_ip)?;
                    let instance = self.peek_at(0);
                    if let Value::Object(Object::Instance(i)) = instance {
                        let v = self.get_property(i, property)?;
                        self.pop_from_stack();
                        self.push_to_stack(v);
                    } else {
                        bail!(self.runtime_error(&format!("Only instances can have properties got {} instead", instance)))
                    }
                    
                }
                Opcode::Method => {
                    let method_name = self.read_string(chunk, current_ip)?;
                    self.define_method(method_name)?;
                }
                Opcode::Invoke => {
                    let method = self.read_string(chunk, current_ip)?;
                    let arg_count = self.read_byte(chunk, current_ip) as usize;
                    let receiver = self.peek_at(arg_count);
                    let fn_start_stack_index = self.stack_top - arg_count - 1;
                    self.invoke(receiver, method, fn_start_stack_index)?;
                    chunk_obj = self.current_chunk();
                    chunk = &chunk_obj;
                    self.set_ip_for_run_method(&mut current_ip);
                }
            };
        }
    }

    fn invoke(&mut self, receiver: Value, method: GCObjectOf<Box<str>>, fn_start_stack_index: usize) -> Result<()> {
        if let Value::Object(Object::Instance(i)) = receiver {
            if let Some(closure) = i.class.methods.get(&method) {
                self.set_stack_mut(fn_start_stack_index, receiver);
                self.push_closure_to_call_frame(*closure, fn_start_stack_index)?;
                return Ok(())
            }
        }
        bail!(self.runtime_error(&format!("Undefined method '{}'", *method)))
    }

    fn set_property(&mut self, instance: &mut Instance, property: GCObjectOf<Box<str>>, value: Value) -> Result<()> {
        let fields = instance.fields.as_mut();
        fields.insert(property, value);
        Ok(())
    }

    fn get_property(
        &mut self,
        instance: GCObjectOf<Instance>,
        property: GCObjectOf<Box<str>>,
    )  -> Result<Value>{
        if let Some(v) =instance.fields.get(&property) {
            Ok(*v)
        } else if let Some(&method) = instance.class.methods.get(&property){
                Ok(self.bind_method(instance, method))
        } else {
            bail!(self.runtime_error(&format!("No property or method with the name {}", *property)))
        }
    }

    fn bind_method(&mut self, instance: GCObjectOf<Instance>, method: GCObjectOf<Closure>) -> Value{
        self.pop_from_stack();
        Value::Object(Object::BoundMethod(instance, method))
    }

    fn define_method(&mut self, method_name: GCObjectOf<Box<str>>) -> Result<()> {
        let value = self.peek_at(0);
        let method = if let Value::Object(Object::Closure(c)) = value {
            c
        } else {
            panic!("{}", self.runtime_error(&format!("VM BUG: expected a closure but got {}", value)));
        };
        if let Value::Object(Object::Class(c)) = self.peek_at(1) {
            let mut methods = c.methods;
            methods.insert(method_name, method);
        } else {
            bail!(self.runtime_error("Only classes can have methods"))
        }
        self.pop_from_stack(); //method closure
        Ok(())
    }

    fn sanitized_stack(&self, range: Range<usize>, with_address: bool) -> Vec<String> {
        let s: Vec<String> = self.stack[range]
            .iter()
            .enumerate()
            .map(|(i, v)| {
                if with_address {
                    format!("{}:({:p}->{})", i, v as *const _, v)
                } else {
                    format!("{}:({})", i, v)
                }
            })
            .collect();
        s
    }

    fn close_upvalues(&mut self, last_index: usize){
        let upvalue_iter = self.up_values.iter().rev();
        let mut count = 0;
        upvalue_iter
            .take_while(|u| match u.location {
                Location::Stack(index) => index >= last_index,
                _ => false,
            })
            .for_each(|u| {
                count += 1;
                let mut u = *u;
                let location = u.location;
                if let Location::Stack(index) = location {
                    let stack_value = self.get_value_from_stack(index);
                    // Moving from stack to heap
                    let heap_value = self.allocator.alloc(stack_value);
                    u.as_mut().location = Location::Heap(heap_value);
                }
            });
        
        // drop the ones we don't need.
        let _captured_values = self.up_values.split_off(self.up_values.len() - count);
    }

    fn capture_upvalue(&mut self, stack_index: usize) -> GCObjectOf<Upvalue> {
        let upvalue_iter = self.up_values.iter().rev();
        let upvalue = upvalue_iter
            .take_while(|&&u| {
                match u.location {
                    Location::Stack(index) => index >= stack_index,
                    _ => false,
                }
            })
            .find(|&&u| {
                match u.location {
                    Location::Stack(index) => index == stack_index,
                    _ => false,
                }
            });
        if let Some(u) = upvalue {
            *u
        } else {
            let created_value = self.allocator.alloc(Upvalue::new_with_location(Location::Stack(stack_index)));
            self.up_values.push_back(created_value);
            created_value
        }
    }

    #[inline(always)]
    fn call_value(&mut self, arg_count: usize, value: Value) -> Result<()> {
        let arg_count = arg_count as usize;
        let start_index = self.stack_top - 1 - arg_count;
        match value {
            Value::Object(Object::Closure(c)) => {
                    self.check_arguments(&c.function.name.unwrap(), c.function.arity,arg_count)?;
                    self.push_closure_to_call_frame(c, start_index)
                }
                Value::Object(Object::Class(class)) => {
                    let methods = class.methods;
                    let fields = self.allocator.alloc(HashMap::<GCObjectOf<Box<str>>, Value>::new());
                    let instance = self.allocator.alloc(Instance::new(class, fields));
                    let receiver = Value::Object(Object::Instance(instance));
                    // TODO preallocate this;
                    let init = self.allocator.alloc("init".to_string().into_boxed_str());
                    if let Some(init) = methods.get(&init) {
                        self.check_arguments(&init.function.name.unwrap(), init.function.arity, arg_count)?;
                        // set the receiver at start index for the constructor;
                        self.set_stack_mut(
                            start_index,
                            receiver
                        );
                        self.push_closure_to_call_frame(*init, start_index)?;
                    } else {
                        if arg_count != 0 {
                            bail!(self
                                .runtime_error(&format!("Expected 0 arguments but got {} for {} constructor", arg_count, *class.name)))
                        }
                        self.set_stack_mut(start_index, receiver);
                    }
                    Ok(())
                },
                Value::Object(Object::BoundMethod(_, closure)) => {
                    self.check_arguments(&closure.function.name.unwrap(), closure.function.arity, arg_count)?;
                    // set the receiver at start index for the constructor;
                    self.set_stack_mut(
                        start_index,
                        value
                    );
                    self.push_closure_to_call_frame(closure, start_index)?;
                    Ok(())
                }
                Value::Object(Object::NativeFunction(f)) => {
                    self.check_arguments(&f.name, f.arity, arg_count)?;
                    self.call_native_function(&f, arg_count, start_index)?;
                    Ok(())
                }
                _ => bail!(self.runtime_error(&format!(
                    "can only call a function/closure, constructor or a class method, got '{}', at stack index {}",
                    value, 
                    self.stack_top - 1 - arg_count
                ))),
            }
        }

    #[inline(always)]
    fn push_closure_to_call_frame(
        &mut self,
        closure: GCObjectOf<Closure>,
        fn_start_stack_index: usize,
    ) -> Result<()> {
        self.push_to_call_frame(CallFrame::new(fn_start_stack_index, closure));
        Ok(())
    }

    fn call_native_function(
        &mut self,
        native_function: &NativeFunction,
        arg_count: usize,
        fn_start_stack_index: usize,
    ) -> Result<()> {
        let mut arguments = Vec::new();
        for v in &self.stack[fn_start_stack_index..(fn_start_stack_index + arg_count)] {
            arguments.push(*v);
        }
        let result = native_function.call(arguments);
        self.stack_top = fn_start_stack_index + 1;
        self.set_stack_mut(fn_start_stack_index, result);
        Ok(())
    }

    #[inline(always)]
    fn check_arguments(&mut self, name: &str, arity: usize, arg_count: usize) -> Result<()> {
        if arity != arg_count {
            bail!(self.runtime_error(&format!(
                "Expected {} arguments but got {} for <fn {}>",
                arity, arg_count, name
            )))
        }
        Ok(())
    }

    #[inline(always)]
    fn read_string(&mut self, chunk:  &Chunk, ip: &mut usize) -> Result<GCObjectOf<Box<str>>> {
        let constant = self.read_constant(chunk, ip)?;
        match constant {
            Value::Object(Object::String(s)) => Ok(s),
            _ => Err(self.runtime_error("message").into()),
        }
    }

    #[inline(always)]
    fn read_function(&mut self, chunk:  &Chunk, ip: &mut usize) -> Result<GCObjectOf<UserDefinedFunction>> {
        let constant = self.read_constant(chunk, ip)?;
        match constant {
            Value::Object(Object::Function(s)) => Ok(s),
            _ => bail!(self.runtime_error("Not a function")),
        }
    }

    fn runtime_error(&self, message: &str) -> ErrorKind {
        let mut error_buf = vec![];
        writeln!(error_buf, "{}", message).expect("Write failed");
        let all_call_frames = (&self.call_frames).iter().rev();
        for frame in all_call_frames {
            let function = *frame.closure.function;
            let fun_name = &function.to_string();
            let ip = frame.ip;
            let line_num = function.chunk.lines[ip];
            writeln!(error_buf, "[line {}] in {}", line_num, fun_name)
                .expect("Write failed")
        }
        if self.stack_top < STACK_SIZE {
            // We print stack only if it is not stack overflow
            error!(
                "Error at function= {}, ip ={}, stack ={:?}",
                &self
                    .current_function()
                    
                    .to_string(),
                self.ip(),
                self.sanitized_stack(0..self.stack_top, false)
            );
        }
        let chunk = self.current_chunk();
        let line = chunk.lines[self.ip()];
        runtime_vm_error(line, &utf8_to_string(&error_buf))
    }

    #[inline(always)]
    fn peek_at(&self, distance: usize) -> Value {
        let top = self.stack_top;
        self.get_value_from_stack(top - 1 - distance)
    }

    #[inline(always)]
    fn equals(&mut self) -> bool {
        let left = self.pop_from_stack();
        let right = self.pop_from_stack();
        value_equals(left, right)
    }

    #[inline(always)]
    fn binary_op(&mut self, op: fn(f64, f64) -> Value) -> Result<()> {
        let (left, right) = (self.peek_at(1), self.peek_at(0));
        let (left, right) = match (left, right) {
            (Value::Number(l), Value::Number(r)) => (l, r),
            _ => bail!(self.runtime_error("Can perform binary operations only on numbers.")),
        };
        self.binary_op_with_num(left, right, op);
        Ok(())
    }

    fn add(&mut self) -> Result<()> {
        match (self.peek_at(1), self.peek_at(0)) {
            (Value::Object(Object::String(l)), Value::Object(Object::String(r))) => {
                let mut concatenated_string = String::new();
                        concatenated_string.push_str(&l);
                        concatenated_string.push_str(&r);
                        self.pop_from_stack();
                        self.pop_from_stack();
                        let allocated_string = self.allocator.alloc(concatenated_string.into_boxed_str());
                        let sv = Value::Object(Object::String(allocated_string));
                        self.push_to_stack(sv);
                        Ok(())
            }
            (Value::Number(_), Value::Number(_)) => self.binary_op(|a, b| Value::Number(a + b)),
            _ => bail!(self.runtime_error(&format!(
                "Add can be perfomed only on numbers or strings, got {} and {}",
                self.peek_at(1),
                self.peek_at(0)
            ))),
        }
    }

    #[inline(always)]
    fn binary_op_with_num(
        &mut self,
        left: f64,
        right: f64,
        op: fn(f64, f64) -> Value,
    ) {
        let result = op(left, right);
        self.pop_from_stack();
        self.pop_from_stack();
        self.push_to_stack(result);
    }

    #[inline(always)]
    fn push_to_stack(&mut self, value: Value) {
        assert!(self.stack_top < STACK_SIZE, "{}", self.runtime_error(&format!("Stack overflow, stack size = {}, index = {}", STACK_SIZE, self.stack_top)));
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }
    #[inline(always)]
    fn pop_from_stack(&mut self) -> Value {
        self.stack_top -= 1;
        assert!(self.stack_top < STACK_SIZE);
        self.stack[self.stack_top]
    }

    pub fn free(&mut self) {
        //TODO
    }

    #[inline(always)]
    fn print_stack_value(&mut self, value: Value) {
        match self.custom_writer.as_deref_mut() {
            Some(w) => print_stack_value(value, w),
            None => print_stack_value(value, &mut stdout()),
        }
    }
    #[inline(always)]
    fn new_line(&mut self) {
        match self.custom_writer.as_deref_mut() {
            Some(w) => writeln!(w).expect("Write failed"),
            None => println!(),
        };
    }
}

#[inline(always)]
fn runtime_vm_error(line: usize, message: &str) -> ErrorKind {
    ErrorKind::RuntimeError(format!("Line: {}, message: {}", line, message))
}

#[inline(always)]
fn num_equals(l: f64, r: f64) -> bool {
    (l - r).abs() < EPSILON
}
#[inline(always)]
fn value_equals(l: Value, r: Value) -> bool {
    match (l, r) {
        (Value::Boolean(l), Value::Boolean(r)) => l == r,
        (Value::Nil, Value::Nil) => true,
        (Value::Number(l), Value::Number(r)) => num_equals(l, r),
        (Value::Object(Object::String(l)), Value::Object(Object::String(r))) => {
            std::ptr::eq(l.as_ptr(), r.as_ptr()) || l == r
        },
        _ => false,
    }
}

fn is_falsey(value: &Value) -> bool {
    match value {
        Value::Boolean(b) => !b,
        Value::Nil => true,
        _ => false,
    }
}

fn print_stack_value(value: Value, writer: &mut dyn Write) {
   opcodes::print_value(value, writer)
}

#[cfg(test)]
mod tests {

    use evie_common::{errors::*, utf8_to_string, print_error};
    use evie_native::clock;

    use crate::vm::VirtualMachine;

    use super::{define_native_fn};
    
    #[test]
    fn vm_numeric_expressions() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        var a = 2;
        print a; //2
        a = -2 + 4 * 2 == 6;
        print a; // true
        var b;
        print !b; // true
        print a == true; //true
        var c = a == !b; // true
        print c; //true
        print !nil; //true
        print 3 == false; //false
        print (2 + 3)/5; //1
        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!(
            "2\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\n1\n",
            utf8_to_string(&buf)
        );
        Ok(())
    }

    #[test]
    fn vm_string_expressions() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        var a = "hello ";
        var b =" world";
        var c= a;
        print c + b;
        print a ==c;
        print a==b;
        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("hello  world\ntrue\nfalse\n", utf8_to_string(&buf));
        Ok(())
    }

    #[test]
    fn vm_block() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        var a = 2;
        {
            print a;
            var a = 3;
            print a;
            var b = a;
        }
        print a;
        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("2\n3\n2\n", utf8_to_string(&buf));
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        var a = 2;
        {
            print a;
            var a = 3;
            print a;
            var b = a;
        }
        print a;
        print b;
        "#;
        match vm.interpret(source.to_string(), None) {
            Ok(_) => panic!("Expected to fail"),
            Err(e) => assert_eq!(
                "Runtime Error: Line: 10, message: Undefined variable 'b'\n[line 10] in <fn script>\n",
                e.to_string()
            ),
        }

        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        var a = 2;
        {
            var b = (2 + a) * 4;
            a = b;
        }
        print a;
        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("16\n", utf8_to_string(&buf));
        Ok(())
    }

    #[test]
    fn vm_if_statement() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        var a = "";
        var condition = true;
        if (condition) {
            a = "if";
        } else {
            a = "else";
        }
        print a;
        condition = 2==3;
        if (condition) {
            a = "if";
        } else {
            a = "else";
        }
        print a;
        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("if\nelse\n", utf8_to_string(&buf));
        Ok(())
    }

    #[test]
    fn vm_logical_operations() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        print 2 or 3;
        print 2 and 3;
        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("2\n3\n", utf8_to_string(&buf));
        Ok(())
    }

    #[test]
    fn vm_while_loop() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        var a = 5;
        var b = 1;
        while (a == 5) {
            print b;
            b = b + 1;
            if (b > 5)
                a = "stop";
        }
        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("1\n2\n3\n4\n5\n", utf8_to_string(&buf));
        Ok(())
    }

    #[test]
    fn vm_call_error_stack_trace() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        fun a() { b(); }
        fun b() { c(); }
        fun c() {
            c("too", "many");
        }

        a();
        "#;
        match vm.interpret(source.to_string(), None) {
            Ok(_) => panic!("Expect this to fail"),
            Err(e) => {
                print_error(e, &mut buf);
            }
        }
        assert_eq!(
            r#"[Runtime Error] Line: 5, message: Expected 0 arguments but got 2 for <fn c>
[line 5] in <fn c>
[line 3] in <fn b>
[line 2] in <fn a>
[line 8] in <fn script>

"#,
            utf8_to_string(&buf)
        );
        Ok(())
    }

    #[test]
    fn vm_call_success() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        var hello = "hello";
        var world = " world";
        fun a() { return b(); }
        fun b() { return c(hello, world); }
        fun c(arg1, arg2) {
            print arg1 + arg2;
        }
        a();
        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("hello world\n", utf8_to_string(&buf));

        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        var hello = "hello";
        var world = " world";
        fun a() { return b(); }
        fun b() { return c(hello, world); }
        fun c(arg1, arg2) {
            return  arg1 + arg2;
        }
        print a();
        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("hello world\n", utf8_to_string(&buf));

        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        fun fib(n) {
            if (n < 2) return n;
            return fib(n - 1) + fib(n - 2); 
          }
          
          print fib(10);
        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("55\n", utf8_to_string(&buf));
        Ok(())
    }

    #[test]
    fn vm_closure() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        fun outer() {
            var x = "outside";
            fun inner() {
              print x;
            }
            inner();
          }
        outer();
        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("outside\n", utf8_to_string(&buf));

        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        fun outer() {
            var x = "outside";
            fun inner() {
              print x;
            }
          
            return inner;
          }
          
        var closure = outer();
        closure();
        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("outside\n", utf8_to_string(&buf));

        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        var globalSet;
        var globalGet;

        fun main() {
          var a = "initial";

          fun set() { a = "updated"; }
          fun get() { print a; }

          globalSet = set;
          globalGet = get;
        }

        main();
        globalSet();
        globalGet();
        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("updated\n", utf8_to_string(&buf));
        Ok(())
    }

    #[test]
    fn vm_class_fields() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        class Pair {}

        var pair = Pair();
        pair.first = 1;
        pair.second = 2;
        print pair.first + pair.second; // 3.

        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("3\n", utf8_to_string(&buf));
        Ok(())
    }

    #[test]
    fn vm_class_methods() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        class Scone {
            topping(first, second) {
              print "scone with " + first + " and " + second;
            }
          }
          
          var scone = Scone();
          scone.topping("berries", "cream");

        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("scone with berries and cream\n", utf8_to_string(&buf));
        Ok(())
    }

    #[test]
    fn vm_bound_methods() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        class Scone {
            topping(first, second) {
              print "scone with " + first + " and " + second;
            }
          }
          
          var scone = Scone();
          var topping = scone.topping;
          topping("berries", "cream");

        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("scone with berries and cream\n", utf8_to_string(&buf));
        Ok(())
    }

    #[test]
    fn vm_class_initializer_and_this() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        class Brunch {
            init(food, drinks) {
                this.food = food;
                this.drinks = drinks;
            }
        }
                  
        var brunch = Brunch("eggs", "coffee");
        
        print brunch.food + " and " + brunch.drinks;

        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("eggs and coffee\n", utf8_to_string(&buf));

        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        class Brunch {
            init(food, drinks) {
                this.food = food;
                this.drinks = drinks;
            }

            set_dessert(item) {
                this.dessert = item;
                return this;
            }
        }
                  
        var brunch = Brunch("eggs", "coffee");

        var brunch_with_dessert = brunch.set_dessert("cake");
        
        print brunch_with_dessert.food + " and " + brunch_with_dessert.drinks + " with " + brunch_with_dessert.dessert + " as dessert";

        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!(
            "eggs and coffee with cake as dessert\n",
            utf8_to_string(&buf)
        );

        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        class Cake {
    
            init(type) {
                this.type = type;
            }
            taste() {
                this.inner_taste();
                this.flavor = "Belgian chocolate";
            }
        
             taste_again() {
                this.inner_taste();
            }
        
            inner_taste() {
                var adjective = "delicious";
                print "The " + this.flavor + " " + this.type + " is " + adjective + "!";
            }
        }

        var cake = Cake("cake");
        cake.flavor = "German chocolate";
        cake.taste();
        cake.taste_again(); 
    
    
        var cookie = Cake("cookie");
        cookie.flavor = "German chocolate";
        cookie.taste();
        cookie.taste_again(); 

        "#;
        vm.interpret(source.to_string(), None)?;
        assert_eq!("The German chocolate cake is delicious!\nThe Belgian chocolate cake is delicious!\nThe German chocolate cookie is delicious!\nThe Belgian chocolate cookie is delicious!\n", utf8_to_string(&buf));

        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        class Brunch {
            init(food, drinks) {
                this.food = food;
                this.drinks = drinks;
            }
        }
                  
        var brunch = Brunch("eggs");
        "#;
        match vm.interpret(source.to_string(), None) {
            Err(e) => {
                print_error(e, &mut buf);
                assert_eq!("[Runtime Error] Line: 9, message: Expected 2 arguments but got 1 for <fn init>\n[line 9] in <fn script>\n\n", utf8_to_string(&buf))
            }
            Ok(_) => panic!("This test is expected to fail"),
        }

        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        class Brunch {
            init(food, drinks) {
                this.food = food;
                this.drinks = drinks;
                return 2;
            }
        }
                  
        var brunch = Brunch("eggs");
        "#;
        match vm.interpret(source.to_string(), None) {
            Err(e) => {
                print_error(e, &mut buf);
                assert_eq!("[Parse Error] [line: 6] Error at <2>: message: Can't return a value from an initializer\n", utf8_to_string(&buf))
            }
            Ok(_) => panic!("This test is expected to fail"),
        }
        Ok(())
    }


    #[test]
    #[should_panic] 
    fn vm_stack_overflow()  {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        fun infinite_recursion() {
            infinite_recursion();
        }

        infinite_recursion();
        "#;
        match vm.interpret(source.to_string(), None) {
            Ok(_) => panic!("This should not happen"),
            Err(_) => panic!("This should not happen"),
        }
    }

    #[test]
    fn vm_native_clock() -> Result<()> {
        let mut buf = vec![];
        let mut vm = VirtualMachine::new_with_writer(Some(&mut buf));
        let source = r#"
        print clock();
        "#;
        define_native_fn("clock", 0, &mut vm, clock);
        let _ = vm.interpret(source.to_string(), None)?;
        let output = utf8_to_string(&buf);
        // This will fail if it is not f64
        let _ = output.trim().parse::<f64>().unwrap();
        Ok(())
    }
}
