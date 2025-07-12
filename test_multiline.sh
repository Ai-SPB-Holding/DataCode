#!/bin/bash

echo "🔄 Testing DataCode REPL multiline functionality"
echo "==============================================="

# Создаем временный файл с многострочными командами
cat > /tmp/datacode_multiline_test.txt << 'EOF'
global numbers = [1, 2, 3, 4, 5]
for num in numbers do
    print('Processing number:', num)
    global doubled = num * 2
    print('Doubled:', doubled)
forend
print('Loop completed!')
vars
exit
EOF

echo "📝 Test commands (multiline for loop):"
cat /tmp/datacode_multiline_test.txt
echo ""
echo "🚀 Running multiline test..."
echo ""

# Запускаем REPL с многострочными командами
cargo run < /tmp/datacode_multiline_test.txt

# Очищаем временный файл
rm /tmp/datacode_multiline_test.txt

echo ""
echo "✅ Multiline test completed!"
