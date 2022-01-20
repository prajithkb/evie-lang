static SOURCE: &str = r#"
fun fib(n) {
    if (n < 2) return n;
    return fib(n - 2) + fib(n - 1);
  }
  
  var start = clock();
  print fib(_COUNT_);
  print clock() - start;
"#;

pub fn src(count: usize) -> String {
    SOURCE.replace("_COUNT_", &count.to_string())
}
