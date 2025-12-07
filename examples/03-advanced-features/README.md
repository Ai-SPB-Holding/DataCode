# 🎯 DataCode Advanced Features

This section demonstrates advanced programming concepts in DataCode: recursion, error handling, and functional methods.

## 📋 Contents

### 1. `recursion.dc` - Recursive Functions
**Description**: Demonstrates recursive function for calculating factorial with intensive computations.

**What you'll learn**:
- Recursive functions
- Base cases for recursion
- Performance of recursive algorithms

**Expected result**: Calculation of very large numbers (factorial(100)³)

**Step-by-step explanation**:
1. `factorial(n)` - recursive factorial function
2. Base case: `if n <= 1 do return 1`
3. Recursive case: `return n * factorial(n - 1)`
4. Intensive computation: `factorial(100) * factorial(100) * factorial(100)`

### 2. `simple_recursion.dc` - Simple Recursion Examples
**Description**: Simpler and clearer examples of recursive algorithms.

**What you'll learn**:
- Recursion basics
- Classic recursive algorithms
- Debugging recursive functions

### 3. `error_handling.dc` - Error Handling
**Description**: Comprehensive demonstration of various error types and methods to prevent them.

**What you'll learn**:
- Error types in DataCode
- Safe functions
- Input data validation
- Edge cases

**Expected output**:
```
⚠️  Error handling demonstration
=================================
✅ Correct operations:
x + y = 15
❌ Error examples (commented for demonstration):
...
🔧 Correct edge case handling:
safe_divide(10, 2) = 5
safe_divide(10, 0) = 0
Warning: division by zero!
```

**Step-by-step explanation**:
1. **Correct operations** - demonstration of correct code
2. **Error examples** - commented examples of various errors
3. **Safe functions** - functions with input data validation
4. **Error types** - description of all DataCode error types

### 4. `functional_methods_demo.dc` - Functional Methods
**Description**: Demonstrates functional programming methods (map, filter, reduce).

**What you'll learn**:
- Functional programming
- Higher-order methods
- Collection processing
- Functional patterns

## 🎯 How to Run Examples

```bash
# Recursive functions (may take time)
cargo run examples/03-advanced-features/recursion.dc

# Simple recursion
cargo run examples/03-advanced-features/simple_recursion.dc

# Error handling
cargo run examples/03-advanced-features/error_handling.dc

# Functional methods
cargo run examples/03-advanced-features/functional_methods_demo.dc
```

## 📚 Key Concepts

### Recursion
```datacode
global function factorial(n) do
    if n <= 1 do
        return 1
    endif
    return n * factorial(n - 1)
endfunction
```

### Safe Functions
```datacode
global function safe_divide(a, b) do
    if b == 0 do
        print('Warning: division by zero!')
        return 0
    else
        return a / b
    endif
endfunction
```

### Data Validation
```datacode
global function validate_input(value) do
    if value < 0 do
        print('Warning: negative value')
        return false
    else
        return true
    endif
endfunction
```

## ⚠️ Important Features

1. **Recursion**: Always define base case for stopping
2. **Performance**: Deep recursion can be slow
3. **Error handling**: Check input data before processing
4. **Edge cases**: Consider special values (0, null, empty arrays)

## 🔍 Error Types in DataCode

1. **SyntaxError** - syntax errors
2. **UndefinedVariable** - undefined variables
3. **TypeError** - type errors
4. **UndefinedFunction** - undefined functions
5. **ArgumentError** - incorrect arguments
6. **ScopeError** - scope errors
7. **ParseError** - parsing errors

## 💡 Practical Tips

1. **Recursion**: Start with simple examples, then move to complex
2. **Debugging**: Add print() to track execution
3. **Testing**: Check edge cases
4. **Performance**: Avoid deep recursion (>100 levels)

## 🔗 Navigation

### Previous Sections
- **[01-basics](../01-basics/)** - basic language concepts
- **[02-language-syntax](../02-language-syntax/)** - functions and conditions
- **[05-data-types](../05-data-types/)** - type system

### Related Sections
- **[04-data-processing](../04-data-processing/)** - applying advanced techniques in data processing
- **[06-developer-tools](../06-developer-tools/)** - debugging complex code

### Final Section
- **[07-demonstrations](../07-demonstrations/)** - comprehensive demonstrations of all capabilities

### Additional Resources
- **[../INDEX.md](../INDEX.md)** - 📋 Quick index of all examples
- **[../README.md](../README.md)** - 📚 Main examples page

## ⚡ Warnings

- `recursion.dc` may take long to execute due to intensive computations
- Some examples in `error_handling.dc` are commented to prevent errors
- Functional methods require understanding of higher-order concepts

---

**Advanced features open new horizons in DataCode!** 🧠✨
