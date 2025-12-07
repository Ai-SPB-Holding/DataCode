# 🎪 DataCode Demonstrations

This section contains comprehensive demonstrations showing all DataCode language capabilities in action.

## 📋 Contents

### 1. `showcase.dc` - Complete Language Demonstration
**Description**: Comprehensive example demonstrating all main DataCode capabilities in one file.

**What is demonstrated**:
- All data types
- User-defined functions
- Conditional constructs
- Loops and iterations
- Working with arrays
- Table operations
- Recursive algorithms
- Error handling
- Built-in functions
- File operations

**Goal**: Show the full power and capabilities of DataCode language in a real usage scenario.

## 🎯 How to Run

```bash
# Complete demonstration (may take time)
cargo run examples/07-demonstrations/showcase.dc

# If DataCode is installed globally
datacode examples/07-demonstrations/showcase.dc
```

## 📚 What showcase.dc Includes

### Basic Capabilities
- Variable declarations of all types
- Arithmetic and logical operations
- String manipulation

### Functions and Algorithms
- User-defined functions with parameters
- Recursive algorithms (factorial, Fibonacci)
- Higher-order functions

### Data Structures
- Arrays (simple and nested)
- Mixed data types
- Indexing and manipulations

### Conditional Logic
- Simple and complex conditions
- Nested conditions
- Logical operators

### Loops and Iterations
- For...in loops
- Array iteration
- Enumeration with indices

### Data Processing
- CSV file loading
- Table operations
- Filtering and sorting
- Data aggregation

### Built-in Functions
- Mathematical functions
- String operations
- Array functions
- File operations

## 🔍 Demonstration Structure

### Section 1: Basics
```datacode
# Variables and data types
global name = 'DataCode'
global version = 1.0
global active = true
```

### Section 2: Functions
```datacode
# User-defined functions
global function greet(name) do
    return 'Hello, ' + name + '!'
endfunction
```

### Section 3: Algorithms
```datacode
# Recursive algorithms
global function fibonacci(n) do
    if n <= 1 do
        return n
    endif
    return fibonacci(n-1) + fibonacci(n-2)
endfunction
```

### Section 4: Data Structures
```datacode
# Arrays and tables
global data = [1, 2, 3, 'mixed', true]
global matrix = [[1, 2], [3, 4]]
```

### Section 5: Data Processing
```datacode
# Working with CSV and tables
global employees = read_file('data.csv')
global filtered = table_select(employees, ['name', 'salary'])
```

## ⚡ Performance

**Warning**: `showcase.dc` performs many operations and may take significant time, especially:
- Recursive computations
- Operations with large arrays
- File loading and processing
- Complex table operations

## 💡 How to Use showcase.dc

### For Learning
1. **Read code by sections** - don't try to understand everything at once
2. **Run in parts** - comment out complex sections
3. **Experiment** - change parameters and values
4. **Study output** - analyze execution results

### For Demonstration
1. **Show capabilities** - use as language presentation
2. **Explain concepts** - comment on key points
3. **Compare with other languages** - show unique features
4. **Emphasize simplicity** - note code readability

### For Testing
1. **Check installation** - ensure everything works
2. **Test performance** - measure execution time
3. **Find bottlenecks** - identify slow operations
4. **Optimize code** - improve performance

## 🔗 Navigation

### All Previous Sections
Showcase.dc combines concepts from all sections - make sure you've studied them before running:
- **[01-basics](../01-basics/)** - basic operations and variables
- **[02-language-syntax](../02-language-syntax/)** - functions, conditions, and loops
- **[05-data-types](../05-data-types/)** - type system and isinstance()
- **[04-data-processing](../04-data-processing/)** - table operations and CSV
- **[03-advanced-features](../03-advanced-features/)** - recursion and error handling
- **[06-developer-tools](../06-developer-tools/)** - debugging and testing

### Additional Resources
- **[../INDEX.md](../INDEX.md)** - 📋 Quick index of all examples
- **[../README.md](../README.md)** - 📚 Main examples page
- **[../../README.md](../../README.md)** - 🏠 Main DataCode documentation

## 📈 Learning Recommendations

### Before running showcase.dc, study:
1. **Basics** - variables and functions
2. **Syntax** - conditions and loops
3. **Data types** - type system
4. **Data processing** - table operations

### After studying showcase.dc:
1. **Create your project** - use learned concepts
2. **Optimize code** - improve performance
3. **Add functionality** - extend capabilities
4. **Share experience** - help others learn DataCode

## ⚠️ Important Notes

- **Execution time**: May be significant due to complexity
- **Resources**: Consumes memory and CPU time
- **Dependencies**: May require data files
- **Debugging**: Use in parts to understand issues

## 🎓 Educational Value

Showcase.dc serves as:
- **Learning guide** - demonstrates all capabilities
- **Reference** - shows correct usage
- **Performance test** - checks system capabilities
- **Inspiration source** - gives ideas for your own projects

---

**Showcase.dc is the culmination of learning DataCode!** 🎪✨
