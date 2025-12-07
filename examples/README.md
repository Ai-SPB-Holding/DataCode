# DataCode Examples 📚

This directory contains professionally organized example programs in DataCode language, demonstrating all language capabilities from basic concepts to advanced techniques.

> **DataCode** - a simple interactive programming language for fast data processing with support for table functions, user-defined functions, loops, and conditional constructs.

## 🚀 Quick Start

```bash
# Run simplest example
cargo run examples/01-basics/simple.dc

# Extended Hello World
cargo run examples/01-basics/hello.dc

# Interactive mode
cargo run
DataCode> print('Hello, DataCode!')
```

## 📁 Example Organization

Examples are organized into thematic sections for convenient learning:

### 🚀 [01-basics](01-basics/) - Language Basics
**Start learning here!** Basic concepts and simplest examples.

- **`simple.dc`** - Simplest example with variables and arithmetic
- **`hello.dc`** - Extended Hello World with main capabilities

**[📖 Detailed Documentation](01-basics/README.md)**

### 🔧 [02-language-syntax](02-language-syntax/) - Syntax Constructs
Learn main language constructs: functions, conditions, arrays.

- **`functions.dc`** - User-defined functions with parameters
- **`conditionals.dc`** - Conditional constructs (if/else/endif)
- **`arrays_example.dc`** - Working with arrays and indexing
- **`complex_expressions.dc`** - Complex expressions and operators
- **`loops.dc`** - Basic calculations and operations

**[📖 Detailed Documentation](02-language-syntax/README.md)**

### 🎯 [03-advanced-features](03-advanced-features/) - Advanced Techniques
Recursion, error handling, and functional programming.

- **`recursion.dc`** - Recursive functions (intensive calculations)
- **`simple_recursion.dc`** - Simple recursion examples
- **`error_handling.dc`** - Error handling and validation
- **`functional_methods_demo.dc`** - Functional methods

**[📖 Detailed Documentation](03-advanced-features/README.md)**

### 📊 [04-data-processing](04-data-processing/) - Data Processing
Powerful capabilities for processing tabular data and CSV files.

- **`table_demo.dc`** - Comprehensive table work
- **`data_filtering_demo.dc`** - Data filtering
- **`filter_demo_basic.dc`** - Basic filtering
- **`filter_demo_simple.dc`** - Simple filtering
- **`enum_demo.dc`** - Enumeration with indices
- **`enum_table_example.dc`** - Tabular data enumeration

**Data files**: `sample_data.csv`, `clean_data.csv`, `simple.csv`

**[📖 Detailed Documentation](04-data-processing/README.md)**

### 🔢 [05-data-types](05-data-types/) - Type System
Learn about data types, type checking, and conversions.

- **`type_checking_demo.dc`** - isinstance() function and type checking
- **`type_conversion_guide.dc`** - Type conversion
- **`multiple_variables_demo.dc`** - Working with multiple variables
- **`simple_multiple_vars_demo.dc`** - Simple multiple variables

**[📖 Detailed Documentation](05-data-types/README.md)**

### 🛠️ [06-developer-tools](06-developer-tools/) - Developer Tools
Debugging, performance testing, and interactive development.

- **`debug_mode_test.dc`** - Debug mode and diagnostics
- **`interactive_demo.dc`** - Examples for REPL mode
- **`stress_benchmark.dc`** - Performance testing ⚠️

**[📖 Detailed Documentation](06-developer-tools/README.md)**

### 🎪 [07-demonstrations](07-demonstrations/) - Comprehensive Demonstrations
Full demonstrations of all language capabilities.

- **`showcase.dc`** - Complete demonstration of all DataCode capabilities

**[📖 Detailed Documentation](07-demonstrations/README.md)**

## 🎯 How to Run Examples

### File Execution
```bash
# Basics
cargo run examples/01-basics/simple.dc
cargo run examples/01-basics/hello.dc

# Language syntax
cargo run examples/02-language-syntax/functions.dc
cargo run examples/02-language-syntax/conditionals.dc

# Data processing
cargo run examples/04-data-processing/table_demo.dc

# Full demonstration
cargo run examples/07-demonstrations/showcase.dc

# Or if DataCode is installed as system command
datacode examples/01-basics/hello.dc
datacode examples/04-data-processing/table_demo.dc
```

### Interactive Mode (REPL)
```bash
# Start interactive mode
cargo run

# In REPL you can enter commands line by line:
DataCode> global x = 42
DataCode> print(x)
DataCode> global function square(n) do return n * n endfunction
DataCode> print(square(5))
```

## Language Capabilities

### ✅ Implemented Features

**Main Constructs:**
- **Variables**: `global var = value`, `local var = value`
- **Data types**: numbers, strings, booleans, arrays, tables, paths
- **Arithmetic operations**: `+`, `-`, `*`, `/`
- **Logical operations**: `and`, `or`, `not`
- **Comparison operators**: `>`, `<`, `>=`, `<=`, `==`, `!=`
- **Conditional constructs**: `if condition do ... else ... endif`
- **User-defined functions**: `global/local function name(params) do ... endfunction`
- **Loops**: `for item in array do ... next item`
- **Recursion**: functions can call themselves
- **Comments**: `# This is a comment`

