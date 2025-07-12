#!/bin/bash

echo "🧠 Debugging Complex Function"
echo "============================="

# Создаем временный файл с командами для отладки
cat > /tmp/datacode_debug_test.txt << 'EOF'
global function complex_calc(a, b, c) do
    local temp1 = (a + b) * c
    print('temp1:', temp1)
    local temp2 = temp1 / 2
    print('temp2:', temp2)
    local result = temp2 - a
    print('final result:', result)
    return result
endfunction
global result = complex_calc(2, 3, 4)
print('Final result:', result)
vars
exit
EOF

echo "📝 Debug commands:"
cat /tmp/datacode_debug_test.txt
echo ""
echo "🚀 Running debug test..."
echo ""

# Запускаем REPL с тестовыми командами
cargo run < /tmp/datacode_debug_test.txt

# Очищаем временный файл
rm /tmp/datacode_debug_test.txt

echo ""
echo "✅ Debug test completed!"
