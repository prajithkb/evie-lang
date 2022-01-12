use std::{convert::TryFrom, fmt::Display, io::Write};

use evie_common::ByteUnit;
use evie_memory::{
    chunk::Chunk,
    objects::{Object, Value, Function},
};
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// The supported op codes for Evie VM.
/// TODO: Add detailed docs for each
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Opcode {
    Constant,
    Return,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Nil,
    True,
    False,
    Not,
    EqualEqual,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Print,
    Pop,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    JumpIfFalse,
    JumpIfTrue,
    Jump,
    Loop,
    Call,
    Closure,
    GetUpvalue,
    SetUpvalue,
    CloseUpvalue,
    Class,
    SetProperty,
    GetProperty,
    Method,
    Invoke,
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("OpCode[{:?}]", self))
    }
}

pub fn simple_instruction(instruction: &Opcode, offset: usize, writer: &mut dyn Write) -> usize {
    writeln!(writer, "{}", instruction.to_string()).expect("Write failed");
    offset + 1
}

pub fn constant_instruction(
    instruction: &Opcode,
    chunk: &Chunk,
    offset: usize,
    writer: &mut dyn Write,
) -> usize {
    let constant = *chunk.code.read_item_at(offset + 1);
    write!(writer, "{:<30} {:4} '", instruction.to_string(), constant).expect("Write failed");
    print_value(chunk.constants.read_item_at(constant as usize), writer);
    writeln!(writer, "'").expect("Write failed");
    offset + 2
}

pub fn byte_instruction(
    instruction: &Opcode,
    chunk: &Chunk,
    offset: usize,
    writer: &mut dyn Write,
) -> usize {
    let slot = *chunk.code.read_item_at(offset + 1);
    writeln!(writer, "{:<30} {:4}", instruction.to_string(), slot).expect("Write failed");
    offset + 2
}

pub fn jump_instruction(
    instruction: &Opcode,
    chunk: &Chunk,
    sign: i32,
    offset: usize,
    writer: &mut dyn Write,
) -> usize {
    let mut jump = as_u16(*chunk.code.read_item_at(offset + 1)) << 8;
    jump |= as_u16(*chunk.code.read_item_at(offset + 2));
    writeln!(
        writer,
        "{:<30} {:4} -> {}",
        instruction.to_string(),
        offset,
        (offset as i32) + 3 + (jump as i32) * sign
    )
    .expect("Write failed");
    offset + 3
}

fn as_u16(i: ByteUnit) -> u16 {
    i as u16
}

pub fn print_value(value: &Value, writer: &mut dyn Write) {
    write!(writer, "{}", value).expect("Write failed");
}

pub fn disassemble_chunk_with_writer(chunk: &Chunk, name: &str, writer: &mut dyn Write) {
    writeln!(writer, "== {} ==", name).expect("Write failed");
    let mut offset = 0;
    while offset < chunk.code.count {
        offset = disassemble_instruction_with_writer(chunk, offset, writer);
    }
}

pub fn disassemble_instruction_with_writer(
    chunk: &Chunk,
    offset: usize,
    writer: &mut dyn Write,
) -> usize {
    write!(writer, "{:04} ", offset).expect("Write failed");
    if offset > 0 && chunk.lines[offset - 1] == chunk.lines[offset] {
        write!(writer, "   | ").expect("Write failed");
    } else {
        write!(writer, "{:04} ", chunk.lines[offset]).expect("Write failed");
    }
    let byte = *chunk.code.read_item_at(offset);
    disassemble_instruction(byte, chunk, offset, writer)
}
pub fn closure_instruction(
    instruction: &Opcode,
    chunk: &Chunk,
    mut offset: usize,
    writer: &mut dyn Write,
) -> usize {
    offset += 1;
    let constant = *chunk.code.read_item_at(offset);
    offset += 1;
    write!(writer, "{:<30} {:4} '", instruction.to_string(), constant).expect("Write failed");
    print_value(chunk.constants.read_item_at(constant as usize), writer);
    writeln!(writer, "'").expect("write failed");
    let v = &*chunk.constants.read_item_at(constant as usize);
    if let Value::Object(Object::Function(c)) = v {
        let function = c.as_ref();
        if let Function::UserDefined(u) = function {
            for _ in 0..u.upvalue_count {
                let is_local = *chunk.code.read_item_at(offset);
                offset += 1;
                let index = *chunk.code.read_item_at(offset);
                offset += 1;
                writeln!(
                    writer,
                    "{:04}    |{:>38}{} {}",
                    offset - 2,
                    "",
                    if is_local == 1 { "local" } else { "upvalue" },
                    index
                )
                .expect("Write failed");
            }
        }
    }
    offset
}

pub fn invoke_instruction(
    instruction: &Opcode,
    chunk: &Chunk,
    offset: usize,
    writer: &mut dyn Write,
) -> usize {
    let constant = *chunk.code.read_item_at(offset + 1);
    let arg_count = *chunk.code.read_item_at(offset + 2);

    write!(
        writer,
        "{:<30}   ({} args){:4} '",
        instruction.to_string(),
        arg_count,
        constant
    )
    .expect("Write failed");
    print_value(chunk.constants.read_item_at(constant as usize), writer);
    writeln!(writer, "'").expect("Write failed");
    offset + 3
}

