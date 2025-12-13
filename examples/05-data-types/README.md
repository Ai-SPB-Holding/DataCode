# 🔢 Data Types in DataCode

This section demonstrates the DataCode type system, type checking, conversions, and working with multiple variables.

## 📋 Contents

### 1. `type_checking_demo.dc` - Type Checking
**Description**: Demonstrates `isinstance()` function for data type checking.

**What you'll learn**:
- `isinstance(value, type)` function
- Type checking in conditions
- Various DataCode data types
- Safe type handling

**Supported types**:
- `integer` - integers
- `float` - floating-point numbers
- `string` - strings
- `boolean` - boolean values
- `array` - arrays
- `table` - tables
- `path` - file system paths

**Usage example**:
```datacode
global value = 42
if isinstance(value, int) do
    print('This is an integer')
endif
```

### 2. `type_conversion_guide.dc` - Type Conversion Guide
**Description**: Shows various ways to convert between data types.

**What you'll learn**:
- Automatic type conversion
- Explicit conversion
- Type compatibility
- Conversion error handling

### 3. `multiple_variables_demo.dc` - Working with Multiple Variables
**Description**: Demonstrates working with several variables simultaneously.

**What you'll learn**:
- Multiple variable declarations
- Group operations
- Comparing variables of different types
- Bulk assignments

### 4. `simple_multiple_vars_demo.dc` - Simple Multiple Variables
**Description**: Simplified version of working with multiple variables for beginners.

**What you'll learn**:
- Basic operations with multiple variables
- Simple comparisons
- Data grouping basics

### 5. `type_conversion_functions.dc` - Type Conversion Functions
**Description**: Comprehensive demonstration of type conversion functions including int(), float(), bool(), str(), date(), money(), and typeof().

**What you'll learn**:
- Converting to integers with `int()`
- Converting to floats with `float()`
- Converting to booleans with `bool()`
- Converting to strings with `str()`
- Converting to dates with `date()`
- Converting to currency with `money()`
- Getting type information with `typeof()`
- Practical examples of type conversion

## 🎯 How to Run Examples

```bash
# Type checking
cargo run examples/05-data-types/type_checking_demo.dc

# Type conversion
cargo run examples/05-data-types/type_conversion_guide.dc

# Multiple variables
cargo run examples/05-data-types/multiple_variables_demo.dc

# Simple multiple variables
cargo run examples/05-data-types/simple_multiple_vars_demo.dc

# Type conversion functions
cargo run examples/05-data-types/type_conversion_functions.dc
```

## 📚 DataCode Type System

### Basic Types
```datacode
global num_int = 42              # integer
global num_float = 3.14          # float
global text = 'Hello'            # string
global flag = true               # boolean
global list = [1, 2, 3]          # array
global file_path = path('/tmp')   # path
```

### Type Checking
```datacode
global function check_type(value) do
    if isinstance(value, int) do
        return 'Integer'
    else
        if isinstance(value, float) do
            return 'Floating-point number'
        else
            if isinstance(value, str) do
                return 'String'
            else
                return 'Other type'
            endif
        endif
    endif
endfunction
```

### Mixed Types in Arrays
```datacode
global mixed = [1, 'text', true, 3.14, [1, 2]]
# DataCode supports mixed types in arrays
```

## ⚠️ Typing Features

### Flexible Typing
- DataCode automatically detects types
- Supports mixed types in arrays
- Warns about incompatible operations

### Numeric Types
- `Integer` and `Float` are considered compatible
- Automatic conversion when needed
- No warnings when mixing Integer/Float

### Special Types
- **Date**: Strings in date format (e.g., '12/9/2019')
- **Currency**: Strings with currency symbols
- **Path**: Special type for file paths
- **Table**: Tabular data from CSV/Excel

## 🔍 isinstance() Function

### Syntax
```datacode
isinstance(value, type_name)
```

### Usage Examples
```datacode
global a = 42
global b = 'hello'
global c = [1, 2, 3]

print(isinstance(a, int))  # true
print(isinstance(b, str))   # true
print(isinstance(c, array))    # true
print(isinstance(a, str))   # false
```

### Practical Application
```datacode
global function safe_add(a, b) do
    if isinstance(a, int) and isinstance(b, int) do
        return a + b
    else
        print('Error: integers expected')
        return 0
    endif
endfunction
```

## 💡 Practical Tips

1. **Use isinstance()** for safe type handling
2. **Check types** before performing operations
3. **Mixed arrays** - powerful DataCode capability
4. **Automatic typing** works for most cases
5. **Test edge cases** with different types

## 🔗 Navigation

### Previous Sections
- **[01-basics](../01-basics/)** - basic data types
- **[02-language-syntax](../02-language-syntax/)** - arrays and functions

### Next Sections
- **[04-data-processing](../04-data-processing/)** - table types and data processing
- **[03-advanced-features](../03-advanced-features/)** - error handling with types

### Additional Resources
- **[../INDEX.md](../INDEX.md)** - 📋 Quick index of all examples
- **[../README.md](../README.md)** - 📚 Main examples page

## 📈 Recommended Learning Order

1. **`simple_multiple_vars_demo.dc`** - start with simple
2. **`type_checking_demo.dc`** - learn isinstance()
3. **`multiple_variables_demo.dc`** - complex scenarios
4. **`type_conversion_guide.dc`** - type conversions
5. **`type_conversion_functions.dc`** - explicit type conversion functions

## ⚡ Important Points

- **isinstance() is critically important** for professional development
- **Mixed types** are supported but require caution
- **Numeric types** (Integer/Float) are compatible
- **Type checking** helps avoid runtime errors

---

**Understanding data types is the foundation of reliable code in DataCode!** 🔢✨
