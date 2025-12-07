# 📋 DataCode Examples Index

Quick index of all examples with brief descriptions and links for convenient navigation through the professionally organized example structure.

## 🎯 Recommended Learning Order

| # | File | Description | Difficulty | Section |
|---|------|----------|-----------|--------|
| 1 | [01-basics/simple.dc](01-basics/simple.dc) | Variables and arithmetic | ⭐ | Basics |
| 2 | [01-basics/hello.dc](01-basics/hello.dc) | Basic language capabilities | ⭐ | Basics |
| 3 | [02-language-syntax/functions.dc](02-language-syntax/functions.dc) | User-defined functions | ⭐⭐ | Syntax |
| 4 | [02-language-syntax/conditionals.dc](02-language-syntax/conditionals.dc) | Conditional constructs | ⭐⭐ | Syntax |
| 5 | [02-language-syntax/loops.dc](02-language-syntax/loops.dc) | For...in loops | ⭐⭐ | Syntax |
| 6 | [02-language-syntax/arrays_example.dc](02-language-syntax/arrays_example.dc) | Working with arrays | ⭐⭐ | Syntax |
| 7 | [05-data-types/type_checking_demo.dc](05-data-types/type_checking_demo.dc) | Type checking isinstance() | ⭐⭐ | Types |
| 8 | [04-data-processing/table_demo.dc](04-data-processing/table_demo.dc) | Working with tables and CSV | ⭐⭐⭐ | Data |
| 9 | [04-data-processing/filter_demo_simple.dc](04-data-processing/filter_demo_simple.dc) | Simple filtering | ⭐⭐ | Data |
| 10 | [04-data-processing/enum_demo.dc](04-data-processing/enum_demo.dc) | Enumeration with indices | ⭐⭐ | Data |
| 11 | [03-advanced-features/simple_recursion.dc](03-advanced-features/simple_recursion.dc) | Simple recursion | ⭐⭐ | Advanced |
| 12 | [03-advanced-features/error_handling.dc](03-advanced-features/error_handling.dc) | Error handling | ⭐⭐⭐ | Advanced |
| 13 | [02-language-syntax/complex_expressions.dc](02-language-syntax/complex_expressions.dc) | Complex expressions | ⭐⭐⭐ | Syntax |
| 14 | [03-advanced-features/recursion.dc](03-advanced-features/recursion.dc) | Recursive functions | ⭐⭐⭐ | Advanced |
| 15 | [06-developer-tools/interactive_demo.dc](06-developer-tools/interactive_demo.dc) | REPL functions | ⭐⭐ | Tools |
| 16 | [07-demonstrations/showcase.dc](07-demonstrations/showcase.dc) | All language capabilities | ⭐⭐⭐⭐ | Demonstrations |

## 📊 By Section (New Organization)

### 🚀 [01-basics/](01-basics/) - Language Basics
- [`simple.dc`](01-basics/simple.dc) - First steps with variables
- [`hello.dc`](01-basics/hello.dc) - Extended Hello World

### 🔧 [02-language-syntax/](02-language-syntax/) - Syntax Constructs
- [`functions.dc`](02-language-syntax/functions.dc) - User-defined functions
- [`conditionals.dc`](02-language-syntax/conditionals.dc) - Conditional constructs if/else/endif
- [`loops.dc`](02-language-syntax/loops.dc) - For...in loops
- [`arrays_example.dc`](02-language-syntax/arrays_example.dc) - Arrays and indexing
- [`basic_calculations.dc`](02-language-syntax/basic_calculations.dc) - Basic calculations
- [`complex_expressions.dc`](02-language-syntax/complex_expressions.dc) - Complex expressions

### 🎯 [03-advanced-features/](03-advanced-features/) - Advanced Techniques
- [`simple_recursion.dc`](03-advanced-features/simple_recursion.dc) - Simple recursion
- [`recursion.dc`](03-advanced-features/recursion.dc) - Complex recursive algorithms
- [`error_handling.dc`](03-advanced-features/error_handling.dc) - Error handling
- [`functional_methods_demo.dc`](03-advanced-features/functional_methods_demo.dc) - Functional methods

