# Specification: Table Join Operations (JOIN)

This document describes the specification for table join operations (JOIN) in the DataCode language.

**📚 Usage examples:**
- Table joins: [`examples/en/09-data-model-creation/`](../../examples/en/09-data-model-creation/)

## 1. General Provisions

JOIN operations are designed to combine two tables based on:
- key equality,
- arbitrary logical conditions,
- temporal proximity of values,
- row position.

All JOIN operations return a new table and do not modify the original data.

## 2. Basic Types

- **Table** — tabular data structure
- **Row** — table row
- **Column** — column identifier
- **Expr** — logical expression
- **JoinType** — join type
- **JoinKey** — Column | (Column, Column)
- **JoinKeys** — JoinKey | List<JoinKey>

## 3. Universal JOIN Function

### 3.1 Signature

```datacode
join(
    left: Table,
    right: Table,
    on: JoinKeys | Expr,
    type: JoinType = "inner",
    suffixes: (string, string) = ("_left", "_right"),
    nulls_equal: boolean = false
) -> Table
```

### 3.2 Supported JOIN Types

```datacode
JoinType :=
    "inner"
  | "left"
  | "right"
  | "full"
  | "cross"
  | "semi"
  | "anti"
```

## 4. Specialized JOIN Functions

All specialized functions are syntactic sugar over `join()`.

### 4.1 INNER JOIN

```datacode
inner_join(
    left: Table,
    right: Table,
    on: JoinKeys
) -> Table
```

**Semantics:** Returns rows for which there is a match in both tables.

**Equivalent:** `join(left, right, on, type="inner")`

**Example:**
```datacode
global users = table([[1, "Alice"], [2, "Bob"]], ["id", "name"])
global orders = table([[1, 100], [1, 200]], ["user_id", "amount"])
global result = inner_join(users, orders, "id", "user_id")
```

### 4.2 LEFT JOIN

```datacode
left_join(
    left: Table,
    right: Table,
    on: JoinKeys
) -> Table
```

**Semantics:** All rows from `left` are preserved. Missing values from `right` are filled with NULL.

**Example:**
```datacode
global users = table([[1, "Alice"], [2, "Bob"]], ["id", "name"])
global orders = table([[1, 100]], ["user_id", "amount"])
global result = left_join(users, orders, "id", "user_id")
# Bob will have NULL in order columns
```

### 4.3 RIGHT JOIN

```datacode
right_join(
    left: Table,
    right: Table,
    on: JoinKeys
) -> Table
```

**Semantics:** All rows from `right` are preserved. Missing values from `left` are filled with NULL.

### 4.4 FULL JOIN

```datacode
full_join(
    left: Table,
    right: Table,
    on: JoinKeys
) -> Table
```

**Semantics:** Returns all rows from both tables. Missing values are filled with NULL.

### 4.5 CROSS JOIN

```datacode
cross_join(
    left: Table,
    right: Table
) -> Table
```

**Semantics:** Returns the Cartesian product of tables.

**Example:**
```datacode
global table1 = table([[1], [2]], ["col1"])
global table2 = table([["a"], ["b"]], ["col2"])
global result = cross_join(table1, table2)
# Result: 4 rows (2 * 2)
```

### 4.6 SEMI JOIN

```datacode
semi_join(
    left: Table,
    right: Table,
    on: JoinKeys
) -> Table
```

**Semantics:**
- Only `left` rows are returned
- `right` columns are not included

**Example:**
```datacode
global users = table([[1, "Alice"], [2, "Bob"], [3, "Charlie"]], ["id", "name"])
global orders = table([[1, 100], [3, 300]], ["user_id", "amount"])
global result = semi_join(users, orders, "id", "user_id")
# Result: only Alice and Charlie (without order columns)
```

### 4.7 ANTI JOIN

```datacode
anti_join(
    left: Table,
    right: Table,
    on: JoinKeys
) -> Table
```

**Semantics:** Returns `left` rows that have no matches in `right`.

**Example:**
```datacode
global users = table([[1, "Alice"], [2, "Bob"], [3, "Charlie"]], ["id", "name"])
global orders = table([[1, 100], [3, 300]], ["user_id", "amount"])
global result = anti_join(users, orders, "id", "user_id")
# Result: only Bob (without orders)
```

