# 📊 Data Processing in DataCode

This section demonstrates powerful DataCode capabilities for working with tabular data, CSV files, filtering, and data enumeration.

## 📋 Contents

### 1. `table_demo.dc` - Table Work Demonstration
**Description**: Comprehensive example of working with tables, including CSV loading, manipulations, and table creation.

**What you'll learn**:
- Loading data from CSV files
- Table information
- Row and column selection
- Data sorting
- Manual table creation

**Data files used**: `sample_data.csv` - employee data

**Expected output**:
```
DataCode - Working with Tables
===============================
Loading data from CSV file...
Data loaded successfully!
Table information:
[table information]
First 5 rows:
[first 5 rows]
...
```

### 2. `data_filtering_demo.dc` - Data Filtering
**Description**: Demonstrates various methods of filtering tabular data.

**What you'll learn**:
- Filtering by conditions
- Complex filters
- Combining conditions
- Working with filtering results

### 3. `filter_demo_basic.dc` - Basic Filtering
**Description**: Simple data filtering examples for beginners.

**What you'll learn**:
- Filtering basics
- Simple conditions
- Basic comparison operators

### 4. `filter_demo_simple.dc` - Simplified Filtering
**Description**: Even simpler filtering examples for understanding concepts.

### 5. `enum_demo.dc` - Data Enumeration
**Description**: Demonstrates enum() function for iteration with indices.

**What you'll learn**:
- `enum()` function for indexed iteration
- Destructuring in loops
- Working with indices and values

### 6. `enum_table_example.dc` - Tabular Data Enumeration
**Description**: Application of enum() for working with tabular data.

**What you'll learn**:
- Enumerating table columns
- Indexed data access
- Combining enum() with table functions

### 7. `advanced_table_operations.dc` - Advanced Table Operations
**Description**: Demonstrates advanced table operations including table_sort(), merge_tables(), and relate().

**What you'll learn**:
- Sorting tables with `table_sort()`
- Merging multiple tables with `merge_tables()`
- Creating relationships between tables with `relate()`
- Complex data processing workflows

### 8. `advanced_filtering.dc` - Advanced Filtering
**Description**: Comprehensive demonstration of advanced filtering functions including table_query(), table_between(), table_in(), table_is_null(), and table_not_null().

**What you'll learn**:
- Complex queries with `table_query()`
- Range filtering with `table_between()`
- List-based filtering with `table_in()`
- NULL value filtering with `table_is_null()` and `table_not_null()`
- Combining multiple filters

## 📁 Data Files

### `sample_data.csv` - Main Dataset
```csv
id,name,age,salary,department,active
1,Alice Johnson,28,75000.50,Engineering,true
2,Bob Smith,null,82000,Marketing,true
3,Charlie Brown,42,null,Engineering,false
...
```

### `clean_data.csv` - Cleaned Data
Dataset without missing values for demonstration.

### `simple.csv` - Simple Data
Minimal dataset for basic examples.

## 🎯 How to Run Examples

```bash
# Main table demonstration
cargo run examples/04-data-processing/table_demo.dc

# Data filtering
cargo run examples/04-data-processing/data_filtering_demo.dc

# Basic filtering
cargo run examples/04-data-processing/filter_demo_basic.dc

# Data enumeration
cargo run examples/04-data-processing/enum_demo.dc

# Table enumeration
cargo run examples/04-data-processing/enum_table_example.dc

# Advanced table operations
cargo run examples/04-data-processing/advanced_table_operations.dc

# Advanced filtering
cargo run examples/04-data-processing/advanced_filtering.dc
```

## 📖 Additional Documentation

**[TABLE_EXAMPLES.md](TABLE_EXAMPLES.md)** - Detailed technical guide for working with tables, including:
- Detailed examples of using all table functions
- Technical features of data typing
- Table formatting and output
- Practical tips for working with large data

## 📚 Key Functions

### Table Functions
```datacode
# Loading data
global data = read_file(path)                              # Basic reading
global data = read_file(path, "Sheet2")                    # Read specific XLSX sheet
global data = read_file(path, 2)                           # Header in row 2 (0-based)
global data = read_file(path, 1, "DataSheet")              # Sheet "DataSheet", header in row 1

# Table information
table_info(data)

# Viewing data
table_head(data)        # First 5 rows
table_tail(data, 3)     # Last 3 rows
show_table(data)        # Entire table

# Column selection
global selected = table_select(data, ["name", "age"])

# Sorting
global sorted = table_sort(data, "salary", false)  # descending
```

### Table Creation
```datacode
global manual_data = [
    [1, "Project A", "Completed"],
    [2, "Project B", "In Progress"]
]
global headers = ["id", "name", "status"]
global table = table(manual_data, headers)
```

### Enumeration
```datacode
# Simple enumeration
for i, data in enum(table['CustomerNo']) do
    print('Index:', i, 'Value:', data)
next i
```

## ⚠️ Important Features

1. **File paths**: Use `getcwd() / "path" / "file.csv"`
2. **Typing**: CSV files are automatically typed
3. **Null values**: Handled as special values
4. **Performance**: For large tables use `table_head()`

## 🔍 Supported Formats

- **CSV files** (.csv) - main format
  - Optional parameter `header_row` to specify header row (default 0)
- **Excel files** (.xlsx) - supported via `read_file()`
  - Optional parameter `sheet_name` to select specific sheet
  - Optional parameter `header_row` to specify header row (default 0)
- **Text files** (.txt) - for simple data

### Examples of using `read_file`:

```datacode
# Basic reading (first row = header, first sheet for XLSX)
data = read_file(path("data.csv"))

# Read XLSX with sheet selection
data = read_file(path("report.xlsx"), "Sales")

# Read CSV with header in row 2 (0-based indexing)
data = read_file(path("data.csv"), 2)

# Read XLSX with sheet and header row selection
data = read_file(path("report.xlsx"), 1, "DataSheet")  # Sheet "DataSheet", header in row 1
```

## 💡 Practical Tips

1. **Start with `table_demo.dc`** - comprehensive overview of capabilities
2. **Study the data** - use `table_info()` before processing
3. **Test filters** - check results on small data
4. **Use enum()** - for working with indices
5. **Check paths** - ensure files are accessible

## 🔗 Navigation

### Previous Sections
- **[01-basics](../01-basics/)** - basic language concepts
- **[02-language-syntax](../02-language-syntax/)** - arrays and loops
- **[05-data-types](../05-data-types/)** - working with types and isinstance()

### Related Sections
- **[03-advanced-features](../03-advanced-features/)** - applying advanced techniques
- **[06-developer-tools](../06-developer-tools/)** - debugging and testing

### Additional Resources
- **[TABLE_EXAMPLES.md](TABLE_EXAMPLES.md)** - 📊 Detailed technical guide for tables
- **[../INDEX.md](../INDEX.md)** - 📋 Quick index of all examples
- **[../README.md](../README.md)** - 📚 Main examples page

## 📈 Recommended Learning Order

1. **`table_demo.dc`** - basics of working with tables
2. **`filter_demo_simple.dc`** - simple filtering
3. **`filter_demo_basic.dc`** - basic filtering
4. **`data_filtering_demo.dc`** - advanced filtering
5. **`enum_demo.dc`** - data enumeration
6. **`enum_table_example.dc`** - table enumeration
7. **`advanced_table_operations.dc`** - sorting, merging, and relationships
8. **`advanced_filtering.dc`** - complex filtering operations

---

**DataCode makes data processing simple and efficient!** 📊✨