### 📊 [04-data-processing/](04-data-processing/) - Data Processing
- [`table_demo.dc`](04-data-processing/table_demo.dc) - Comprehensive table work
- [`filter_demo_simple.dc`](04-data-processing/filter_demo_simple.dc) - Simple filtering
- [`filter_demo_basic.dc`](04-data-processing/filter_demo_basic.dc) - Basic filtering
- [`data_filtering_demo.dc`](04-data-processing/data_filtering_demo.dc) - Advanced filtering
- [`enum_demo.dc`](04-data-processing/enum_demo.dc) - Enumeration with indices
- [`enum_table_example.dc`](04-data-processing/enum_table_example.dc) - Table enumeration

### 🔢 [05-data-types/](05-data-types/) - Type System
- [`type_checking_demo.dc`](05-data-types/type_checking_demo.dc) - Type checking isinstance()
- [`type_conversion_guide.dc`](05-data-types/type_conversion_guide.dc) - Type conversion
- [`multiple_variables_demo.dc`](05-data-types/multiple_variables_demo.dc) - Multiple variables
- [`simple_multiple_vars_demo.dc`](05-data-types/simple_multiple_vars_demo.dc) - Simple multiple variables

### 🛠️ [06-developer-tools/](06-developer-tools/) - Developer Tools
- [`interactive_demo.dc`](06-developer-tools/interactive_demo.dc) - Examples for REPL
- [`debug_mode_test.dc`](06-developer-tools/debug_mode_test.dc) - Debug mode
- [`stress_benchmark.dc`](06-developer-tools/stress_benchmark.dc) - Performance testing ⚠️

### 🎪 [07-demonstrations/](07-demonstrations/) - Comprehensive Demonstrations
- [`showcase.dc`](07-demonstrations/showcase.dc) - Complete demonstration of all capabilities

## 📚 Documentation

### Main Documentation
- **[README.md](README.md)** - Main examples page with full description
- **[../README.md](../README.md)** - Main DataCode project documentation

### Section Documentation
- **[01-basics/README.md](01-basics/README.md)** - 🚀 Language basics documentation
- **[02-language-syntax/README.md](02-language-syntax/README.md)** - 🔧 Syntax constructs
- **[03-advanced-features/README.md](03-advanced-features/README.md)** - 🎯 Advanced techniques
- **[04-data-processing/README.md](04-data-processing/README.md)** - 📊 Data processing
- **[05-data-types/README.md](05-data-types/README.md)** - 🔢 Type system
- **[06-developer-tools/README.md](06-developer-tools/README.md)** - 🛠️ Developer tools
- **[07-demonstrations/README.md](07-demonstrations/README.md)** - 🎪 Comprehensive demonstrations

## 🚀 Quick Start

```bash
# Start with simple example
cargo run examples/01-basics/simple.dc

# Extended Hello World
cargo run examples/01-basics/hello.dc

# Try interactive mode
cargo run
DataCode> print('Hello, DataCode!')

# Study functions
cargo run examples/02-language-syntax/functions.dc

# Working with tables and data
cargo run examples/04-data-processing/table_demo.dc

# Complete demonstration of all capabilities
cargo run examples/07-demonstrations/showcase.dc
```

## 💡 Learning Tips

- **Beginners**: start with section `01-basics/` (`simple.dc` → `hello.dc`)
- **Programmers**: move to `02-language-syntax/` (`functions.dc` → `conditionals.dc`)
- **For data work**: study section `04-data-processing/` and its detailed documentation
- **To learn all capabilities**: complete learning with section `07-demonstrations/showcase.dc`
- **For debugging and development**: use tools from `06-developer-tools/`

## 🎓 Structured Learning

Follow the recommended section order:
**01-basics** → **02-language-syntax** → **05-data-types** → **04-data-processing** → **03-advanced-features** → **06-developer-tools** → **07-demonstrations**

---
*Updated: 2025-07-15 | Professional DataCode examples organization*
