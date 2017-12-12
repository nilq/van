# van

an attempt to make a tiny high-control language with sugar syntax

## TODO

- parse blocks

- parse types properly

- parse calls - Haskell style

- parse structs

- parse interfaces

- parse flow control

- errors

- semantics

- transpiler and compiler

## syntax

```
a: i32      = 10
mut b: char = '\n'
c := r"strong raw string"
```

```
fun fib n: i32 -> i128 {
  match n {
    | 0 -> 0
    | 1 -> 1
    | n -> fib (n - 1) + fib (n - 2)
  }
}
```

```
function fib -> i128 {
  | 0 -> 0
  | 1 -> 1
  | n -> fib (n - 1) + fib (n - 2)
}
```

```
interface Vector<T> {
  fun magnitude: self -> T
}

struct Point<T> {
  x: T
  y: T
}

impl<T> Vector<T> for Point<T> {
  fun magnitude: self -> T {
    math.sqrt ((self.x + self.y)^^2)
  }
}

mut pos := new Point<f32> {
  x: 100
  y: 100
}

length_of_point = pos.magnitude!
```

```
fun apply<A, B> a: A, f: fun(A) -> B {
  f a
}
```
