# Boo Language

## Overview
**Boo** is a simple, statically typed, imperative programming language built in **Rust**.

## Installation
Clone the repository:
```bash
$ git clone https://github.com/joaopugsley/boo-lang.git
```
Compile and run a ``.boo`` file:
```bash
# Note: if no filename is provided, it defaults to 'main.boo'
$ cargo run [filename]
```
⚠️ You need cargo installed to run Boo. If you don’t have it, follow [Rust's Installation Documentation](https://doc.rust-lang.org/book/ch01-01-installation.html)

## Example
```boo
fun fibonacci(num n) -> num {
  if (n <= 0) {
    return 0;
  }

  if (n <= 2) {
    return 1;
  }

  return fibonacci(n - 1) + fibonacci(n - 2);
}

print("Result: " >< fibonacci(10)); // => Result: 55
```
More examples can be found in the `examples` folder.