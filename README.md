# 🧠 DataCode - Interactive Programming Language

<img width="935" height="451" alt="Screenshot 2025-07-15 at 16 32 23" src="https://github.com/user-attachments/assets/5f2d3745-dcf2-47cf-9caa-881c7d10ae71" />

**DataCode** is a simple, interactive programming language designed for fast data processing and easy learning. It features an intuitive syntax, powerful array support, built-in functions, and user-defined functions with local scope.

## 🚀 Features

- **Interactive REPL** with multiline support and command history
- **File execution** - write programs in `.dc` files
- **Array literals** - `[1, 2, 3]`, `['a', 'b']`, mixed types supported
- **Array indexing** - `arr[0]`, `nested[0][1]` with full nesting support
- **User-defined functions** with local scope, parameters and recursion
- **Conditional statements** - if/else/endif with nesting support
- **For loops** - iterate over arrays with `for...in`
- **Arithmetic and logical operations** with proper precedence
- **String manipulation** and concatenation
- **Table operations** - work with CSV/Excel files, automatic typing
- **40+ built-in functions** - math, string, array, file, and table operations
- **Path manipulation** for file system operations
- **Flexible data types** - numbers, strings, booleans, arrays, tables, paths
- **Improved error messages** with line numbers and context
- **Comment support** with `#`

## 📦 Installation

### Option 1: Global Installation (Recommended)
Install DataCode as a global command:

```bash
# Clone and install
git clone https://github.com/igornet0/DataCode.git
cd DataCode

# Install globally
make install
# or
./install.sh

# Now you can use datacode from anywhere!
datacode --help
```

### Option 2: Development Mode
Run directly with Cargo:

```bash
git clone https://github.com/igornet0/DataCode.git
cd DataCode
cargo run
```

## 🎯 Usage

### After Global Installation
```bash
datacode                   # Start interactive REPL
datacode filename.dc       # Execute DataCode file
datacode filename.dc --build_model  # Execute and export tables to SQLite
datacode filename.dc --build_model output.db  # Export to specific file
datacode --repl            # Start interactive REPL
datacode --demo            # Run demonstration
datacode --help            # Show help
```

### Development Mode
```bash
cargo run                  # Start interactive REPL
cargo run filename.dc      # Execute DataCode file
cargo run -- --help       # Show help

# Or use Makefile
make run                   # Start REPL
make examples              # Run all examples
make test                  # Run tests
```

### Quick Examples
```bash
# Create a simple DataCode file
echo 'print("Hello, DataCode!")' > hello.dc

# Create an array example
echo 'global arr = [1, 2, 3]
print("Array:", arr)
print("First element:", arr[0])' > arrays.dc

# Execute the files
datacode hello.dc          # (after global installation)
datacode arrays.dc
# or
cargo run hello.dc         # (development mode)
cargo run arrays.dc
```

### Programmatic Usage
```rust
use data_code::interpreter::Interpreter;

fn main() {
    let mut interp = Interpreter::new();
    interp.exec("global basePath = getcwd()").unwrap();
    interp.exec("global files = list_files(basePath / 'data')").unwrap();
}
```
---

## 📄 Language Syntax

### 🔹 Variables
```DataCode
global path = getcwd()
local subdir = 'data'
```
• `global` — stores variable globally
• `local` — limited to current context (e.g., loop)

### 🔹 Arithmetic Operations
```DataCode
global x = 10
global y = 20
global sum = x + y          # Addition
global diff = x - y         # Subtraction
global prod = x * y         # Multiplication
global quot = x / y       # Division
global complex = (x + y) * 2 - 5  # Complex expressions
```

### 🔹 Arrays
```DataCode
# Creating arrays of any types
global numbers = [1, 2, 3, 4, 5]
global strings = ['hello', 'world', 'datacode']
global booleans = [true, false, true]
global mixed = [1, 'hello', true, 3.14]
global empty = []

# Nested arrays
global matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
global nested_mixed = [[1, 'a'], [true, 3.14]]

# Accessing elements (0-based indexing)
print(numbers[0])        # 1
print(strings[1])        # world
print(mixed[2])          # true
print(matrix[0][1])      # 2
print(nested_mixed[1][0]) # true

# Trailing comma supported
global trailing = [1, 2, 3,]

# Using in loops
for item in [1, 2, 3] do
    print('Item:', item)
next item
```

