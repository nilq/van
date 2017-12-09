# van

an attempt to make a tiny high-control language with sugar syntax

## syntax

```
a: i32      = 10
mut b: char = '\n'
c := r"strong raw string"
```

```
fun fib: n -> i128 =
  match n of
    | 0 -> 0
    | 1 -> 1
    | n -> fib (n - 1) + fib (n - 2)
```

```
function fib -> i128 =
  | 0 -> 0
  | 1 -> 1
  | n -> fib (n - 1) + fib (n - 2)
```

```
interface Vector<T> =
  fun magnitude: self -> T

struct Point<T> =
  { x: T
    y: T
  }

impl<T> Vector<T> for Point<T> =
  fun magnitude: self -> T =
    math.sqrt ((self.x + self.y)^^2)

mut pos: Point<f32> =
  { x: 100
    y: 100
  }

length_of_point = pos.magnitude!
```
