# 📊 Working with Tables in DataCode

DataCode now supports powerful functions for working with tabular data, including automatic typing and convenient viewing.

## 🚀 Main Capabilities

### 1. Automatic Table Creation from CSV/Excel Files
```datacode
# Load CSV file - table is automatically created
global data = read_file(getcwd() / "data.csv")
show_table(data)
```

### 2. Manual Table Creation
```datacode
# From array of arrays
global data = [
    [1, "Alice", 25],
    [2, "Bob", 30],
    [3, "Charlie", 35]
]
global headers = ["id", "name", "age"]
global my_table = table(data, headers)
```

### 3. Flexible Typing with Warnings
DataCode automatically determines column types based on majority of values:
- **Integer/Float** for numeric data
- **String** for text data  
- **Bool** for boolean values
- **Date** for dates in standard formats

When heterogeneous data is detected, warnings are displayed:
```
⚠️ Column 'id' contains heterogeneous data: 20.0% of values do not match main type Integer
```

## 📋 Table Functions

### Viewing Data
```datacode
show_table(my_table)        # Show entire table
table_head(my_table, 5)     # First 5 rows
table_tail(my_table, 3)     # Last 3 rows
table_info(my_table)        # Table and type information
```

### Selection and Sorting
```datacode
# Select specific columns
global selected = table_select(my_table, ["name", "age"])

# Sort by column
global sorted_asc = table_sort(my_table, "age", true)   # Ascending
global sorted_desc = table_sort(my_table, "age", false) # Descending
```

## 🎯 Usage Examples

### Example 1: Employee Data Analysis
```datacode
# Load data
global employees = read_file(getcwd() / "employees.csv")

# View general information
table_info(employees)

# Top 5 by salary
global top_earners = table_sort(employees, "salary", false)
table_head(top_earners, 5)

# Only active employees
global active_only = table_select(employees, ["name", "department", "salary"])
show_table(active_only)
```

### Example 2: Creating a Report
```datacode
# Create summary table
global summary = [
    ["Engineering", 5, 425000],
    ["Marketing", 3, 242500], 
    ["HR", 2, 126000]
]
global summary_table = table(summary, ["department", "count", "total_salary"])

print("📊 Department Summary:")
show_table(summary_table)
```

## 🔧 Technical Details

### Supported File Formats
- **CSV** - automatic delimiter detection
- **Excel (.xlsx)** - reads first sheet
- **Text files** - as usual

### Data Typing
DataCode uses intelligent typing:
1. Analyzes all values in column
2. Determines predominant type (>50%)
3. Issues warnings about heterogeneity
4. Supports automatic type conversion

### Output Formatting
Tables are displayed in beautiful ASCII format:
```
┌────┬─────────────┬─────┬─────────┐
│ id │ name        │ age │ salary  │
├────┼─────────────┼─────┼─────────┤
│ 1  │ Alice       │ 28  │ 75000.5 │
│ 2  │ Bob         │ 35  │ 82000   │
└────┴─────────────┴─────┴─────────┘
```

## 🚀 Running Examples

```bash
# Basic example
cargo run examples/04-data-processing/table_demo.dc

# Data filtering
cargo run examples/04-data-processing/data_filtering_demo.dc

# Data enumeration
cargo run examples/04-data-processing/enum_demo.dc

# Interactive mode
cargo run
DataCode> global data = read_file(getcwd() / "examples" / "04-data-processing" / "sample_data.csv")
DataCode> show_table(data)
```

## 💡 Usage Tips

1. **Check types**: Always use `table_info()` to check typing
2. **Handle warnings**: Heterogeneous data may indicate data issues
3. **Use sorting**: `table_sort()` helps quickly find extreme values
4. **Select needed columns**: `table_select()` simplifies work with large tables
5. **Limit output**: `table_head()` and `table_tail()` for large tables
