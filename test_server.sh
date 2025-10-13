#!/bin/bash
# Script de teste para validar as melhorias do servidor

echo "🧪 Testando melhorias do servidor Senior System"
echo "================================================"
echo ""

# Verificar se o servidor está rodando
echo "1️⃣ Testando endpoint raiz..."
curl -s http://localhost:3000/ && echo " ✅" || echo " ❌ Servidor não está rodando"
echo ""

# Testar health check
echo "2️⃣ Testando health check..."
curl -s http://localhost:3000/healthz | jq '.' && echo " ✅" || echo " ⚠️  Health check falhou (servidor pode não estar rodando)"
echo ""

# Testar login e obter token
echo "3️⃣ Testando login (JWT)..."
RESPONSE=$(curl -s -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{"usuario":"admin","senha":"1234"}')

echo "$RESPONSE" | jq '.'

if echo "$RESPONSE" | grep -q "token"; then
    echo " ✅ Login retornou token JWT"
    TOKEN=$(echo "$RESPONSE" | jq -r '.token')
    echo "    Token (primeiros 50 chars): ${TOKEN:0:50}..."
else
    echo " ❌ Login não retornou token"
fi
echo ""

echo "================================================"
echo "✅ Testes concluídos!"
echo ""
echo "💡 Dicas:"
echo "  - Inicie o servidor com: cargo run --bin senior-system-server"
echo "  - Configure JWT_SECRET no arquivo .env"
echo "  - Veja logs detalhados com: RUST_LOG=debug cargo run --bin senior-system-server"
