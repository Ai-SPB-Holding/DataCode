#!/bin/bash

echo "🧠 Debugging Simple Local Variables"
echo "==================================="

# Создаем временный файл с командами для отладки
cat > /tmp/datacode_simple_debug.txt << 'EOF'
global function test_local() do
    local x = 10
    print('x after assignment:', x)
    local y = x + 5
    print('y after assignment:', y)
    return y
endfunction
global result = test_local()
print('Final result:', result)
vars
exit
EOF

echo "📝 Simple debug commands:"
cat /tmp/datacode_simple_debug.txt
echo ""
echo "🚀 Running simple debug test..."
echo ""

# Запускаем REPL с тестовыми командами
cargo run < /tmp/datacode_simple_debug.txt

# Очищаем временный файл
rm /tmp/datacode_simple_debug.txt

echo ""
echo "✅ Simple debug test completed!"
