# 🔧 DataCode Language Syntax

This section demonstrates main syntax constructs of DataCode language: functions, conditions, loops, arrays, and complex expressions.

## 📋 Contents

### 1. `functions.dc` - User-defined Functions
**Description**: Demonstrates creation and use of user-defined functions with parameters and return values.

**What you'll learn**:
- Function declarations with `global function`
- Function parameters
- Local variables with `local`
- Return values with `return`

**Expected output**:
```
🔧 User-defined functions demonstration
=========================================
add(15, 25) = 40
greet("DataCode") = Hello, DataCode!
circle_area(5) = 78.53975
✅ All functions work correctly!
```

### 2. `conditionals.dc` - Conditional Constructs
**Description**: Complete demonstration of if/else/endif conditional operators with various scenarios.

**What you'll learn**:
- Basic conditions `if...do...endif`
- Conditions with `else`
- Logical operators `and`, `or`, `not`
- Nested conditions
- Comparison operators

### 3. `loops.dc` - For...in Loops
**Description**: Comprehensive demonstration of for...in loops with various data types and usage scenarios.

**What you'll learn**:
- Simple loops over arrays
- Loops over strings and mixed arrays
- Loops over array literals
- Nested loops
- Loops with conditions
- Loops with accumulation and calculations
- Using loops with functions

### 4. `basic_calculations.dc` - Basic Calculations
**Description**: Demonstrates basic arithmetic operations and calculations.

**What you'll learn**:
- Arithmetic operations
- Working with strings
- Complex expressions
- Logical operations

### 5. `arrays_example.dc` - Working with Arrays
**Description**: Comprehensive example of working with arrays, including creation, indexing, and built-in functions.

**What you'll learn**:
- Array literals `[1, 2, 3]`
- Mixed types in arrays
- Indexing `arr[0]`
- Nested arrays
- Array built-in functions
- Trailing comma

### 6. `complex_expressions.dc` - Complex Expressions
**Description**: Demonstrates complex arithmetic and logical expressions with operator precedence.

**What you'll learn**:
- Operator precedence
- Parentheses in expressions
- Combining operators
- Complex logic

### 7. `file_operations.dc` - File Operations
**Description**: Comprehensive demonstration of file operations including path(), list_files(), read_file(), analyze_csv(), and write_file().

**What you'll learn**:
- Working with file paths using `path()`
- Listing files with `list_files()`
- Reading files with `read_file()`
- Analyzing CSV files with `analyze_csv()`
- Writing files with `write_file()`
- Processing multiple files

### 8. `advanced_array_operations.dc` - Advanced Array Operations
**Description**: Demonstrates advanced array operations including reverse(), append(), array_builder(), bulk_create(), and extend().

**What you'll learn**:
- Reversing arrays with `reverse()`
- Appending elements with `append()`
- Creating arrays with initial capacity using `array_builder()`
- Bulk creation with `bulk_create()`
- Extending arrays with `extend()`
- Efficient array manipulation techniques

## 🎯 How to Run Examples

```bash
# User-defined functions
cargo run examples/02-language-syntax/functions.dc

# Conditional constructs
cargo run examples/02-language-syntax/conditionals.dc

# For...in loops
cargo run examples/02-language-syntax/loops.dc

# Basic calculations
cargo run examples/02-language-syntax/basic_calculations.dc

# Working with arrays
cargo run examples/02-language-syntax/arrays_example.dc

# Complex expressions
cargo run examples/02-language-syntax/complex_expressions.dc

# File operations
cargo run examples/02-language-syntax/file_operations.dc

# Advanced array operations
cargo run examples/02-language-syntax/advanced_array_operations.dc
```

## 📚 Key Concepts

### Functions
```datacode
global function name(param1, param2) do
    local result = param1 + param2
    return result
endfunction
```

### Conditions
```datacode
if condition do
    # actions
else
    # alternative actions
endif
```

### Arrays
```datacode
global arr = [1, 2, 3, 'mixed', true]
global element = arr[0]  # Indexing
global nested = [[1, 2], [3, 4]]
global value = nested[0][1]  # Nested indexing
```

### Logical Operators
- `and` - logical AND
- `or` - logical OR  
- `not` - logical NOT
- `>`, `<`, `>=`, `<=`, `==`, `!=` - comparisons

## ⚠️ Important Features

1. **Functions**: Always use `global` or `local` when declaring
2. **Local variables**: Available only inside function
3. **Conditions**: Don't forget `do` after condition and `endif` at the end
4. **Arrays**: Support mixed types and nesting
5. **Indexing**: Starts at 0

## 🔗 Recommended Learning Order

1. **`functions.dc`** - foundation for understanding functions
2. **`conditionals.dc`** - logic and conditions
3. **`loops.dc`** - loops and iterations
4. **`arrays_example.dc`** - data structures
5. **`basic_calculations.dc`** - basic calculations
6. **`complex_expressions.dc`** - complex expressions
7. **`file_operations.dc`** - working with files
8. **`advanced_array_operations.dc`** - advanced array manipulation

## 💡 Practical Tips

1. **Test functions** with different parameters
2. **Experiment with conditions** - try different values
3. **Study arrays gradually** - from simple to nested
4. **Use comments** to understand complex logic

## 🔗 Navigation

### Previous Section
- **[01-basics](../01-basics/)** - basic language concepts

### Next Sections
- **[05-data-types](../05-data-types/)** - type system and isinstance()
- **[04-data-processing](../04-data-processing/)** - applying syntax in data processing
- **[03-advanced-features](../03-advanced-features/)** - recursion and advanced techniques

### Additional Resources
- **[../INDEX.md](../INDEX.md)** - 📋 Quick index of all examples
- **[../README.md](../README.md)** - 📚 Main examples page

---

**Learning syntax is the key to mastering DataCode!** 🧠✨