### 🔹 Comparison Operators
```DataCode
global eq = x == y          # Equality
global ne = x != y          # Inequality
global gt = x > y           # Greater than
global lt = x < y           # Less than
global ge = x >= y          # Greater than or equal
global le = x <= y          # Less than or equal
```

### 🔹 Logical Operations
```DataCode
global flag1 = true
global flag2 = false
global and_result = flag1 and flag2    # Logical AND
global or_result = flag1 or flag2      # Logical OR
global not_result = not flag1          # Logical NOT
global complex_logic = (x > 5) and (y < 30) or flag1
```

### 🔹 Path Concatenation
```DataCode
global dir = basePath / 'data' / 'images'
```
• `/` is used for Path + String (contextually determined)

### 🔹 String Concatenation
```DataCode
global name = 'image' + '001.jpg'
global greeting = 'Hello, ' + name + '!'
```
• `+` concatenates strings

---

## 🔁 Loops
```DataCode
# Loop over array variable
for file in files do
    local path = basePath / 'data' / file
    local text = read_file(path)
    print('>>', file, 'length:', text)
next file

# Loop over array literal
for number in [1, 2, 3, 4, 5] do
    print('Number:', number, 'Squared:', number * number)
next number

# Loop over mixed array
for item in ['hello', 42, true] do
    print('Item:', item)
next item

# Loop over nested array
for row in [[1, 2], [3, 4], [5, 6]] do
    print('Row:', row, 'Sum:', sum(row))
next row
```
- `for x in array do ... next x` - iteration over array
- `x` — variable accessible inside loop body
- Both array variables and array literals are supported

---

## 🔧 Built-in Functions (40+)

### 📁 File Operations
| Function | Description |
|---------|----------|
| `getcwd()` | Current directory |
| `path(string)` | Create path from string |
| `read_file(path)` | Read files (.txt, .csv, .xlsx) |
| `read_file(path, sheet_name)` | Read XLSX with sheet selection by name |
| `read_file(path, header_row)` | Read CSV/XLSX with header row selection (0-based) |
| `read_file(path, header_row, sheet_name)` | Read XLSX with header row and sheet selection by name |

**Optional parameters for `read_file()`:**
- `sheet_name` (string) - sheet name for XLSX files (default: first sheet)
- `header_row` (number) - header row number, starting from 0 (default: 0)

**Examples:**
```datacode
# Basic reading
data = read_file(path("data.csv"))

# Read specific Excel sheet
data = read_file(path("report.xlsx"), "Sales")

# Read with header in row 2
data = read_file(path("data.csv"), 2)

# Combination: sheet + header row
data = read_file(path("report.xlsx"), 1, "DataSheet")
```

### 🧮 Mathematical Functions
| Function | Description |
|---------|----------|
| `abs(n)` | Absolute value |
| `sqrt(n)` | Square root |
| `pow(base, exp)` | Power |
| `min(...)` | Minimum value |
| `max(...)` | Maximum value |
| `round(n)` | Rounding |

### 📝 String Functions
| Function | Description |
|---------|----------|
| `length(str)` | String length |
| `upper(str)` | To uppercase |
| `lower(str)` | To lowercase |
| `trim(str)` | Trim whitespace |
| `split(str, delim)` | Split string |
| `join(array, delim)` | Join array |
| `contains(str, substr)` | Check substring |

### 📊 Array Functions
| Function | Description |
|---------|----------|
| `push(array, item)` | Add element |
| `pop(array)` | Remove last |
| `unique(array)` | Unique elements |
| `reverse(array)` | Reverse order |
| `sort(array)` | Sort |
| `sum(array)` | Sum numbers |
| `average(array)` | Average value |
| `count(array)` | Element count |

### 📋 Table Functions
| Function | Description |
|---------|----------|
| `table(data, headers)` | Create table |
| `show_table(table)` | Display table |
| `table_info(table)` | Table information |
| `table_head(table, n)` | First n rows |
| `table_tail(table, n)` | Last n rows |
| `table_select(table, cols)` | Select columns |
| `table_sort(table, col, asc)` | Sort table |

