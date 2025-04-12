fun fibonacci(num n) -> num {
  if (n <= 0) {
    return 0;
  }

  if (n <= 2) {
    return 1;
  }

  return fibonacci(n - 1) + fibonacci(n - 2);
}

print("Result: " >< fibonacci(10)); // should output 55