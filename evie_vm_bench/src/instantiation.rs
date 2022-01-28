static SOURCE: &str = r#"
// This benchmark stresses instance creation and initializer calling.

class Foo {
  init() {}
}

var start = clock();
var i = 0;
while (i < _COUNT_) {
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  Foo();
  i = i + 1;
}


"#;

pub fn src(count: usize) -> String {
    SOURCE.replace("_COUNT_", &count.to_string())
}
