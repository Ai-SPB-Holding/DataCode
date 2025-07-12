#!/bin/bash

echo "🧠 Testing DataCode User Functions"
echo "=================================="

# Создаем временный файл с командами для тестирования функций
cat > /tmp/datacode_functions_test.txt << 'EOF'
global function add(a, b) do
    return a + b
endfunction
global result = add(5, 3)
print('Result:', result)
vars
exit
EOF

echo "📝 Test commands (simple function):"
cat /tmp/datacode_functions_test.txt
echo ""
echo "🚀 Running function test..."
echo ""

# Запускаем REPL с тестовыми командами
cargo run < /tmp/datacode_functions_test.txt

# Очищаем временный файл
rm /tmp/datacode_functions_test.txt

echo ""
echo "✅ Function test completed!"
