# 📊 Database Model Creation

This section demonstrates how to build a complete database model from CSV files using DataCode. You'll learn to load data from multiple files, merge tables, create relations, and export everything to a SQLite database.

## 📁 Files in This Section

### Main Script
- **[`load_model_data.dc`](load_model_data.dc)** - Complete example that loads all data from the `model_data/` directory, merges monthly and quarterly tables, creates relations, and exports to SQLite

### Step-by-Step Examples

1. **[`01-file-operations.dc`](01-file-operations.dc)** - Working with files and directories
   - List files in directories
   - Check file existence
   - Build file paths
   - Iterate over directory structures

2. **[`02-merge-tables.dc`](02-merge-tables.dc)** - Merging multiple tables
   - Basic table merging with `merge_tables()`
   - Merging tables loaded from files
   - Handling edge cases (empty arrays, single tables)

3. **[`03-create-relations.dc`](03-create-relations.dc)** - Creating relations between tables
   - Simple relations between two tables
   - Multiple relations
   - Relations after merging tables
   - Safe relation creation using helper functions

4. **[`04-load-quarterly-data.dc`](04-load-quarterly-data.dc)** - Loading quarterly aggregated data
   - Navigating nested directory structures
   - Loading quarterly summary files
   - Merging quarterly data tables

5. **[`05-table-joins.dc`](05-table-joins.dc)** - Table JOIN operations
   - INNER JOIN - matching rows from both tables
   - LEFT JOIN - all left rows with matching right rows
   - RIGHT JOIN - all right rows with matching left rows
   - FULL JOIN - all rows from both tables
   - CROSS JOIN - Cartesian product
   - SEMI JOIN - left rows with matches (no right columns)
   - ANTI JOIN - left rows without matches
   - JOIN with multiple keys
   - Practical use cases

## 🚀 Quick Start

### Running Individual Examples

```bash
# Working with files
cargo run examples/en/09-data-model-creation/01-file-operations.dc

# Merging tables
cargo run examples/en/09-data-model-creation/02-merge-tables.dc

# Creating relations
cargo run examples/en/09-data-model-creation/03-create-relations.dc

# Loading quarterly data
cargo run examples/en/09-data-model-creation/04-load-quarterly-data.dc

# Table JOIN operations
cargo run examples/en/09-data-model-creation/05-table-joins.dc
```

### Running Complete Example

```bash
# Load all data and create SQLite database
cargo run examples/en/09-data-model-creation/load_model_data.dc --build_model
```

This will:
1. Load reference data (product catalog, regions, employees)
2. Load monthly data (sales, inventory, refunds, marketing)
3. Load quarterly aggregated data (financial summaries, regional summaries, etc.)
4. Merge all monthly tables into consolidated tables
5. Merge all quarterly tables into consolidated tables
6. Create relations between tables
7. Export everything to SQLite database `load_model_data.db`

## 📚 Concepts Covered

### File Operations
- **`list_files(path)`** - Get list of files and directories by path
- **`read_file(path)`** - Read CSV file into table
- **Path concatenation** - Using `/` operator to build paths
- **`getcwd()`** - Get current working directory

### Table Operations
- **`table_create(data, headers)`** - Create table from data
- **`merge_tables(tables_array)`** - Merge multiple tables with same structure
- **`table_headers(table)`** - Get column headers from table
- **`len(table)`** - Get number of rows in table

### Relations
- **`relate(table1[col1], table2[col2])`** - Create explicit relation between two table columns
- Relations are used when exporting to SQLite to create foreign key constraints
- Both columns must have compatible types (String ↔ String, Number ↔ Number)
- When using `--build_model`, explicit relations created with `relate()` take priority over automatic foreign key detection
- Example: `relate(products["product_id"], sales["product_id"])` creates a foreign key from `sales.product_id` to `products.product_id`

### Table JOIN Operations
- **`inner_join(left, right, left_key, right_key)`** - Returns matching rows from both tables
- **`left_join(left, right, left_key, right_key)`** - All left rows + matching right rows
- **`right_join(left, right, left_key, right_key)`** - All right rows + matching left rows
- **`full_join(left, right, left_key, right_key)`** - All rows from both tables
- **`cross_join(left, right)`** - Cartesian product (all combinations)
- **`semi_join(left, right, left_key, right_key)`** - Left rows with matches (no right columns)
- **`anti_join(left, right, left_key, right_key)`** - Left rows without matches
- **`join(left, right, keys, type)`** - Universal join function with type parameter
- JOIN with multiple keys: `inner_join(t1, t2, [["col1", "col1"], ["col2", "col2"]])`

### Error Handling
- **`try/catch/endtry`** - Error handling
- Check for null values before operations
- Validate table structures before merging

## 📂 Data Structure

The `model_data/` directory has the following structure:

```
model_data/
├── docs/                          # Reference data
│   ├── product_catalog.csv
│   ├── regions.csv
│   └── employees.csv
├── 2023/                          # Year directories
│   ├── 01/                        # Month directories
│   │   ├── sales_2023_01.csv
│   │   ├── inventory_2023_01.csv
│   │   ├── refunds_2023_01.csv
│   │   └── marketing_spend_2023_01.csv
│   ├── 03/
│   │   ├── sales_2023_03.csv
│   │   └── quarter_2023_Q1/       # Quarterly aggregates
│   │       ├── financial_summary_2023_Q1.csv
│   │       ├── regional_summary_2023_Q1.csv
│   │       ├── product_summary_2023_Q1.csv
│   │       └── employee_performance_2023_Q1.csv
│   └── ...
├── 2024/
└── 2025/
```

## 🔗 Relations Between Tables

The example creates the following relations:

- `product_catalog.product_id` ↔ `sales_all.product_id`
- `product_catalog.product_id` ↔ `inventory_all.product_id`
- `product_catalog.product_id` ↔ `refunds_all.product_id`
- `regions.region_code` ↔ `sales_all.region`
- `regions.region_code` ↔ `inventory_all.region`
- `regions.region_code` ↔ `refunds_all.region`
- `regions.region_code` ↔ `marketing_spend_all.region`
- `employees.employee_id` ↔ `sales_all.employee_id`
- `sales_all.transaction_id` ↔ `refunds_all.transaction_id`
- And others...

## 💡 Best Practices

1. **Always check for null** - Before using merged tables, check if they are null
2. **Use try-catch** - Wrap file operations in try-catch blocks
3. **Validate structure** - Ensure tables have compatible structures before merging
4. **Create relations after merging** - Relations should be created after merging all tables
5. **Use helper functions** - Create helper functions for common operations (e.g., safe relation creation)

## 🎯 Learning Path

1. Start with **01-file-operations.dc** to understand file operations
2. Move to **02-merge-tables.dc** to learn table merging
3. Study **03-create-relations.dc** to understand relations
4. Explore **04-load-quarterly-data.dc** for complex directory navigation
5. Learn **05-table-joins.dc** to master table join operations
6. Finally, study **load_model_data.dc** to see everything together

## 📖 Related Examples

- **[01-basics/](../01-basics/)** - Basic operations
- **[03-data-types/](../03-data-types/)** - Understanding DataCode types
- **[04-advanced/](../04-advanced/)** - Advanced features and error handling

---

*This example demonstrates real-world database modeling scenarios using DataCode's built-in functions for file operations, table manipulations, and relation management.*

