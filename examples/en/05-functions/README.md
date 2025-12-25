# 📦 Functions in DataCode

This section contains examples of creating and using functions.

## 📋 Contents

### 1. `simple_functions.dc` - Simple Functions
**Description**: Demonstrates creating and using basic functions.

**What you'll learn**:
- Function declaration: `fn name(parameters) { ... }`
- Returning values: `return`
- Calling functions
- Functions with and without parameters

**Run**:
```bash
cargo run examples/en/05-functions/simple_functions.dc
```

### 2. `recursion.dc` - Recursion
**Description**: Demonstrates recursive functions.

**What you'll learn**:
- Recursive function calls
- Base cases for recursion
- Examples: factorial, Fibonacci numbers, sum of numbers

**Run**:
```bash
cargo run examples/en/05-functions/recursion.dc
```

### 3. `nested_functions.dc` - Nested Functions
**Description**: Demonstrates using functions as arguments to other functions.

**What you'll learn**:
- Calling functions in expressions
- Functions as arguments to other functions
- Function composition

**Run**:
```bash
cargo run examples/en/05-functions/nested_functions.dc
```

## 🎯 Concepts Covered

### Function Declaration
```dc
fn function_name(parameter1, parameter2) {
    // function code
    return value
}
```

### Returning Values
- `return value` - returns a value from the function
- Function can return any value (number, string, boolean)

### Recursion
A recursive function calls itself. It's important to define a base case to avoid infinite recursion.

### Scope
- Function parameters are accessible only inside the function
- Local variables in functions are not visible outside
- Global variables are accessible from functions

## 🔗 Navigation

### Next Steps
After learning functions, move on to:
- **[07-loops](../07-loops/)** - while and for loops
- **[04-advanced](../04-advanced/)** - advanced programming techniques

---

**Learn functions!** 🚀

