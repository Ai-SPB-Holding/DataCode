# 📊 Creating Database Models

This section demonstrates how to build a complete database model from CSV files using DataCode. You'll learn how to load data from multiple files, merge tables, create relationships, and export everything to a SQLite database.

## 📁 Files in This Section

### Main Script
- **[`load_model_data.dc`](load_model_data.dc)** - Complete example that loads all data from `model_data/` directory, merges monthly and quarterly tables, creates relationships, and exports to SQLite

### Step-by-Step Examples

1. **[`01-file-operations.dc`](01-file-operations.dc)** - Working with files and directories
   - Listing files in directories
   - Checking file existence
   - Building file paths
   - Iterating through directory structures

2. **[`02-merge-tables.dc`](02-merge-tables.dc)** - Merging multiple tables
   - Basic table merging with `merge_tables()`
   - Merging tables loaded from files
   - Handling edge cases (empty arrays, single tables)

3. **[`03-create-relations.dc`](03-create-relations.dc)** - Creating relationships between tables
   - Simple relations between two tables
   - Multiple relations
   - Relations after merging tables
   - Safe relation creation with helper functions

4. **[`04-load-quarterly-data.dc`](04-load-quarterly-data.dc)** - Loading quarterly aggregated data
   - Navigating nested directory structures
   - Loading quarterly summary files
   - Merging quarterly data tables

## 🚀 Quick Start

### Run Individual Examples

```bash
# File operations
cargo run examples/09-creat-database-model/01-file-operations.dc

# Merging tables
cargo run examples/09-creat-database-model/02-merge-tables.dc

# Creating relations
cargo run examples/09-creat-database-model/03-create-relations.dc

# Loading quarterly data
cargo run examples/09-creat-database-model/04-load-quarterly-data.dc
```

### Run Complete Example

```bash
# Load all data and create SQLite database
cargo run examples/09-creat-database-model/load_model_data.dc --build_model
```

This will:
1. Load reference data (product catalog, regions, employees)
2. Load monthly data (sales, inventory, refunds, marketing)
3. Load quarterly aggregated data (financial summaries, regional summaries, etc.)
4. Merge all monthly tables into consolidated tables
5. Merge all quarterly tables into consolidated tables
6. Create relationships between tables
7. Export everything to `load_model_data.db` SQLite database

## 📚 Concepts Covered

### File Operations
- **`list_files(path)`** - List files and directories in a path
- **`read_file(path)`** - Read CSV file into a table
- **Path concatenation** - Using `/` operator to build paths
- **`getcwd()`** - Get current working directory

### Table Operations
- **`table_create(data, headers)`** - Create a table from data
- **`merge_tables(tables_array)`** - Merge multiple tables with same structure
- **`table_headers(table)`** - Get column headers from a table
- **`len(table)`** - Get number of rows in a table

### Relations
- **`relate(column1, column2)`** - Create a relationship between two columns
- Relations are used when exporting to SQLite to create foreign key constraints
- Both columns must have compatible types (String ↔ String, Number ↔ Number)

### Error Handling
- **`try/catch/endtry`** - Handle errors gracefully
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

## 🔗 Table Relationships

The example creates the following relationships:

- `product_catalog.product_id` ↔ `sales_all.product_id`
- `product_catalog.product_id` ↔ `inventory_all.product_id`
- `product_catalog.product_id` ↔ `refunds_all.product_id`
- `regions.region_code` ↔ `sales_all.region`
- `regions.region_code` ↔ `inventory_all.region`
- `regions.region_code` ↔ `refunds_all.region`
- `regions.region_code` ↔ `marketing_spend_all.region`
- `employees.employee_id` ↔ `sales_all.employee_id`
- `sales_all.transaction_id` ↔ `refunds_all.transaction_id`
- And more...

## 💡 Best Practices

1. **Always check for null** - Before using merged tables, check if they're null
2. **Use try-catch** - Wrap file operations in try-catch blocks
3. **Validate structure** - Ensure tables have compatible structures before merging
4. **Create relations after merging** - Relations should be created after all tables are merged
5. **Use helper functions** - Create helper functions for common operations (like safe relation creation)

## 🎯 Learning Path

1. Start with **01-file-operations.dc** to understand file handling
2. Move to **02-merge-tables.dc** to learn table merging
3. Study **03-create-relations.dc** to understand relationships
4. Explore **04-load-quarterly-data.dc** for complex directory navigation
5. Finally, examine **load_model_data.dc** to see everything combined

## 📖 Related Examples

- **[04-data-processing/](../04-data-processing/)** - Basic data processing operations
- **[05-data-types/](../05-data-types/)** - Understanding DataCode types
- **[03-advanced-features/](../03-advanced-features/)** - Advanced features and error handling

---

*This example demonstrates real-world database modeling scenarios using DataCode's built-in functions for file operations, table manipulation, and relationship management.*


