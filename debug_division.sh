#!/bin/bash

echo "🧠 Debugging Division with Local Variables"
echo "=========================================="

# Создаем временный файл с командами для отладки
cat > /tmp/datacode_division_debug.txt << 'EOF'
global function test_division() do
    local temp1 = 20
    print('temp1:', temp1)
    local temp2 = temp1 / 2
    print('temp2:', temp2)
    return temp2
endfunction
global result = test_division()
print('Final result:', result)
vars
exit
EOF

echo "📝 Division debug commands:"
cat /tmp/datacode_division_debug.txt
echo ""
echo "🚀 Running division debug test..."
echo ""

# Запускаем REPL с тестовыми командами
cargo run < /tmp/datacode_division_debug.txt

# Очищаем временный файл
rm /tmp/datacode_division_debug.txt

echo ""
echo "✅ Division debug test completed!"
