# 🚀 DataCode Basics

This section contains basic examples for learning the fundamentals of the DataCode programming language. Start learning the language with these examples.

## 📋 Contents

### 1. `simple.dc` - Simplest Example
**Description**: Demonstrates basic operations with variables and arithmetic.

**What you'll learn**:
- Global variable declarations
- Arithmetic operations
- `print()` output function

**Expected output**:
```
x = 10
y = 20
sum = 30
```

**Step-by-step explanation**:
1. `global x = 10` - declare global variable x with value 10
2. `global y = 20` - declare global variable y with value 20
3. `global sum = x + y` - calculate sum of x and y, save to variable sum
4. `print()` - output variable values to screen

### 2. `hello.dc` - Extended Hello World
**Description**: Comprehensive example demonstrating main language capabilities.

**What you'll learn**:
- Working with strings and numbers
- String concatenation
- Mathematical operations
- Logical operations
- Formatted output

**Expected output**:
```
🧠 Welcome to DataCode!
==============================
Programming Language: DataCode
Version: 1.0
Year: 2024

📊 Mathematical operations:
x = 10
y = 20
x + y = 30
x * y = 200

💬 Working with strings:
Hello, DataCode!

🔍 Logical operations:
sum > 0: true

✅ Program executed successfully!
```

**Step-by-step explanation**:
1. **Variable declarations**: Create variables to store language information
2. **Mathematical operations**: Perform addition and multiplication
3. **Working with strings**: Concatenate strings using `+` operator
4. **Logical operations**: Check condition `sum > 0`
5. **Formatted output**: Use emojis and separators for beautiful output

## 🎯 How to Run Examples

### Running from project root directory:
```bash
# Simplest example
cargo run examples/01-basics/simple.dc

# Extended Hello World
cargo run examples/01-basics/hello.dc
```

### If DataCode is installed globally:
```bash
datacode examples/01-basics/simple.dc
datacode examples/01-basics/hello.dc
```

## 📚 Concepts Covered

### Variables
- **Global variables**: `global var_name = value`
- **Data types**: numbers, strings, booleans
- **Automatic type detection**

### Operators
- **Arithmetic**: `+`, `-`, `*`, `/`
- **String**: concatenation with `+`
- **Logical**: `>`, `<`, `==`, `!=`

### Built-in Functions
- **`print()`**: output data to screen
- Support for multiple arguments

## ⚠️ Important Points

1. **Variable declarations**: Always use `global` for top-level variables
2. **Data types**: DataCode automatically detects types
3. **Strings**: Use single quotes `'text'`
4. **Comments**: Start with `#` symbol

## 🔗 Navigation

### Next Steps
After learning basics, move to:
- **[02-language-syntax](../02-language-syntax/)** - learning functions and conditions
- **[05-data-types](../05-data-types/)** - type system and type checking
- **[04-data-processing](../04-data-processing/)** - working with tables and data

### Additional Resources
- **[../INDEX.md](../INDEX.md)** - 📋 Quick index of all examples
- **[../README.md](../README.md)** - 📚 Main examples page
- **[../../README.md](../../README.md)** - 🏠 Main DataCode documentation

## 💡 Tips for Beginners

1. **Start with `simple.dc`** - this is the simplest example
2. **Experiment** - change variable values
3. **Read comments** - they explain each step
4. **Use REPL** - run `cargo run` for interactive mode

---

**Good luck learning DataCode!** 🧠✨
