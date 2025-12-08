#!/usr/bin/env python3
"""
Генератор тестовых данных для моделирования корпоративного хранилища данных (DWH).

Генерирует расширенную структуру данных, максимально приближенную к реальным корпоративным хранилищам:

Структура каталога:
    data/
    ├── docs/
    │   ├── product_catalog.csv
    │   ├── regions.csv
    │   └── employees.csv
    ├── YYYY/
    │   ├── MM/
    │   │   ├── sales_YYYY_MM.csv
    │   │   ├── inventory_YYYY_MM.csv
    │   │   ├── refunds_YYYY_MM.csv
    │   │   ├── marketing_spend_YYYY_MM.csv
    │   │   └── quarter_YYYY_QX/  (в последнем месяце квартала)
    │   │       ├── financial_summary_QX.csv
    │   │       ├── regional_summary_QX.csv
    │   │       ├── product_summary_QX.csv
    │   │       └── employee_performance_QX.csv
    │   └── ...
    └── ...

Особенности:
- Ежемесячные файлы для каждого месяца (01-12)
- Квартальные агрегаты в папках quarter_YYYY_QX (Q1-Q4)
- Квартальные файлы содержат данные за все 3 месяца квартала
- Edge cases для тестирования парсеров (отсутствующие файлы, поврежденные данные, и т.д.)
- Взаимосвязанные сущности для построения Dependency Graph

Dependency Graph:
- refunds → sales (по transaction_id)
- financial_summary → sales + refunds + marketing
- product_summary → sales + inventory + refunds + product_catalog
- regional_summary → sales + refunds
- employee_performance → sales + employees
"""

import csv
import random
from datetime import datetime, timedelta
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Tuple

# Константы
BASE_DIR = Path(__file__).parent
DOCS_DIR = BASE_DIR / "docs"
YEARS = [2023, 2024, 2025]
MONTHS = list(range(1, 13))
QUARTERS = {
    1: (1, 2, 3),    # Q1
    4: (4, 5, 6),    # Q2
    7: (7, 8, 9),    # Q3
    10: (10, 11, 12) # Q4
}

# Параметры генерации
NUM_PRODUCTS = 50
NUM_REGIONS = 10
NUM_EMPLOYEES = 30
NUM_STORES_PER_REGION = 3
NUM_WAREHOUSES_PER_REGION = 2
NUM_TRANSACTIONS_PER_MONTH = 500
NUM_REFUNDS_PER_MONTH = 20
NUM_MARKETING_CAMPAIGNS_PER_MONTH = 5

# Edge cases конфигурация для тестирования парсеров
# Формат: (year, month, file_type)
# file_type может быть: 'sales', 'inventory', 'refunds', 'marketing_spend'
EDGE_CASES = {
    'missing_files': [(2023, 3, 'inventory'), (2024, 6, 'marketing_spend'), (2025, 9, 'refunds')],
    'empty_files': [(2023, 7, 'refunds'), (2024, 11, 'inventory')],
    'different_delimiter': [(2023, 5, 'sales'), (2024, 8, 'inventory')],  # Разделитель ';' вместо ','
    'bom_files': [(2023, 9, 'sales'), (2024, 12, 'marketing_spend')],  # UTF-8 с BOM
    'damaged_lines': [(2023, 2, 'sales'), (2024, 7, 'inventory')],  # Поврежденные строки
    'missing_fields': [(2023, 6, 'sales'), (2024, 10, 'refunds')],  # Отсутствующие обязательные поля
    'duplicate_transactions': [(2023, 4, 'sales'), (2024, 11, 'sales')],  # Дубликаты transaction_id
    'wrong_encoding': [(2023, 8, 'marketing_spend'), (2024, 2, 'sales')],  # CP1251 вместо UTF-8
    'date_format_variants': [(2023, 10, 'sales'), (2024, 5, 'refunds')]  # Формат даты DD/MM/YYYY
}


