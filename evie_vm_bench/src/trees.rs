static SOURCE: &str = r#"
class Tree {
    init(depth) {
      this.depth = depth;
      if (depth > 0) {
        this.a = Tree(depth - 1);
        this.b = Tree(depth - 1);
        this.c = Tree(depth - 1);
        this.d = Tree(depth - 1);
        this.e = Tree(depth - 1);
      }
    }
  
    walk() {
      if (this.depth == 0) return 0;
      return this.depth 
          + this.a.walk()
          + this.b.walk()
          + this.c.walk()
          + this.d.walk()
          + this.e.walk();
    }
  }
  
  var tree = Tree(_COUNT_);
  var start = clock();
  var i = 0;
  while (i < 10) {
    tree.walk();
    i = i+1;
  }
"#;

pub fn src(count: usize) -> String {
    SOURCE.replace("_COUNT_", &count.to_string())
}
