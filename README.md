# van

an attempt to make a tiny high-control language with a fair amount of sugar

## TODO

- compiler

- optimizer

- proper, advanced type inference

## syntax

### variables

strongly typed declarations

```
foo: int    = 100
foo: string = "yo world"
```

```
mut foo: string = "hello"
foo = foo ++ ", world"
```

```
foo := 100
mut foo := "mutable inferred string"
```

### funs

they are funny

```
fun foo {
  print "yoo"
}
```

```
fun same a: int {
  a
}

fun same a: string {
  return "explicit string grr: " ++ a
}

fun same a: int -> int {
  a
}
```

```
fun apply a: int f: fun int -> int -> int {
  f a
}

fun add10 a: int -> int {
  10 + a
}

a := apply 10 add10
```

### match

```
a := 10

match a {
  | 0 -> print "zero"
  | 1 -> print "one .."
  | n -> print "nvm another value"
}
```

### if

as statement

```
a := 10

if a == 0 {
  "zero"
} elif a == 1 {
  "one"
} else {
  print "neither one nor zero"
}
```

as expression

```
a := 10

b: string = if a == 0 {
  "zero"
} elif a == 1 {
  "one"
} else {
  "neither one nor zero"
}

print b
```

### functions

matching functions

```
function fib {
  | 0 -> 0
  | 1 -> 1
  | n -> fib (n - 1) + fib (n - 2)
}

function fib -> int {
  | 0 -> 0
  | 1 -> 1
  | n -> fib (n - 1) + fib (n - 2)
}
```

(basically the same as ..)
```
fun fib a: int -> int {
  match a {
    | 0 -> 0
    | 1 -> 1
    | n -> fib (n - 1) + fib (n - 2)
  }
}
```

### struct

```
struct Point {
  x: int
  y: int
}
```

```
pos: Point = new {
    x = 10
    y = 10
}

pos2 := new Point {
    x = 100
    y = 100
}
```

### interface

require function signatures on struct

```
interface Debug {
  debug: fun -> string
}
```

#### implementation

implement interfaces

```
implement Point as Debug {
  fun debug -> string {
    "insert debugged point"
  }
}
```

### arrays

trailing commas are important

```
a: [int; 3] = [1, 2, 3,]
b := [4, 3, 2, 1,]

c: int = a[0]
```

```
fun foo i: int -> int {
  return 1000 + i
}

weird: [fun int -> int; 1] = [foo]

c: int = weird[0] 10
```

### calls

calls are all haskell and nice, arguments are separated by whitespace, so parens will come
in handy when using calls as args.

with some exceptions calls will be parsed until it reaches and `)`, `]`, `,`, newline or an operator

```
foo (10 + 10) (10 + 10) + 1
```

context exception 

```
(foo 10 + 10)
```