class DataGenerator:
    def __init__(self):
        self.products = []
        self.regions = []
        self.employees = []
        self.stores = []
        self.warehouses = []
        self.transaction_counter = 0
        self.refund_counter = 0
        self.customer_ids = set()
        
        # Хранилища для агрегации
        self.monthly_data = defaultdict(lambda: {
            'sales': [],
            'refunds': [],
            'inventory': [],
            'marketing': []
        })
    
    def generate_reference_data(self):
        """Генерация справочников"""
        print("Генерация справочников...")
        
        # Создаем папку docs
        DOCS_DIR.mkdir(exist_ok=True)
        
        # 1. Product Catalog
        self.products = self._generate_products()
        self._write_csv(
            DOCS_DIR / "product_catalog.csv",
            ["product_id", "category", "subcategory", "brand", "launch_date", "is_active"],
            self.products
        )
        
        # 2. Regions
        self.regions = self._generate_regions()
        self._write_csv(
            DOCS_DIR / "regions.csv",
            ["region_code", "region_name", "timezone", "currency", "tax_rate"],
            self.regions
        )
        
        # 3. Employees
        self.employees = self._generate_employees()
        self._write_csv(
            DOCS_DIR / "employees.csv",
            ["employee_id", "name", "role", "region", "hire_date", "is_active"],
            self.employees
        )
        
        # Генерируем stores и warehouses
        self.stores = self._generate_stores()
        self.warehouses = self._generate_warehouses()
        
        print(f"  ✓ Сгенерировано {len(self.products)} продуктов")
        print(f"  ✓ Сгенерировано {len(self.regions)} регионов")
        print(f"  ✓ Сгенерировано {len(self.employees)} сотрудников")
    
    def _generate_products(self) -> List[Dict]:
        categories = [
            ("Electronics", ["Phones", "Laptops", "Tablets", "Accessories"]),
            ("Clothing", ["Men", "Women", "Kids", "Sportswear"]),
            ("Food", ["Beverages", "Snacks", "Dairy", "Frozen"]),
            ("Home", ["Furniture", "Decor", "Kitchen", "Bathroom"]),
            ("Books", ["Fiction", "Non-Fiction", "Children", "Technical"])
        ]
        
        brands = ["BrandA", "BrandB", "BrandC", "BrandD", "BrandE"]
        
        products = []
        product_id = 1
        for category, subcategories in categories:
            for subcategory in subcategories:
                for _ in range(NUM_PRODUCTS // len(categories) // len(subcategories) + 1):
                    if product_id > NUM_PRODUCTS:
                        break
                    launch_date = (datetime(2020, 1, 1) + timedelta(days=random.randint(0, 1000))).strftime("%Y-%m-%d")
                    products.append({
                        "product_id": f"PROD_{product_id:03d}",
                        "category": category,
                        "subcategory": subcategory,
                        "brand": random.choice(brands),
                        "launch_date": launch_date,
                        "is_active": random.choice(["true", "false"])
                    })
                    product_id += 1
        
        return products[:NUM_PRODUCTS]
    
    def _generate_regions(self) -> List[Dict]:
        region_names = [
            "North America", "South America", "Europe", "Asia Pacific",
            "Middle East", "Africa", "Central Asia", "Oceania",
            "Eastern Europe", "Scandinavia"
        ]
        
        timezones = ["UTC-5", "UTC-3", "UTC+1", "UTC+8", "UTC+3", "UTC+2", "UTC+6", "UTC+10", "UTC+2", "UTC+1"]
        currencies = ["USD", "BRL", "EUR", "CNY", "AED", "ZAR", "KZT", "AUD", "PLN", "SEK"]
        
        regions = []
        for i, name in enumerate(region_names[:NUM_REGIONS]):
            regions.append({
                "region_code": f"REG_{i+1:02d}",
                "region_name": name,
                "timezone": timezones[i],
                "currency": currencies[i],
                "tax_rate": round(random.uniform(0.05, 0.25), 3)
            })
        
        return regions
    
    def _generate_employees(self) -> List[Dict]:
        roles = ["Sales Associate", "Store Manager", "Cashier", "Sales Lead", "Supervisor"]
        names = [
            "John Smith", "Maria Garcia", "David Lee", "Sarah Johnson", "Michael Brown",
            "Emily Davis", "James Wilson", "Jessica Martinez", "Robert Taylor", "Amanda Anderson",
            "William Thomas", "Jennifer Jackson", "Richard White", "Lisa Harris", "Joseph Martin",
            "Nancy Thompson", "Charles Garcia", "Karen Martinez", "Thomas Robinson", "Betty Clark",
            "Daniel Rodriguez", "Helen Lewis", "Matthew Walker", "Sandra Hall", "Anthony Allen",
            "Donna Young", "Mark King", "Carol Wright", "Paul Lopez", "Michelle Hill"
        ]
        
        employees = []
        region_index = 0
        
        # Распределяем сотрудников более равномерно, гарантируя хотя бы одного в каждом регионе
        for i, name in enumerate(names[:NUM_EMPLOYEES]):
            # Первые N сотрудников гарантированно распределяются по всем регионам
            if i < len(self.regions):
                region = self.regions[region_index % len(self.regions)]["region_code"]
                region_index += 1
            else:
                # Остальные распределяются случайно
                region = random.choice(self.regions)["region_code"]
            
            hire_date = (datetime(2018, 1, 1) + timedelta(days=random.randint(0, 2000))).strftime("%Y-%m-%d")
            employees.append({
                "employee_id": f"EMP_{i+1:03d}",
                "name": name,
                "role": random.choice(roles),
                "region": region,
                "hire_date": hire_date,
                "is_active": random.choice(["true", "false"])
            })
        
        return employees
    
    def _generate_stores(self) -> List[Dict]:
        stores = []
        store_id = 1
        for region in self.regions:
            for _ in range(NUM_STORES_PER_REGION):
                stores.append({
                    "store_id": f"STORE_{store_id:03d}",
                    "region": region["region_code"]
                })
                store_id += 1
        return stores
    
    def _generate_warehouses(self) -> List[Dict]:
        warehouses = []
        warehouse_id = 1
        for region in self.regions:
            for _ in range(NUM_WAREHOUSES_PER_REGION):
                warehouses.append({
                    "warehouse_id": f"WH_{warehouse_id:03d}",
                    "region": region["region_code"]
                })
                warehouse_id += 1
        return warehouses
    
    def generate_monthly_data(self, year: int, month: int):
        """
        Генерация ежемесячных файлов.
        
        Создает 4 типа файлов для каждого месяца:
        - sales_YYYY_MM.csv: данные о продажах
        - inventory_YYYY_MM.csv: данные об инвентаре
        - refunds_YYYY_MM.csv: данные о возвратах
        - marketing_spend_YYYY_MM.csv: данные о маркетинговых расходах
        
        Также сохраняет данные в памяти для последующей квартальной агрегации.
        
        Args:
            year: Год
            month: Месяц (1-12)
        """
        month_dir = BASE_DIR / str(year) / f"{month:02d}"
        month_dir.mkdir(parents=True, exist_ok=True)
        
        # Проверяем edge cases
        edge_cases = self._get_edge_cases_for_month(year, month)
        
        # Генерируем данные
        sales_data = self._generate_sales(year, month, edge_cases)
        inventory_data = self._generate_inventory(year, month, edge_cases)
        refunds_data = self._generate_refunds(year, month, sales_data, edge_cases)
        marketing_data = self._generate_marketing_spend(year, month, edge_cases)
        
        # Сохраняем в память для квартальной агрегации
        key = (year, month)
        self.monthly_data[key]['sales'] = sales_data
        self.monthly_data[key]['refunds'] = refunds_data
        self.monthly_data[key]['inventory'] = inventory_data
        self.monthly_data[key]['marketing'] = marketing_data
        
        # Записываем файлы (если не пропущены в edge cases)
        missing_files = [file_type for case_type, file_type in edge_cases if case_type == 'missing_files']
        empty_files = [file_type for case_type, file_type in edge_cases if case_type == 'empty_files']
        
        if 'sales' not in missing_files:
            self._write_monthly_file(
                month_dir / f"sales_{year}_{month:02d}.csv",
                ["transaction_id", "date", "region", "store_id", "employee_id", "product_id",
                 "units", "unit_price", "discount", "revenue", "payment", "customer_id", "customer_segment"],
                [] if 'sales' in empty_files else sales_data,
                edge_cases,
                'sales'
            )
        
        if 'inventory' not in missing_files:
            self._write_monthly_file(
                month_dir / f"inventory_{year}_{month:02d}.csv",
                ["product_id", "region", "warehouse_id", "start_stock", "end_stock", "stock_losses", "stock_replenished"],
                [] if 'inventory' in empty_files else inventory_data,
                edge_cases,
                'inventory'
            )
        
        if 'refunds' not in missing_files:
            self._write_monthly_file(
                month_dir / f"refunds_{year}_{month:02d}.csv",
                ["refund_id", "transaction_id", "refund_date", "region", "product_id", "units", "refund_amount", "reason"],
                [] if 'refunds' in empty_files else refunds_data,
                edge_cases,
                'refunds'
            )
        
        if 'marketing_spend' not in missing_files:
            self._write_monthly_file(
                month_dir / f"marketing_spend_{year}_{month:02d}.csv",
                ["region", "channel", "spend", "conversions", "campaign_name"],
                [] if 'marketing_spend' in empty_files else marketing_data,
                edge_cases,
                'marketing_spend'
            )
    
    def _get_edge_cases_for_month(self, year: int, month: int) -> List[str]:
        """Получить edge cases для конкретного месяца"""
        cases = []
        
        for case_type, file_list in EDGE_CASES.items():
            for y, m, file_type in file_list:
                if y == year and m == month:
                    cases.append((case_type, file_type))
        
        return cases
    
    def _generate_sales(self, year: int, month: int, edge_cases: List[Tuple]) -> List[Dict]:
        """Генерация данных о продажах"""
        sales = []
        num_days = (datetime(year, month + 1, 1) - datetime(year, month, 1)).days if month < 12 else 31
        
        # Проверяем, нужны ли дубликаты
        has_duplicates = any(case_type == 'duplicate_transactions' and file_type == 'sales' 
                            for case_type, file_type in edge_cases)
        
        transactions_to_generate = NUM_TRANSACTIONS_PER_MONTH
        if has_duplicates:
            transactions_to_generate += 5  # Добавим несколько дубликатов
        
        transaction_ids_used = set()
        
        for _ in range(transactions_to_generate):
            date = datetime(year, month, random.randint(1, num_days))
            region = random.choice(self.regions)
            region_code = region["region_code"]
            
            # Выбираем магазин в регионе (или любой, если нет)
            stores_in_region = [s for s in self.stores if s["region"] == region_code]
            if stores_in_region:
                store = random.choice(stores_in_region)
            elif self.stores:
                store = random.choice(self.stores)
            else:
                # Fallback - создаем временный магазин
                store = {"store_id": f"STORE_TEMP_{region_code}", "region": region_code}
            
            # Выбираем сотрудника в регионе (или любого, если нет)
            employees_in_region = [e for e in self.employees if e["region"] == region_code]
            if employees_in_region:
                employee = random.choice(employees_in_region)
            elif self.employees:
                employee = random.choice(self.employees)
            else:
                # Fallback - создаем временного сотрудника
                employee = {"employee_id": f"EMP_TEMP_{region_code}", "region": region_code}
            
            product = random.choice(self.products)
            
            units = random.randint(1, 5)
            unit_price = round(random.uniform(10.0, 500.0), 2)
            discount = round(random.uniform(0.0, 0.3), 2)
            revenue = round(units * unit_price * (1 - discount), 2)
            payment = random.choice(["card", "cash", "mobile", "credit"])
            
            # Генерируем customer_id
            customer_id = f"CUST_{random.randint(1000, 9999)}"
            self.customer_ids.add(customer_id)
            customer_segment = random.choice(["VIP", "Regular", "New", "Premium"])
            
            # Проверяем дубликаты
            if has_duplicates and len(transaction_ids_used) > 0 and random.random() < 0.05:
                transaction_id = random.choice(list(transaction_ids_used))
            else:
                self.transaction_counter += 1
                transaction_id = f"TXN_{year}{month:02d}_{self.transaction_counter:06d}"
                transaction_ids_used.add(transaction_id)
            
            # Проверяем отсутствующие поля
            has_missing_fields = any(case_type == 'missing_fields' and file_type == 'sales' 
                                   for case_type, file_type in edge_cases)
            
            sale = {
                "transaction_id": transaction_id,
                "date": date.strftime("%Y-%m-%d" if not any(case_type == 'date_format_variants' and file_type == 'sales' 
                                                           for case_type, file_type in edge_cases) else "%d/%m/%Y"),
                "region": region["region_code"],
                "store_id": store["store_id"],
                "employee_id": employee["employee_id"] if not (has_missing_fields and random.random() < 0.1) else "",
                "product_id": product["product_id"],
                "units": units,
                "unit_price": unit_price,
                "discount": discount,
                "revenue": revenue,
                "payment": payment,
                "customer_id": customer_id if not (has_missing_fields and random.random() < 0.15) else "",
                "customer_segment": customer_segment
            }
            
            sales.append(sale)
        
        return sales
    
    def _generate_inventory(self, year: int, month: int, edge_cases: List[Tuple]) -> List[Dict]:
        """Генерация данных об инвентаре"""
        inventory = []
        
        for product in self.products:
            for region in self.regions:
                warehouses_in_region = [w for w in self.warehouses if w["region"] == region["region_code"]]
                for warehouse in warehouses_in_region:
                    start_stock = random.randint(50, 500)
                    stock_losses = random.randint(0, 10)
                    stock_replenished = random.randint(20, 100)
                    end_stock = start_stock - stock_losses + stock_replenished
                    
                    inventory.append({
                        "product_id": product["product_id"],
                        "region": region["region_code"],
                        "warehouse_id": warehouse["warehouse_id"],
                        "start_stock": start_stock,
                        "end_stock": end_stock,
                        "stock_losses": stock_losses,
                        "stock_replenished": stock_replenished
                    })
        
        return inventory
    
    def _generate_refunds(self, year: int, month: int, sales_data: List[Dict], edge_cases: List[Tuple]) -> List[Dict]:
        """Генерация данных о возвратах"""
        refunds = []
        num_days = (datetime(year, month + 1, 1) - datetime(year, month, 1)).days if month < 12 else 31
        
        # Проверяем формат даты для возвратов
        has_date_variant = any(case_type == 'date_format_variants' and file_type == 'refunds' 
                              for case_type, file_type in edge_cases)
        date_format = "%d/%m/%Y" if has_date_variant else "%Y-%m-%d"
        
        if not sales_data:
            return refunds
        
        # Берем подмножество транзакций для возвратов
        refund_transactions = random.sample(sales_data, min(NUM_REFUNDS_PER_MONTH, len(sales_data)))
        
        reasons = ["Defective", "Not as described", "Customer request", "Wrong item", "Damaged in transit"]
        
        for sale in refund_transactions:
            self.refund_counter += 1
            refund_date = datetime(year, month, random.randint(1, num_days))
            
            # Парсим дату продажи (может быть в разных форматах)
            try:
                sale_date = datetime.strptime(sale["date"], "%Y-%m-%d")
            except ValueError:
                try:
                    sale_date = datetime.strptime(sale["date"], "%d/%m/%Y")
                except ValueError:
                    sale_date = datetime(year, month, 1)
            
            # Возврат обычно происходит после продажи
            if refund_date < sale_date:
                refund_date = sale_date + timedelta(days=random.randint(1, 15))
            
            units_refunded = random.randint(1, sale["units"])
            refund_amount = round(units_refunded * sale["unit_price"], 2)
            
            refunds.append({
                "refund_id": f"REF_{year}{month:02d}_{self.refund_counter:04d}",
                "transaction_id": sale["transaction_id"],
                "refund_date": refund_date.strftime(date_format),
                "region": sale["region"],
                "product_id": sale["product_id"],
                "units": units_refunded,
                "refund_amount": refund_amount,
                "reason": random.choice(reasons)
            })
        
        return refunds
    
    def _generate_marketing_spend(self, year: int, month: int, edge_cases: List[Tuple]) -> List[Dict]:
        """Генерация данных о маркетинговых расходах"""
        marketing = []
        channels = ["Social Media", "TV", "Radio", "Online Ads", "Email", "Print"]
        campaign_names = [
            "Summer Sale", "Back to School", "Holiday Special", "Black Friday",
            "New Year Promotion", "Spring Collection", "Clearance Event"
        ]
        
        for _ in range(NUM_MARKETING_CAMPAIGNS_PER_MONTH):
            region = random.choice(self.regions)
            channel = random.choice(channels)
            spend = round(random.uniform(1000.0, 50000.0), 2)
            conversions = random.randint(50, 500)
            
            marketing.append({
                "region": region["region_code"],
                "channel": channel,
                "spend": spend,
                "conversions": conversions,
                "campaign_name": random.choice(campaign_names)
            })
        
        return marketing
    
    def _write_monthly_file(self, filepath: Path, headers: List[str], data: List[Dict], 
                           edge_cases: List[Tuple], file_type: str):
        """Запись ежемесячного файла с учетом edge cases"""
        # Проверяем edge cases
        has_different_delimiter = any(case_type == 'different_delimiter' and file_type == file_type_check
                                     for case_type, file_type_check in edge_cases)
        has_bom = any(case_type == 'bom_files' and file_type == file_type_check
                     for case_type, file_type_check in edge_cases)
        has_damaged_lines = any(case_type == 'damaged_lines' and file_type == file_type_check
                               for case_type, file_type_check in edge_cases)
        has_wrong_encoding = any(case_type == 'wrong_encoding' and file_type == file_type_check
                                for case_type, file_type_check in edge_cases)
        
        delimiter = ';' if has_different_delimiter else ','
        encoding = 'cp1251' if has_wrong_encoding else 'utf-8-sig' if has_bom else 'utf-8'
        
        with open(filepath, 'w', encoding=encoding, newline='') as f:
            writer = csv.DictWriter(f, fieldnames=headers, delimiter=delimiter)
            writer.writeheader()
            
            # Если данных нет, файл будет содержать только заголовки
            if not data:
                return
            
            for row in data:
                # Добавляем поврежденные строки
                if has_damaged_lines and random.random() < 0.05:
                    # Записываем поврежденную строку
                    f.write(f"{delimiter.join([str(v) if v else '' for v in row.values()])}broken_data\n")
                else:
                    writer.writerow(row)
    
    def generate_quarterly_data(self, year: int, quarter: int):
        """
        Генерация квартальных агрегатов.
        
        Создает папку quarter_YYYY_QX в последнем месяце квартала (например, quarter_2023_Q1 в папке 03/).
        Агрегирует данные за все 3 месяца квартала в 4 файла:
        - financial_summary_QX.csv: финансовый свод за квартал
        - regional_summary_QX.csv: региональная статистика
        - product_summary_QX.csv: статистика по продуктам
        - employee_performance_QX.csv: производительность сотрудников
        
        Args:
            year: Год
            quarter: Номер первого месяца квартала (1, 4, 7, 10)
        """
        quarter_months = QUARTERS[quarter]
        quarter_name = f"Q{(quarter-1)//3 + 1}"
        
        # Собираем данные за все 3 месяца квартала
        quarter_sales = []
        quarter_refunds = []
        quarter_inventory = []
        quarter_marketing = []
        
        for month in quarter_months:
            key = (year, month)
            if key in self.monthly_data:
                # Агрегируем данные из всех месяцев квартала
                quarter_sales.extend(self.monthly_data[key]['sales'])
                quarter_refunds.extend(self.monthly_data[key]['refunds'])
                quarter_inventory.extend(self.monthly_data[key]['inventory'])
                quarter_marketing.extend(self.monthly_data[key]['marketing'])
        
        # Создаем папку для квартала в последнем месяце квартала
        # Формат: YYYY/MM/quarter_YYYY_QX (например, 2023/03/quarter_2023_Q1)
        last_month = quarter_months[-1]
        quarter_dir = BASE_DIR / str(year) / f"{last_month:02d}" / f"quarter_{year}_{quarter_name}"
        quarter_dir.mkdir(parents=True, exist_ok=True)
        
        # Генерируем квартальные файлы с агрегированными данными за все 3 месяца
        self._generate_financial_summary(year, quarter_name, quarter_dir, quarter_sales, quarter_refunds, quarter_marketing)
        self._generate_regional_summary(year, quarter_name, quarter_dir, quarter_sales, quarter_refunds)
        self._generate_product_summary(year, quarter_name, quarter_dir, quarter_sales, quarter_refunds, quarter_inventory)
        self._generate_employee_performance(year, quarter_name, quarter_dir, quarter_sales)
    
    def _generate_financial_summary(self, year: int, quarter: str, dir_path: Path,
                                   sales: List[Dict], refunds: List[Dict], marketing: List[Dict]):
        """Генерация финансового свода за квартал"""
        total_revenue = sum(float(s.get("revenue", 0)) for s in sales)
        total_refunds = sum(float(r.get("refund_amount", 0)) for r in refunds)
        marketing_spend = sum(float(m.get("spend", 0)) for m in marketing)
        net_revenue = total_revenue - total_refunds
        profit_estimation = net_revenue - marketing_spend - (total_revenue * 0.4)  # Примерная оценка
        
        # Определяем даты квартала
        quarter_num = int(quarter[1])
        start_month = (quarter_num - 1) * 3 + 1
        start_date = datetime(year, start_month, 1)
        end_month = start_month + 2  # Последний месяц квартала: 3, 6, 9, 12
        
        # Вычисляем конечную дату квартала
        if end_month == 12:
            # Для Q4 (декабрь) - последний день года
            end_date = datetime(year, 12, 31)
        else:
            # Для остальных кварталов - последний день последнего месяца квартала
            end_date = datetime(year, end_month + 1, 1) - timedelta(days=1)
        
        summary = [{
            "quarter": quarter,
            "start_date": start_date.strftime("%Y-%m-%d"),
            "end_date": end_date.strftime("%Y-%m-%d"),
            "total_revenue": round(total_revenue, 2),
            "total_refunds": round(total_refunds, 2),
            "net_revenue": round(net_revenue, 2),
            "marketing_spend": round(marketing_spend, 2),
            "profit_estimation": round(profit_estimation, 2)
        }]
        
        self._write_csv(dir_path / f"financial_summary_{quarter}.csv",
                       ["quarter", "start_date", "end_date", "total_revenue", "total_refunds",
                        "net_revenue", "marketing_spend", "profit_estimation"],
                       summary)
    
    def _generate_regional_summary(self, year: int, quarter: str, dir_path: Path,
                                  sales: List[Dict], refunds: List[Dict]):
        """Генерация регионального свода за квартал"""
        regional_stats = defaultdict(lambda: {
            "total_sales": 0.0,
            "total_units": 0,
            "revenues": [],
            "products": defaultdict(int),
            "customers": set()
        })
        
        for sale in sales:
            region = sale["region"]
            regional_stats[region]["total_sales"] += float(sale.get("revenue", 0))
            regional_stats[region]["total_units"] += int(sale.get("units", 0))
            regional_stats[region]["revenues"].append(float(sale.get("revenue", 0)))
            regional_stats[region]["products"][sale.get("product_id", "")] += int(sale.get("units", 0))
            if sale.get("customer_id"):
                regional_stats[region]["customers"].add(sale["customer_id"])
        
        summary = []
        for region_code in self.regions:
            region_code = region_code["region_code"]
            stats = regional_stats[region_code]
            
            avg_order_value = (sum(stats["revenues"]) / len(stats["revenues"]) 
                             if stats["revenues"] else 0)
            top_product = max(stats["products"].items(), key=lambda x: x[1])[0] if stats["products"] else ""
            new_customers = len(stats["customers"])  # Упрощенная метрика
            churn_rate = round(random.uniform(0.05, 0.15), 3)  # Примерная метрика
            
            summary.append({
                "region": region_code,
                "total_sales": round(stats["total_sales"], 2),
                "total_units": stats["total_units"],
                "avg_order_value": round(avg_order_value, 2),
                "top_product": top_product,
                "new_customers": new_customers,
                "churn_rate": churn_rate
            })
        
        self._write_csv(dir_path / f"regional_summary_{quarter}.csv",
                       ["region", "total_sales", "total_units", "avg_order_value",
                        "top_product", "new_customers", "churn_rate"],
                       summary)
    
    def _generate_product_summary(self, year: int, quarter: str, dir_path: Path,
                                 sales: List[Dict], refunds: List[Dict], inventory: List[Dict]):
        """Генерация свода по продуктам за квартал"""
        product_stats = defaultdict(lambda: {
            "sales_units": 0,
            "revenue": 0.0,
            "returns": 0,
            "stockouts": 0
        })
        
        # Агрегируем продажи
        for sale in sales:
            product_id = sale.get("product_id", "")
            product_stats[product_id]["sales_units"] += int(sale.get("units", 0))
            product_stats[product_id]["revenue"] += float(sale.get("revenue", 0))
        
        # Агрегируем возвраты
        for refund in refunds:
            product_id = refund.get("product_id", "")
            product_stats[product_id]["returns"] += int(refund.get("units", 0))
        
        # Агрегируем stockouts из инвентаря
        for inv in inventory:
            product_id = inv.get("product_id", "")
            if int(inv.get("end_stock", 0)) == 0:
                product_stats[product_id]["stockouts"] += 1
        
        summary = []
        for product in self.products:
            product_id = product["product_id"]
            stats = product_stats[product_id]
            
            performance_score = round(
                (stats["revenue"] / 1000) - (stats["returns"] * 10) - (stats["stockouts"] * 5),
                2
            ) if stats["sales_units"] > 0 else 0.0
            
            summary.append({
                "product_id": product_id,
                "category": product["category"],
                "sales_units": stats["sales_units"],
                "revenue": round(stats["revenue"], 2),
                "returns": stats["returns"],
                "stockouts": stats["stockouts"],
                "performance_score": performance_score
            })
        
        self._write_csv(dir_path / f"product_summary_{quarter}.csv",
                       ["product_id", "category", "sales_units", "revenue", "returns",
                        "stockouts", "performance_score"],
                       summary)
    
    def _generate_employee_performance(self, year: int, quarter: str, dir_path: Path,
                                      sales: List[Dict]):
        """Генерация свода по производительности сотрудников за квартал"""
        employee_stats = defaultdict(lambda: {
            "total_sales": 0.0,
            "transactions": 0,
            "revenues": []
        })
        
        for sale in sales:
            employee_id = sale.get("employee_id", "")
            if employee_id:
                employee_stats[employee_id]["total_sales"] += float(sale.get("revenue", 0))
                employee_stats[employee_id]["transactions"] += 1
                employee_stats[employee_id]["revenues"].append(float(sale.get("revenue", 0)))
        
        summary = []
        for employee in self.employees:
            employee_id = employee["employee_id"]
            stats = employee_stats[employee_id]
            
            avg_revenue = (sum(stats["revenues"]) / len(stats["revenues"]) 
                          if stats["revenues"] else 0)
            absentee_days = random.randint(0, 5)  # Примерная метрика
            bonus = round(stats["total_sales"] * 0.02, 2) if stats["transactions"] > 0 else 0.0
            
            summary.append({
                "employee_id": employee_id,
                "region": employee["region"],
                "total_sales": round(stats["total_sales"], 2),
                "transactions": stats["transactions"],
                "avg_revenue": round(avg_revenue, 2),
                "absentee_days": absentee_days,
                "bonus": bonus
            })
        
        self._write_csv(dir_path / f"employee_performance_{quarter}.csv",
                       ["employee_id", "region", "total_sales", "transactions",
                        "avg_revenue", "absentee_days", "bonus"],
                       summary)
    
    def _write_csv(self, filepath: Path, headers: List[str], data: List[Dict], delimiter: str = ','):
        """Утилита для записи CSV файла"""
        with open(filepath, 'w', encoding='utf-8', newline='') as f:
            if data:
                writer = csv.DictWriter(f, fieldnames=headers, delimiter=delimiter)
                writer.writeheader()
                writer.writerows(data)
            else:
                writer = csv.DictWriter(f, fieldnames=headers, delimiter=delimiter)
                writer.writeheader()
    
    def generate_all(self):
        """Генерация всех данных"""
        print("=" * 60)
        print("ГЕНЕРАЦИЯ ТЕСТОВЫХ ДАННЫХ ДЛЯ DWH")
        print("=" * 60)
        
        # Генерируем справочники
        self.generate_reference_data()
        
        # Генерируем ежемесячные данные
        print("\nГенерация ежемесячных данных...")
        for year in YEARS:
            for month in MONTHS:
                print(f"  {year}-{month:02d}...", end=" ")
                self.generate_monthly_data(year, month)
                print("✓")
        
        # Генерируем квартальные данные
        print("\nГенерация квартальных агрегатов...")
        for year in YEARS:
            for quarter_start in QUARTERS.keys():
                quarter_num = (quarter_start - 1) // 3 + 1
                quarter_name = f"Q{quarter_num}"
                print(f"  {year} {quarter_name}...", end=" ")
                self.generate_quarterly_data(year, quarter_start)
                print("✓")
        
        print("\n" + "=" * 60)
        print("ГЕНЕРАЦИЯ ЗАВЕРШЕНА")
        print("=" * 60)
        print(f"\nСтруктура данных:")
        print(f"  - Справочники: {DOCS_DIR}")
        print(f"  - Ежемесячные файлы: {len(YEARS) * 12} месяцев")
        print(f"    * Каждый месяц содержит: sales, inventory, refunds, marketing_spend")
        print(f"  - Квартальные агрегаты: {len(YEARS) * 4} кварталов")
        print(f"    * Каждый квартал (в последнем месяце) содержит папку quarter_YYYY_QX")
        print(f"    * Квартальные файлы агрегируют данные за все 3 месяца квартала")
        print(f"    * Файлы: financial_summary, regional_summary, product_summary, employee_performance")
        print(f"\nEdge cases применены:")
        for case_type, files in EDGE_CASES.items():
            if files:
                print(f"  - {case_type}: {len(files)} файлов")


def main():
    generator = DataGenerator()
    generator.generate_all()


if __name__ == "__main__":
    main()

