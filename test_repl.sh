#!/bin/bash

echo "🧠 Testing DataCode REPL functionality"
echo "======================================"

# Создаем временный файл с командами для тестирования
cat > /tmp/datacode_test.txt << 'EOF'
global x = 10
global y = 20
global sum = x + y
global product = x * y
global comparison = x > 5
global logic = comparison and (y < 30)
global greeting = 'Hello, ' + 'DataCode!'
print('Sum:', sum)
print('Product:', product)
print('Comparison:', comparison)
print('Logic:', logic)
print(greeting)
vars
exit
EOF

echo "📝 Test commands:"
cat /tmp/datacode_test.txt
echo ""
echo "🚀 Running test..."
echo ""

# Запускаем REPL с тестовыми командами
cargo run < /tmp/datacode_test.txt

# Очищаем временный файл
rm /tmp/datacode_test.txt

echo ""
echo "✅ Test completed!"
