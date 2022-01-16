# Evie 

**Evie** - a dynamic typed language inspired by Lox from [craftinginterpretors.com](craftinginterpreters.com) implemented in Rust. 

You can find my attempt to implement Lox in purely safe Rust [here](https://github.com/prajithkb/lox-rs). That implementation turned out to be 5 times slower than the corresponding C implementation due to memory safety restrictions imposed by Rust. For Evie I have used `unsafe` implementations when I thought performance was a concern but I have tried to keep those to a minimum. 

In the code base of x lines of code only y% of the code is unsafe.

## Why Evie?
This is a hobby project, with the intention of creating a ***highly performant*** language. 

## Evie features
The full grammer can be found [here](todo). 

1. Primitives 
   1. Nil
   2. String
   3. Number
   4. Boolean
2. Conditional (if/else)
3. Loop 
   1. while
4. Functions
5. Closures
6. Collection
7. Object
   


## Performance
Check [this](./performance_improvements.md) for all the different performance improvements. That explains how I improved the performance by nearly ten times!


