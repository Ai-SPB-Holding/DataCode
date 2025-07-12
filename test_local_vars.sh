#!/bin/bash

echo "🧠 Testing DataCode Local Variables in Functions"
echo "==============================================="

# Создаем временный файл с командами для тестирования локальных переменных
cat > /tmp/datacode_local_test.txt << 'EOF'
global function test_local() do
    local x = 42
    return x
endfunction
global result = test_local()
print('Result:', result)
vars
exit
EOF

echo "📝 Test commands (local variables):"
cat /tmp/datacode_local_test.txt
echo ""
echo "🚀 Running local variables test..."
echo ""

# Запускаем REPL с тестовыми командами
cargo run < /tmp/datacode_local_test.txt

# Очищаем временный файл
rm /tmp/datacode_local_test.txt

echo ""
echo "✅ Local variables test completed!"
