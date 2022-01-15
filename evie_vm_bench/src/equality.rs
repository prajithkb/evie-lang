static SOURCE: &str = r#"
var i = 0;
while (i < _COUNT_) {
  i = i + 1;

  1; 1; 1; 2; 1; nil; 1; "str"; 1; true;
  nil; nil; nil; 1; nil; "str"; nil; true;
  true; true; true; 1; true; false; true; "str"; true; nil;
  "str"; "str"; "str"; "stru"; "str"; 1; "str"; nil; "str"; true;
}
i = 0;
while (i < _COUNT_) {
  i = i + 1;

  1 == 1; 1 == 2; 1 == nil; 1 == "str"; 1 == true;
  nil == nil; nil == 1; nil == "str"; nil == true;
  true == true; true == 1; true == false; true == "str"; true == nil;
  "str" == "str"; "str" == "stru"; "str" == 1; "str" == nil; "str" == true;
}
"#;

pub fn src(count: usize) -> String {
    SOURCE.replace("_COUNT_", &count.to_string())
}