pub fn disassemble_instruction(
    byte: ByteUnit,
    chunk: &Chunk,
    offset: usize,
    writer: &mut dyn Write,
) -> usize {
    match Opcode::try_from(byte) {
        Ok(instruction) => match instruction {
            Opcode::Constant => constant_instruction(&instruction, chunk, offset, writer),
            Opcode::SetLocal => byte_instruction(&instruction, chunk, offset, writer),
            Opcode::Jump => jump_instruction(&instruction, chunk, 1, offset, writer),
            Opcode::Loop => jump_instruction(&instruction, chunk, -1, offset, writer),
            Opcode::Return => simple_instruction(&instruction, offset, writer),
            Opcode::Add => simple_instruction(&instruction, offset, writer),
            Opcode::Subtract => simple_instruction(&instruction, offset, writer),
            Opcode::Multiply => simple_instruction(&instruction, offset, writer),
            Opcode::Divide => simple_instruction(&instruction, offset, writer),
            Opcode::Negate => simple_instruction(&instruction, offset, writer),
            Opcode::Nil => simple_instruction(&instruction, offset, writer),
            Opcode::True => simple_instruction(&instruction, offset, writer),
            Opcode::False => simple_instruction(&instruction, offset, writer),
            Opcode::Not => simple_instruction(&instruction, offset, writer),
            Opcode::EqualEqual => simple_instruction(&instruction, offset, writer),
            Opcode::BangEqual => simple_instruction(&instruction, offset, writer),
            Opcode::Greater => simple_instruction(&instruction, offset, writer),
            Opcode::GreaterEqual => simple_instruction(&instruction, offset, writer),
            Opcode::Less => simple_instruction(&instruction, offset, writer),
            Opcode::LessEqual => simple_instruction(&instruction, offset, writer),
            Opcode::Print => simple_instruction(&instruction, offset, writer),
            Opcode::Pop => simple_instruction(&instruction, offset, writer),
            Opcode::Closure => closure_instruction(&instruction, chunk, offset, writer),
            Opcode::CloseUpvalue => simple_instruction(&instruction, offset, writer),
            Opcode::DefineGlobal => constant_instruction(&instruction, chunk, offset, writer),
            Opcode::GetGlobal => constant_instruction(&instruction, chunk, offset, writer),
            Opcode::SetGlobal => constant_instruction(&instruction, chunk, offset, writer),
            Opcode::GetLocal => byte_instruction(&instruction, chunk, offset, writer),
            Opcode::Call => byte_instruction(&instruction, chunk, offset, writer),
            Opcode::GetUpvalue => byte_instruction(&instruction, chunk, offset, writer),
            Opcode::SetUpvalue => byte_instruction(&instruction, chunk, offset, writer),
            Opcode::JumpIfFalse => jump_instruction(&instruction, chunk, 1, offset, writer),
            Opcode::JumpIfTrue => jump_instruction(&instruction, chunk, 1, offset, writer),
            Opcode::Class => constant_instruction(&instruction, chunk, offset, writer),
            Opcode::SetProperty => constant_instruction(&instruction, chunk, offset, writer),
            Opcode::GetProperty => constant_instruction(&instruction, chunk, offset, writer),
            Opcode::Method => constant_instruction(&instruction, chunk, offset, writer),
            Opcode::Invoke => invoke_instruction(&instruction, chunk, offset, writer),
        },
        Err(e) => {
            eprintln!(
                "Invalid instruction {:?}[value={}], error: {}",
                byte, offset, e
            );
            offset + 1
        }
    }
}

#[cfg(test)]
mod tests {

    use evie_common::{errors::*, utf8_to_string, ByteUnit};
    use evie_memory::{chunk::Chunk, objects::Value};

    use crate::opcodes::{disassemble_chunk_with_writer, Opcode};

    #[test]
    fn test_chunk() -> Result<()> {
        let mut chunk = Chunk::new();

        // -((1.2 + 3.4)/5.6)
        let constant = chunk.add_constant(Value::Number(1.2));
        chunk.write_chunk(Opcode::Constant.into(), 123);
        chunk.write_chunk(constant as ByteUnit, 123);

        let constant = chunk.add_constant(Value::Number(3.4));
        chunk.write_chunk(Opcode::Constant.into(), 123);
        chunk.write_chunk(constant as ByteUnit, 123);

        chunk.write_chunk(Opcode::Add.into(), 123);

        let constant = chunk.add_constant(Value::Number(5.6));
        chunk.write_chunk(Opcode::Constant.into(), 123);
        chunk.write_chunk(constant as ByteUnit, 123);

        chunk.write_chunk(Opcode::Divide.into(), 123);

        chunk.write_chunk(Opcode::Negate.into(), 123);

        chunk.write_chunk(Opcode::Return.into(), 123);
        let mut buf = vec![];
        disassemble_chunk_with_writer(&chunk, "test", &mut buf);
        assert_eq!(
            r#"== test ==
0000 0123 OpCode[Constant]                  0 '1.2'
0002    | OpCode[Constant]                  1 '3.4'
0004    | OpCode[Add]
0005    | OpCode[Constant]                  2 '5.6'
0007    | OpCode[Divide]
0008    | OpCode[Negate]
0009    | OpCode[Return]
"#,
            utf8_to_string(&buf)
        );
        Ok(())
    }
}
