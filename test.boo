// doubles a value if it is greater than 10
fun double_if_greater_than_10(num x) -> num {
  if (x > 10) {
    return x + x;
  }

  return x;
}

// should return 80
num result = double_if_greater_than_10(40);
print(result);

// doubles a value if it is greater or equal to 10
fun double_if_greater_or_equal_10(num number) -> num {
  if (number >= 10) {
    return number + number;
  }

  return number;
}

// should return 20
num result2 = double_if_greater_or_equal_10(10);
print(result2);

// assignment operator tests
num test_x_assign = 10;

test_x_assign += 10; // should be 20
print(test_x_assign);

test_x_assign -= 5; // should be 15
print(test_x_assign);

test_x_assign *= 2; // should be 30
print(test_x_assign);

test_x_assign /= 10; // should be 3
print(test_x_assign);

// this should be true
bool is_equal = test_x_assign == 3;
print(is_equal);

num complex_expression = 10 + (150 * (130 + 12)) / 2;
print(complex_expression);

num precedence_test1 = 5 + 3 * 2; // should be 11
print(precedence_test1);

num precedence_test2 = (5 + 3) * 2; // should be 16
print(precedence_test2);

num precedence_test3 = 10 / 2 + 3; // should be 8
print(precedence_test3);

num precedence_test4 = 10 / (2 + 3); // should be 2
print(precedence_test4);

num test_power = 10 ** 2; // should be 100
print(test_power);

num test_mod = 10 % 3; // should be 1
print(test_mod);

test_power **= 2; // should be 10000
print(test_power);

test_power %= 15; // should be 10
print(test_power);

str test_str = "Hello World";
print(test_str);
print(test_str.len().to_string().len());

fun fibonacci(num n) -> num {
  if (n <= 0) {
    return 0;
  }

  if (n <= 2) {
    return 1;
  }

  return fibonacci(n - 1) + fibonacci(n - 2);
}

num fib = fibonacci(10);
print(fib);

str result2323 = fibonacci(10).to_string().len().to_string();
print(result2323);

fun factorial(num n) -> num {
  if (n <= 1) {
    return 1;
  }

  return n * factorial(n - 1);
}

print("Running comprehensive language test");
print("==================================");

print("1. Testing mathematical operations and precedence");
num math_test1 = 15 + 5 * 10 / (2 + 3) - 7;
print("15 + 5 * 10 / (2 + 3) - 7 = " >< math_test1);

num math_test2 = 2 ** 3 ** 2;  // Should follow right-to-left for power
print("2 ** 3 ** 2 = " >< math_test2);

num math_test3 = 100 % 7 * 5 + 3;
print("100 % 7 * 5 + 3 = " >< math_test3);

print("2. Testing compound assignments");
num test_var = 10;
print("Initial value: " >< test_var);

test_var += 5;
print("After += 5: " >< test_var);

test_var *= 2;
print("After *= 2: " >< test_var);

test_var /= 5;
print("After /= 5: " >< test_var);

test_var **= 2;
print("After **= 2: " >< test_var);

// Test conditional logic
print("3. Testing conditional logic");
num x = 42;
if (x > 50) {
  print("x is greater than 50");
} else {
  print("x is 50 or less");
}

// Test string concatenation and manipulation
print("4. Testing string operations");
str greeting = "Hello";
str name = "World";
str full_greeting = greeting >< " " >< name >< "!";
print(full_greeting);
print("Length of greeting: " >< full_greeting.len());

// Test factorial function
print("5. Testing factorial function");
num n = 5;
print("Factorial of " >< n >< ": " >< factorial(n));

// Basic arithmetic operators

// Addition
print("EXPECTED: 5   | " >< (2 + 3))

// Subtraction
print("EXPECTED: 1   | " >< (3 - 2))

// Multiplication
print("EXPECTED: 20  | " >< (4 * 5))

// Division
print("EXPECTED: 5   | " >< (10 / 2))

// Exponentiation (Power)
print("EXPECTED: 8   | " >< (2 ** 3))

// Modulo
print("EXPECTED: 1   | " >< (10 % 3))

// Operator precedence tests

// Multiplication before addition (2 + (3 * 4))
print("EXPECTED: 14  | " >< (2 + 3 * 4))

// Parentheses override default precedence ((2 + 3) * 4)
print("EXPECTED: 20  | " >< ((2 + 3) * 4))

// Associativity and chained operations

// Addition is left-associative
print("EXPECTED: 10  | " >< (1 + 2 + 3 + 4))

// Subtraction is left-associative
print("EXPECTED: -8  | " >< (1 - 2 - 3 - 4))

// Multiplication chaining
print("EXPECTED: 24  | " >< (1 * 2 * 3 * 4))

// Division chaining (left-to-right)
print("EXPECTED: 8   | " >< (64 / 4 / 2))

// Exponentiation is usually right-associative
// 2 ** 3 ** 2 is interpreted as 2 ** (3 ** 2) = 2 ** 9 = 512
print("EXPECTED: 512 | " >< (2 ** 3 ** 2))

// Modulo chaining (left-associative)
print("EXPECTED: 1   | " >< ((10 % 3) % 2))

// Complex expressions with mixed operators

// Combine multiplication, division, power, and modulo in one expression
// (2 * 3) + (4 / 2) + (2 ** 2) - (10 % 3)
// => 6 + 2 + 4 - 1 = 11
print("EXPECTED: 11  | " >< ((2 * 3) + (4 / 2) + (2 ** 2) - (10 % 3)))

// Tests involving negative numbers

// Simple negative addition
print("EXPECTED: -1  | " >< (-3 + 2))

// How negative numbers interact with power operator:
// (-2) raised to the 4th power (using parentheses) should give 16.
print("EXPECTED: 16  | " >< ((-2) ** 4))

// Without parentheses, the unary minus might apply after exponentiation,
// so -2 ** 4 might be interpreted as -(2 ** 4) = -16.
print("EXPECTED: -16 | " >< (-2 ** 4))

// Floating point division check
print("EXPECTED: 2.5 | " >< (5 / 2))