**Built-in Functions (40+ functions):**
- **Basic**: `print()`, `now()`
- **Mathematical**: `abs()`, `sqrt()`, `pow()`, `min()`, `max()`, `round()`
- **String**: `length()`, `upper()`, `lower()`, `trim()`, `split()`, `join()`, `contains()`
- **Arrays**: `push()`, `pop()`, `unique()`, `reverse()`, `sort()`, `sum()`, `average()`, `count()`
- **File**: `getcwd()`, `path()`, `read_file()` (supports .txt, .csv, .xlsx)
- **Table**: `table()`, `show_table()`, `table_info()`, `table_head()`, `table_tail()`, `table_select()`, `table_sort()`

**Data Processing:**
- **CSV/Excel files**: automatic table creation with typing
- **Flexible typing**: automatic type detection with warnings
- **Beautiful table output**: ASCII formatting with borders

### 🔄 Limitations

- **Array syntax**: no literals `[1, 2, 3]` (use creation functions)
- **Indexing**: no index access `arr[0]`
- **Objects**: no syntax `{key: value}`
- **Nested conditions**: require careful use
- **While loops**: only `for...in` loops

## Example Structure

Each example contains:
- 📝 Comments explaining the code
- 🎯 Demonstration of specific capability
- ✅ Result verification
- 💡 Usage tips

## 📈 Recommended Learning Order

### Stage 1: Basics (required for everyone)
1. **[01-basics/simple.dc](01-basics/simple.dc)** - start here
2. **[01-basics/hello.dc](01-basics/hello.dc)** - basic capabilities

### Stage 2: Language Syntax
3. **[02-language-syntax/functions.dc](02-language-syntax/functions.dc)** - user-defined functions
4. **[02-language-syntax/conditionals.dc](02-language-syntax/conditionals.dc)** - conditional logic
5. **[02-language-syntax/arrays_example.dc](02-language-syntax/arrays_example.dc)** - arrays
6. **[02-language-syntax/complex_expressions.dc](02-language-syntax/complex_expressions.dc)** - complex expressions

### Stage 3: Data Types
7. **[05-data-types/type_checking_demo.dc](05-data-types/type_checking_demo.dc)** - type checking
8. **[05-data-types/multiple_variables_demo.dc](05-data-types/multiple_variables_demo.dc)** - multiple variables

### Stage 4: Data Processing
9. **[04-data-processing/table_demo.dc](04-data-processing/table_demo.dc)** - working with tables
10. **[04-data-processing/filter_demo_simple.dc](04-data-processing/filter_demo_simple.dc)** - filtering
11. **[04-data-processing/enum_demo.dc](04-data-processing/enum_demo.dc)** - enumeration

### Stage 5: Advanced Capabilities
12. **[03-advanced-features/simple_recursion.dc](03-advanced-features/simple_recursion.dc)** - simple recursion
13. **[03-advanced-features/error_handling.dc](03-advanced-features/error_handling.dc)** - error handling
14. **[03-advanced-features/recursion.dc](03-advanced-features/recursion.dc)** - complex recursion

### Stage 6: Tools and Demonstrations
15. **[06-developer-tools/interactive_demo.dc](06-developer-tools/interactive_demo.dc)** - REPL mode
16. **[07-demonstrations/showcase.dc](07-demonstrations/showcase.dc)** - full demonstration

## DataCode Programming Tips

### 🎯 Best Practices

- Use `global` for top-level variables
- Use `local` for variables inside functions
- Add comments to explain complex logic
- Check edge cases in functions
- Use descriptive names for variables and functions

### ⚠️ Common Mistakes

- Forgetting `do` after conditions and functions
- Not closing `endif` or `endfunction`
- Using undefined variables
- Wrong number of function arguments
- Attempting to use local variables outside function

### 🚀 Performance

- Avoid deep recursion (>100 levels)
- Use local variables in functions
- Cache results of complex calculations
- For large tables use `table_head()` and `table_tail()` instead of `show_table()`
- CSV files are automatically typed when loaded

## 📚 Additional Documentation

### Section Documentation
- **[01-basics/README.md](01-basics/README.md)** - 🚀 Language basics documentation
- **[02-language-syntax/README.md](02-language-syntax/README.md)** - 🔧 Syntax constructs
- **[03-advanced-features/README.md](03-advanced-features/README.md)** - 🎯 Advanced techniques
- **[04-data-processing/README.md](04-data-processing/README.md)** - 📊 Data processing
- **[05-data-types/README.md](05-data-types/README.md)** - 🔢 Type system
- **[06-developer-tools/README.md](06-developer-tools/README.md)** - 🛠️ Developer tools
- **[07-demonstrations/README.md](07-demonstrations/README.md)** - 🎪 Comprehensive demonstrations

### Additional Materials
- **[INDEX.md](INDEX.md)** - 📋 Quick index of all examples with navigation by new structure
- **[04-data-processing/TABLE_EXAMPLES.md](04-data-processing/TABLE_EXAMPLES.md)** - Detailed technical guide for working with tables

### Main Project Documentation
- **[../README.md](../README.md)** - Main DataCode project documentation
- **[../docs/DEMO_RESULTS.md](../docs/DEMO_RESULTS.md)** - Technical documentation on table function implementation
- **[../IMPLEMENTATION_REPORT.md](../IMPLEMENTATION_REPORT.md)** - Report on implemented features

## 🔗 Useful Links

- **Repository**: [DataCode on GitHub](https://github.com/igornet0/DataCode)
- **Installation**: Instructions in main README.md
- **Tests**: `tests/` folder contains automatic tests for all functions

## Feedback

If you found an error in examples or want to suggest an improvement, create an issue in the project repository.

---

**DataCode** - simple and powerful language for fast data processing! 🧠✨