### 🔧 Utilities
| Function | Description |
|---------|----------|
| `print(...)` | Print values |
| `now()` | Current time |

---

## 🗄️ SQLite Export (--build_model)

DataCode supports automatic export of all tables from global variables to SQLite database with automatic dependency detection between tables.

### Main Features

- ✅ **Automatic table export** - all tables from global variables are exported to separate SQLite tables
- ✅ **Variable metadata** - creates `_datacode_variables` table with information about all global variables
- ✅ **Automatic dependency detection** - system automatically finds relationships between tables by ID columns
- ✅ **Index creation** - automatic creation of indexes for ID columns and foreign keys
- ✅ **Type conversion** - automatic conversion of DataCode types to SQLite types

### Usage

```bash
# Export with default name (script_name.db)
datacode load_model_data.dc --build_model

# Export with specified output file
datacode load_model_data.dc --build_model output.db

# Using environment variable
DATACODE_SQLITE_OUTPUT=model.db datacode load_model_data.dc --build_model
```

### Example Script

```datacode
# Load data
global sales = read_file("sales.csv")
global products = read_file("products.csv")
global customers = read_file("customers.csv")

# Process data
global sales_table = table(sales)
global products_table = table(products)
global customers_table = table(customers)

# Filter and transform
global filtered_sales = table_where(sales_table, "amount > 100")
```

Execution:
```bash
datacode load_model_data.dc --build_model
```

### Export Result

After execution, a SQLite database is created with the following tables:

1. **Data tables** - each global variable of type `Table` is exported to a separate table:
   - `sales_table` - all data from sales
   - `products_table` - all data from products
   - `customers_table` - all data from customers
   - `filtered_sales` - filtered data

2. **Metadata table `_datacode_variables`** - contains information about all global variables:
   ```sql
   CREATE TABLE _datacode_variables (
       variable_name TEXT PRIMARY KEY,
       variable_type TEXT NOT NULL,      -- Table, Array, Object, Number, String, etc.
       table_name TEXT,                   -- SQLite table name (for tables)
       row_count INTEGER,                 -- Row count (for tables)
       column_count INTEGER,              -- Column count (for tables)
       created_at TEXT,                   -- Export timestamp
       description TEXT,                  -- Description (optional)
       value TEXT                         -- String representation of value
   );
   ```

3. **Automatic dependencies** - if a table has columns with ID-like names (`*_id`, `id`), the system automatically detects relationships:
   - If `sales_table` has `product_id` and `products_table` has `id`, a relationship is created
   - If `sales_table` has `customer_id` and `customers_table` has `id`, a relationship is created

### Dependency Detection Algorithm

The system automatically detects primary keys and foreign keys:

**Primary keys are determined by:**
- Columns named `id` of type Integer
- Columns with names `*_id` of type Integer
- Columns with prefix `pk_` or `key_`
- Columns where all values are unique

**Foreign keys are determined by:**
- Columns with ID-like names: `*_id`, `id`, `*Id`, `*ID`
- Data type match (Integer)
- Presence of corresponding primary key in another table

### Type Conversion

| DataCode Type | SQLite Type |
|--------------|------------|
| `Integer` | `INTEGER` |
| `Float` | `REAL` |
| `String` | `TEXT` |
| `Bool` | `INTEGER` (0/1) |
| `Date` | `TEXT` (ISO format) |
| `Currency` | `REAL` |
| `Null` | `NULL` |
| `Mixed` | `TEXT` |

### Limitations

- Only **global variables** are exported (local variables are not exported)
- Only variables of type `Table` are exported (other types are saved only in metadata)
- Foreign keys are created as indexes (full table recreation for FK constraints is not yet implemented)

---