## 5. JOIN by Arbitrary Condition

### 5.1 NON-EQUI JOIN

```datacode
join_on(
    left: Table,
    right: Table,
    condition: Expr,
    type: JoinType = "inner"
) -> Table
```

**Examples:**

```datacode
# Simplified syntax: array ["left_col", "operator", "right_col"]
global result = join_on(orders, prices, ["date", ">=", "start_date"])

# Or string: "left_col >= right_col"
global result = join_on(orders, prices, "date >= start_date")
```

## 6. JOIN by Multiple Keys

```datacode
global on = [
    ["user_id", "id"],
    ["region", "region"]
]
```

Join is performed on all keys with logical AND.

**Example:**
```datacode
global result = inner_join(table1, table2, [["id", "id"], ["region", "region"]])
```

## 7. Temporal JOIN (ASOF)

```datacode
asof_join(
    left: Table,
    right: Table,
    on: Column,
    by: Column | List<Column>,
    direction: "backward" | "forward" | "nearest" = "backward"
) -> Table
```

**Purpose:**
- Time series
- Financial data
- Events and logs

**Example:**
```datacode
global prices = table([[100, 10.5], [200, 11.0]], ["time", "price"])
global trades = table([[150, 100], [250, 200]], ["time", "amount"])
global result = asof_join(trades, prices, "time", direction="backward")
# For each trade, finds the nearest price <= trade time
```

## 8. Index JOIN

```datacode
zip_join(
    left: Table,
    right: Table
) -> Table
```

**Semantics:** Joins rows by index (positionally).

**Example:**
```datacode
global table1 = table([[1], [2], [3]], ["col1"])
global table2 = table([["a"], ["b"], ["c"]], ["col2"])
global result = zip_join(table1, table2)
# Result: [[1, "a"], [2, "b"], [3, "c"]]
```

## 9. APPLY / LATERAL JOIN

```datacode
apply_join(
    left: Table,
    fn: (Row) -> Table,
    type: "inner" | "left"
) -> Table
```

**Semantics:** For each `left` row, a subtable is computed.

**Note:** Requires additional infrastructure to support functions as values.

## 10. Column Name Collisions

If column names match:
- Suffixation is applied (`suffixes`)
- Or explicit renaming via `as`

**Example:**
```datacode
global result = left_join(users, orders, "id", suffixes=["_user", "_order"])
# Column "id" becomes "id_user" and "id_order" if there is a collision
```

## 11. Recommended Language Syntax

### 11.1 Object Style

```datacode
users.left_join(orders, on="user_id")
```

### 11.2 Functional Style

```datacode
left_join(users, orders, on="user_id")
```

### 11.3 Pipeline Style (DSL)

```datacode
users
|> left_join(orders, on="user_id")
|> anti_join(bans, on="user_id")
```

**Note:** The pipeline operator (`|>`) is not yet implemented.

## 12. Implementation Guarantees

- JOIN is deterministic
- Row order is preserved when possible
- NULL is not equal to NULL if `nulls_equal = false`

## 13. Implementation Note

The implementation may choose the optimal algorithm:
- Hash Join
- Merge Join
- Nested Loop

without changing the language semantics.

## 14. Implementation Algorithms

### Hash Join (for equi-joins)

1. Build a hash table from the right table by keys
2. Iterate through the left table and search for matches in the hash table
3. For each match, create a resulting row

### Nested Loop Join (for non-equi and small tables)

1. For each left row:
   - For each right row:
     - Check condition
     - If condition is true, add to result

### ASOF Join Algorithm

1. Sort both tables by temporal column
2. For each left row:
   - Using binary search, find the nearest right row
   - Consider direction (backward/forward/nearest)
   - If `by` is specified, limit search to the corresponding group

## 15. Edge Case Handling

- Empty tables
- Missing columns in keys
- NULL values in keys
- Duplicate keys
- Incompatible data types in keys
- Very large tables (possible optimization)

---

**See also:**
- [Working with Tables](./table_create_function.md) - creating and basic table operations
- [Data Types](./data_types.md) - more about Table type
- [Data Model Creation Examples](../../examples/en/09-data-model-creation/) - practical examples of table joins

