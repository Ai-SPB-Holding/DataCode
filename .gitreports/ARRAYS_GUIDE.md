# 📊 DataCode Arrays Guide

Полное руководство по работе с массивами в DataCode.

## 🚀 Создание массивов

### Базовые типы
```datacode
# Числовые массивы
global numbers = [1, 2, 3, 4, 5]
global floats = [1.1, 2.2, 3.3]

# Строковые массивы
global strings = ['hello', 'world', 'datacode']
global names = ['Alice', 'Bob', 'Charlie']

# Булевы массивы
global flags = [true, false, true, false]

# Пустой массив
global empty = []
```

### Смешанные типы
```datacode
# Массив с разными типами данных
global mixed = [1, 'hello', true, 3.14, false]
global data = [42, 'answer', [1, 2, 3]]
```

### Вложенные массивы
```datacode
# Двумерные массивы (матрицы)
global matrix = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
]

# Массивы разной длины
global jagged = [
    [1, 2],
    [3, 4, 5, 6],
    [7]
]

# Глубокая вложенность
global deep = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]]
```

## 🔍 Доступ к элементам

### Базовое индексирование
```datacode
global arr = [10, 20, 30, 40, 50]

print(arr[0])    # 10 (первый элемент)
print(arr[1])    # 20 (второй элемент)
print(arr[4])    # 50 (последний элемент)
```

### Вложенное индексирование
```datacode
global matrix = [[1, 2, 3], [4, 5, 6]]

print(matrix[0])     # [1, 2, 3] (первая строка)
print(matrix[0][0])  # 1 (первый элемент первой строки)
print(matrix[1][2])  # 6 (третий элемент второй строки)
```

## 🔄 Использование в циклах

### Простые циклы
```datacode
# Цикл по литералу массива
for num in [1, 2, 3, 4, 5] do
    print('Number:', num)
forend

# Цикл по переменной-массиву
global colors = ['red', 'green', 'blue']
for color in colors do
    print('Color:', color)
forend
```

### Циклы с вложенными массивами
```datacode
global matrix = [[1, 2], [3, 4], [5, 6]]

for row in matrix do
    print('Row:', row)
    for item in row do
        print('  Item:', item)
    forend
forend
```

## 🛠️ Встроенные функции для массивов

### Информационные функции
```datacode
global arr = [1, 2, 3, 4, 5]

print('Количество элементов:', count(arr))  # 5
print('Сумма:', sum(arr))                   # 15
print('Среднее:', average(arr))             # 3
```

### Модификация массивов
```datacode
global arr = [1, 2, 3]

# Добавление элемента
push(arr, 4)
print('После push:', arr)  # [1, 2, 3, 4]

# Удаление последнего элемента
pop(arr)
print('После pop:', arr)   # [1, 2, 3]
```

### Преобразования
```datacode
global arr = [3, 1, 4, 1, 5]

print('Отсортированный:', sort(arr))      # [1, 1, 3, 4, 5]
print('Обращенный:', reverse(arr))        # [5, 1, 4, 1, 3]
print('Уникальные:', unique(arr))         # [3, 1, 4, 5]
```

### Строковые операции
```datacode
global words = ['hello', 'world', 'datacode']
global joined = join(words, ' ')
print('Объединенные:', joined)  # "hello world datacode"

global text = 'a,b,c,d'
global split_arr = split(text, ',')
print('Разделенные:', split_arr)  # ['a', 'b', 'c', 'd']
```

## 💡 Практические примеры

### Обработка данных
```datacode
# Анализ оценок студентов
global grades = [85, 92, 78, 96, 88, 91, 84]

print('📊 Анализ оценок:')
print('Всего оценок:', count(grades))
print('Средний балл:', average(grades))
print('Максимальная оценка:', max(grades))
print('Минимальная оценка:', min(grades))

# Подсчет отличных оценок (>= 90)
global excellent_count = 0
for grade in grades do
    if grade >= 90 do
        global excellent_count = excellent_count + 1
    endif
forend
print('Отличных оценок:', excellent_count)
```

### Работа с матрицами
```datacode
# Сумма элементов матрицы
global matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
global total_sum = 0

for row in matrix do
    local row_sum = sum(row)
    print('Сумма строки:', row_sum)
    global total_sum = total_sum + row_sum
forend

print('Общая сумма матрицы:', total_sum)
```

### Фильтрация данных
```datacode
# Создание нового массива с четными числами
global numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
global evens = []

for num in numbers do
    if num % 2 == 0 do
        push(evens, num)
    endif
forend

print('Четные числа:', evens)  # [2, 4, 6, 8, 10]
```

## ⚡ Продвинутые техники

### Trailing comma
```datacode
# Поддерживается trailing comma для удобства
global arr = [
    1,
    2,
    3,  # <- trailing comma OK
]
```

### Динамическое создание массивов
```datacode
# Создание массива в цикле
global squares = []
for i in [1, 2, 3, 4, 5] do
    push(squares, i * i)
forend
print('Квадраты:', squares)  # [1, 4, 9, 16, 25]
```

### Комбинирование с функциями
```datacode
global function process_array(arr) do
    local result = []
    for item in arr do
        push(result, item * 2)
    forend
    return result
endfunction

global original = [1, 2, 3, 4]
global doubled = process_array(original)
print('Удвоенные:', doubled)  # [2, 4, 6, 8]
```

## 🎯 Лучшие практики

1. **Используйте описательные имена**: `user_scores` вместо `arr`
2. **Проверяйте границы**: убедитесь, что индекс существует
3. **Используйте встроенные функции**: `sum(arr)` вместо ручного подсчета
4. **Группируйте связанные данные**: `[name, age, email]` для записи пользователя
5. **Документируйте структуру**: комментируйте сложные вложенные массивы

## 🚨 Частые ошибки

- **Index out of bounds**: обращение к несуществующему индексу
- **Смешивание типов**: будьте осторожны с операциями над смешанными массивами
- **Забытые скобки**: `arr[0]` а не `arr0`
- **Неправильная вложенность**: `matrix[0][1]` для двумерного массива

---

Теперь вы знаете все о массивах в DataCode! 🎉
