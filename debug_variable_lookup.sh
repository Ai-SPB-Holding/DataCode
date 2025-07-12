#!/bin/bash

echo "🧠 Debugging Variable Lookup"
echo "============================"

# Создаем временный файл с командами для отладки
cat > /tmp/datacode_var_debug.txt << 'EOF'
global temp1 = 44
print('Global temp1:', temp1)
global function test_lookup() do
    local temp1 = 20
    print('Local temp1:', temp1)
    local temp2 = temp1 / 2
    print('temp2 (should be 10):', temp2)
    return temp2
endfunction
global result = test_lookup()
print('Final result:', result)
print('Global temp1 after function:', temp1)
vars
exit
EOF

echo "📝 Variable lookup debug commands:"
cat /tmp/datacode_var_debug.txt
echo ""
echo "🚀 Running variable lookup debug test..."
echo ""

# Запускаем REPL с тестовыми командами
cargo run < /tmp/datacode_var_debug.txt

# Очищаем временный файл
rm /tmp/datacode_var_debug.txt

echo ""
echo "✅ Variable lookup debug test completed!"