## 🧪 Example Program
```DataCode
# User-defined function for array analysis
global function analyze_array(arr) do
    local size = count(arr)
    local sum_val = sum(arr)
    local avg_val = average(arr)

    print('📊 Array analysis:', arr)
    print('  Size:', size)
    print('  Sum:', sum_val)
    print('  Average:', avg_val)

    return [size, sum_val, avg_val]
endfunction

# Working with arrays and files
global basePath = getcwd()
global dataPath = basePath / 'examples'

# Create data arrays
global numbers = [10, 20, 30, 40, 50]
global mixed_data = [1, 'test', true, 3.14]
global matrix = [[1, 2], [3, 4], [5, 6]]

print('🧮 Numeric data analysis')
global stats = analyze_array(numbers)

print('')
print('📋 Working with files')
global files = ['sample.csv', 'data.txt']

for file in files do
    local fullPath = dataPath / file
    print('📄 Processing:', file)

    # If it's a CSV file, show the table
    if contains(file, '.csv') do
        local table = read_file(fullPath)
        print('📊 Table contents:')
        table_head(table, 3)
    endif
next file

print('')
print('🔢 Working with nested arrays')
for row in matrix do
    local row_sum = sum(row)
    print('Row:', row, 'Sum:', row_sum)
next row

print('✅ Analysis complete!')
```

---

## 📦 Supported Types

| Type | Example | Description |
|-----|--------|----------|
| String | `'abc'`, `'hello.txt'` | Always in single quotes |
| Number | `42`, `3.14` | Integer and floating-point numbers |
| Bool | `true`, `false` | Boolean values |
| Array | `[1, 'hello', true]` | Arrays of any data types |
| Path | `base / 'file.csv'` | Built with `/` |
| Table | `table(data, headers)` | Tabular data |
| Null | — | Returned by `print(...)` |


---

## ⚠️ Errors

Typical error messages:
- Unknown variable: foo
- Invalid / expression
- Unsupported expression
- read_file() expects 1-3 arguments (path, [header_row], [sheet_name])

---

## 📚 Extension

The project is easily extensible:
- Add functions in builtins.rs
- Add types in value.rs
- Add syntax in interpreter.rs

---

## 🧪 Testing

Run:
```bash
cargo test
```
Tests check:
- Variable declarations
- Path concatenation
- Built-in function calls
- For loop execution

---

## 🛠 CLI Usage Example
```bash
cargo run
```

---

## 🎯 Interactive REPL

### Starting
```bash
cargo run
```

### Special REPL Commands
- `help` — show help
- `exit` or `quit` — exit interpreter
- `clear` — clear screen
- `vars` — show all variables
- `reset` — reset interpreter

### Пример сессии
```
🧠 DataCode Interactive Interpreter
>>> global x = 10
✓ x = Number(10.0)
>>> global y = 20
✓ y = Number(20.0)
>>> global result = (x + y) * 2
✓ result = Number(60.0)
>>> print('Result is:', result)
Result is: 60
>>> vars
📊 Current Variables:
  x = Number(10.0)
  y = Number(20.0)
  result = Number(60.0)
>>> exit
Goodbye! �
```

### Multiline Constructs
REPL supports multiline input for loops and arrays:
```
>>> global arr = [1, 2, 3, 4, 5]
✓ arr = Array([Number(1.0), Number(2.0), Number(3.0), Number(4.0), Number(5.0)])
>>> print(arr[0])
1
>>> global nested = [[1, 2], [3, 4]]
✓ nested = Array([Array([Number(1.0), Number(2.0)]), Array([Number(3.0), Number(4.0)])])
>>> print(nested[0][1])
2
>>> for i in [1, 2, 3] do
...     print('Number:', i)
...     global doubled = i * 2
...     print('Doubled:', doubled)
... next i
Number: 1
Doubled: 2
Number: 2
Doubled: 4
Number: 3
Doubled: 6
```

---

## 📚 Examples and Learning

DataCode comes with a professionally organized collection of examples that will help you learn the language from basics to advanced techniques.

### 🎯 Quick Start with Examples

```bash
# Simplest example - start here!
cargo run examples/01-basics/simple.dc

# Extended Hello World
cargo run examples/01-basics/hello.dc

# Working with functions
cargo run examples/02-language-syntax/functions.dc

# Processing data from CSV
cargo run examples/04-data-processing/table_demo.dc
```

### 📁 Example Organization

Examples are organized into thematic sections for systematic learning:

