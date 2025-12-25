# Table Creation Functions in DataCode

## Overview

DataCode provides two equivalent functions for creating tables from array data:
- `table(data, headers?)` - Original table creation function
- `table_create(data, headers?)` - Alternative name for better clarity

Both functions create table structures from two-dimensional data arrays with optional column headers.

**📚 Usage examples:**
- Data model creation: [`examples/en/09-data-model-creation/`](../../examples/en/09-data-model-creation/)
- File operations: [`examples/en/01-basics/`](../../examples/en/01-basics/)

## Syntax

```datacode
table(data)
table(data, headers)

table_create(data)
table_create(data, headers)
```

### Parameters

- **data** (Array): Two-dimensional array, where each subarray represents a table row
- **headers** (Array, optional): Array of strings representing column names

### Return Value

- **Table**: DataCode table structure with rows and columns

## Examples

### Basic Table Creation

```datacode
# Create a simple numeric table
global data = [[1, 25], [2, 30], [3, 35]]
global my_table = table_create(data)
show_table(my_table)
```

Output:
```
┌──────────┬──────────┐
│ Column_0 │ Column_1 │
├──────────┼──────────┤
│ 1        │ 25       │
│ 2        │ 30       │
│ 3        │ 35       │
└──────────┴──────────┘
```

### Table with Custom Headers

```datacode
# Create a table with custom column names
global employee_data = [
    [1, "Alice", 28, 75000],
    [2, "Bob", 35, 82000],
    [3, "Charlie", 42, 68000]
]
global headers = ["id", "name", "age", "salary"]
global employees = table_create(employee_data, headers)
show_table(employees)
```

Output:
```
┌────┬─────────┬─────┬────────┐
│ id │ name    │ age │ salary │
├────┼─────────┼─────┼────────┤
│ 1  │ Alice   │ 28  │ 75000  │
│ 2  │ Bob     │ 35  │ 82000  │
│ 3  │ Charlie │ 42  │ 68000  │
└────┴─────────┴─────┴────────┘
```

### Mixed Data Types

```datacode
# Table with various data types
global mixed_data = [
    [1, "Active", true, "$50000"],
    [2, "Inactive", false, "$60000"],
    [3, "Pending", true, "$55000"]
]
global headers = ["id", "status", "enabled", "budget"]
global status_table = table_create(mixed_data, headers)
show_table(status_table)
```

### Department Summary Example

```datacode
# Create a summary table by departments
global summary = [
    ["Engineering", 5, 425000],
    ["Marketing", 3, 242500], 
    ["HR", 2, 126000]
]
global summary_headers = ["department", "count", "total_salary"]
global summary_table = table_create(summary, summary_headers)

print("Department Summary:")
show_table(summary_table)
```

Output:
```
Department Summary:
┌─────────────┬───────┬──────────────┐
│ department  │ count │ total_salary │
├─────────────┼───────┼──────────────┤
│ Engineering │ 5     │ 425000       │
│ Marketing   │ 3     │ 242500       │
│ HR          │ 2     │ 126000       │
└─────────────┴───────┴──────────────┘
```

## Working with Created Tables

After creating a table, you can use various functions for working with tables:

```datacode
# Create a table
global data = [[1, "Alice", 28], [2, "Bob", 35], [3, "Charlie", 42]]
global headers = ["id", "name", "age"]
global my_table = table_create(data, headers)

# Show table information
table_info(my_table)

# Show first 2 rows
global head_table = table_head(my_table, 2)
show_table(head_table)

# Select specific columns
global names_only = table_select(my_table, ["name", "age"])
show_table(names_only)

# Filter data
global adults = table_where(my_table, "age", ">", 30)
show_table(adults)

# Sort by age
global sorted_table = table_sort(my_table, "age")
show_table(sorted_table)
```

**📚 Examples:** [`examples/en/09-data-model-creation/`](../../examples/en/09-data-model-creation/)

## Error Handling

The `table_create` function will return errors in the following cases:

1. **No arguments provided**:
   ```datacode
   global my_table = table_create()  # Error: Invalid number of arguments
   ```

2. **Data is not an array**:
   ```datacode
   global my_table = table_create("not an array")  # Error: Type error
   ```

3. **Inconsistent row lengths**:
   ```datacode
   global bad_data = [[1, 2], [3, 4, 5]]  # Warning: Row length mismatch
   global my_table = table_create(bad_data)
   ```

## Recommendations

1. **Use descriptive headers**: Always provide meaningful column names
   ```datacode
   # Good
   global headers = ["employee_id", "full_name", "department", "salary"]
   
   # Avoid
   global headers = ["col1", "col2", "col3", "col4"]
   ```

2. **Consistent data types**: Keep data types consistent within columns
   ```datacode
   # Good - consistent numeric data
   global ages = [[25], [30], [35]]
   
   # Avoid - mixed types in one column
   global mixed = [[25], ["thirty"], [35]]
   ```

3. **Handle missing data**: Use null values for missing data
   ```datacode
   global data_with_nulls = [
       [1, "Alice", 28],
       [2, "Bob", null],
       [3, "Charlie", 42]
   ]
   ```

## Function Equivalence

Both functions `table` and `table_create` are functionally identical:

```datacode
global data = [[1, 2], [3, 4]]

# These calls are equivalent
global table1 = table(data)
global table2 = table_create(data)

# Both create the same table structure
```

Use whichever name feels more natural in your code. `table_create` may be more self-documenting for new users.

## Related Functions

- `show_table(table)` - Display table in formatted form
- `table_info(table)` - Show table metadata and statistics
- `table_head(table, n)` - Get first n rows
- `table_tail(table, n)` - Get last n rows
- `table_select(table, columns)` - Select specific columns
- `table_where(table, column, operator, value)` - Filter rows
- `table_sort(table, column)` - Sort by column
- `table_distinct(table, column)` - Get unique values
- `table_sample(table, n)` - Random row sample

**📚 More details:** See [Built-in Functions](./builtin_functions.md#table-functions)

---

**See also:**
- [Data Types](./data_types.md) - more about Table type
- [Built-in Functions](./builtin_functions.md) - complete list of table functions
- [Data Model Creation Examples](../../examples/en/09-data-model-creation/) - practical examples

