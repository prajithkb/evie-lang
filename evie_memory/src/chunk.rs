use evie_common::ByteUnit;

use crate::objects::Value;

///  Chunk in evie holds the byte code & constants. Created by the Compiler.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Memory<ByteUnit>,
    pub constants: Memory<Value>,
    pub lines: Vec<usize>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
#[allow(unused)]
impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Memory::new(),
            constants: Memory::new(),
            lines: Vec::new(),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> ByteUnit {
        self.constants.write_item(value);
        // /After we add the constant, we return the index where the constant was appended
        // so that we can locate that same constant later.
        (self.constants.item_count() - 1) as ByteUnit
    }

    #[inline]
    pub fn read_constant_at(&self, offset: usize) -> Value {
        let offset = self.code.read_item_at(offset);
        self.constants.read_item_at(offset as usize)
    }

    pub fn write_chunk(&mut self, byte: ByteUnit, line: usize) {
        self.code.write_item(byte);
        self.lines.push(line);
    }
    pub fn free_code(&mut self) {
        self.code.free_items();
    }

    pub fn free_data(&mut self) {
        self.constants.free_items();
    }

    pub fn free_all(&mut self) {
        self.free_code();
        self.free_data();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Memory<T: Copy> {
    pub inner: Vec<T>,
}
#[allow(unused)]
impl<T: Copy> Memory<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Memory { inner: vec![] }
    }

    #[inline(always)]
    pub fn item_count(&self) -> usize {
        self.inner.len()
    }

    #[inline(always)]
    pub fn write_item(&mut self, item: T) {
        self.inner.push(item);
    }

    #[inline(always)]
    pub fn read_item_at(&self, index: usize) -> T {
        assert!(
            index < self.inner.len(),
            "Out of bound access, index {}, len {}",
            index,
            self.inner.len()
        );
        unsafe { *self.inner.get_unchecked(index) }
    }

    #[inline(always)]
    pub fn insert_at(&mut self, index: usize, v: T) {
        self.inner[index] = v;
    }

    pub fn free_items(&mut self) {
        self.inner.clear();
    }
}
