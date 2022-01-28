static SOURCE: &str = r#"
class Zoo {
    init() {
      this.aarvark  = 1;
      this.baboon   = 1;
      this.cat      = 1;
      this.donkey   = 1;
      this.elephant = 1;
      this.fox      = 1;
    }
    ant()    { return this.aarvark; }
    banana() { return this.baboon; }
    tuna()   { return this.cat; }
    hay()    { return this.donkey; }
    grass()  { return this.elephant; }
    mouse()  { return this.fox; }
  }
  
  var zoo = Zoo();
  var sum = 0;
  var start = clock();
  while (sum < _COUNT_) {
    sum = sum + zoo.ant()
              + zoo.banana()
              + zoo.tuna()
              + zoo.hay()
              + zoo.grass()
              + zoo.mouse();
  }
  
  
"#;

pub fn src(count: usize) -> String {
    SOURCE.replace("_COUNT_", &count.to_string())
}