#### 🚀 [examples/01-basics/](examples/01-basics/) - Language Basics
Start learning DataCode with these examples:
- `simple.dc` - variables and arithmetic
- `hello.dc` - extended Hello World

#### 🔧 [examples/02-language-syntax/](examples/02-language-syntax/) - Syntax
Learn the main language constructs:
- `functions.dc` - user-defined functions
- `conditionals.dc` - conditional constructs
- `arrays_example.dc` - working with arrays
- `complex_expressions.dc` - complex expressions

#### 🎯 [examples/03-advanced-features/](examples/03-advanced-features/) - Advanced Techniques
Recursion, error handling, and functional programming:
- `recursion.dc` - recursive algorithms
- `error_handling.dc` - error handling
- `functional_methods_demo.dc` - functional methods

#### 📊 [examples/04-data-processing/](examples/04-data-processing/) - Data Processing
Powerful capabilities for processing tabular data:
- `table_demo.dc` - working with tables and CSV
- `data_filtering_demo.dc` - data filtering
- `enum_demo.dc` - enumeration with indices

#### 🔢 [examples/05-data-types/](examples/05-data-types/) - Type System
Learn about data types and type checking:
- `type_checking_demo.dc` - isinstance() function
- `type_conversion_guide.dc` - type conversion

#### 🛠️ [examples/06-developer-tools/](examples/06-developer-tools/) - Developer Tools
Debugging and performance testing:
- `debug_mode_test.dc` - debug mode
- `interactive_demo.dc` - examples for REPL
- `stress_benchmark.dc` - performance testing

#### 🎪 [examples/07-demonstrations/](examples/07-demonstrations/) - Full Demonstrations
- `showcase.dc` - comprehensive demonstration of all capabilities

### 📖 Detailed Documentation

Each section contains detailed documentation:
- **[examples/README.md](examples/README.md)** - main examples page
- Individual README.md files in each section with step-by-step explanations
- Recommended learning order
- Practical tips and best practices

### 🎓 Recommended Learning Path

1. **Basics** → `01-basics/simple.dc` and `hello.dc`
2. **Syntax** → `02-language-syntax/functions.dc` and `conditionals.dc`
3. **Data Types** → `05-data-types/type_checking_demo.dc`
4. **Data Processing** → `04-data-processing/table_demo.dc`
5. **Advanced Capabilities** → `03-advanced-features/`
6. **Full Demonstration** → `07-demonstrations/showcase.dc`

---

## 📋 Technical Documentation

### Developer Documentation
- **[docs/DEMO_RESULTS.md](docs/DEMO_RESULTS.md)** - Detailed report on table function implementation
  - Architectural decisions and technical details
  - Type system implementation results
  - Performance and limitations
  - Feature development plans

---

## 📅 Implementation Status
### ✅ Fully Implemented
- ✅ Improved error system with detailed messages
- ✅ Powerful expression parser with operator precedence
- ✅ **Array literals** `[1, 2, 3]`, `['a', 'b']`, mixed types
- ✅ **Array indexing** `arr[0]`, `nested[0][1]` with full nesting support
- ✅ Arithmetic operations (+, -, *, /)
- ✅ Comparison operators (==, !=, <, >, <=, >=)
- ✅ Logical operations (and, or, not)
- ✅ Interactive REPL with multiline support and command history
- ✅ Global / local variable support
- ✅ Conditional constructs if/else/endif (with nesting support)
- ✅ User-defined functions with local scope
- ✅ Recursive functions
- ✅ For loops ... in (including array literals)
- ✅ 40+ built-in functions (mathematical, string, file, table)
- ✅ Working with tables and CSV/Excel files
- ✅ Automatic data typing with warnings
- ✅ File system path support
- ✅ .dc file execution
- ✅ **Objects with methods** `{key: value}`
- ✅ **Exception handling try/catch**
- ✅ **SQLite export** (`--build_model`) - automatic table export with dependency detection

### 🔄 Known Limitations
- ⚠️ Nested conditions require careful use

### 📋 Planned for Future
- 📋 Module imports
- 📋 Array destructuring

---

## 🧑‍💻 Автор

Made by Igornet0.
