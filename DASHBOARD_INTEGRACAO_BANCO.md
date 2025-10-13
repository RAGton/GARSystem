# Dashboard - Integração com Banco de Dados

## 📋 Resumo das Alterações

O painel principal (Dashboard) foi **completamente refatorado** para buscar dados **reais do banco de dados** via API REST, substituindo os dados simulados/mockados anteriormente.

## 🎯 Objetivo

Implementar consultas automáticas ao banco de dados através da API para exibir:
- ✅ KPIs calculados a partir de dados reais (faturamento, ticket médio, total de OS)
- ✅ Gráficos de faturamento baseados nas Ordens de Serviço
- ✅ Distribuição de OS ao longo do tempo
- ✅ Status das OS em tempo real
- ✅ Últimas 5 ordens de serviço cadastradas

## 🔄 Arquitetura da Solução

### Fluxo de Dados

```
┌─────────────┐      HTTP GET      ┌──────────┐      SQL Query     ┌──────────┐
│  Dashboard  │ ─────────────────> │  Server  │ ─────────────────> │  MySQL   │
│   (GUI)     │ <───────────────── │  (API)   │ <───────────────── │   (DB)   │
└─────────────┘    JSON Response   └──────────┘   Result<Vec<T>>   └──────────┘
```

### Componentes Modificados

**Arquivo:** `src/telas/painel_principal.rs`

#### 1. Nova Estrutura de Estado

```rust
pub struct TelaDashboard {
    papel_usuario: PapelUsuario,
    endereco_servidor: Arc<Mutex<String>>,          // ← NOVO
    estado_carregamento: Arc<Mutex<EstadoCarregamento>>, // ← NOVO
    ordens_servico: Arc<Mutex<Vec<OrdemServico>>>,  // ← NOVO (dados reais)
    vendas_por_dia: Vec<[f64; 2]>,                  // processado
    os_por_dia: Vec<[f64; 2]>,                      // processado
    status_os: Vec<(String, u32)>,                  // processado
    dados_carregados: bool,                         // ← NOVO (flag de controle)
}
```

**Mudanças:**
- ❌ Removido: `vendas_recentes` (dados mockados)
- ✅ Adicionado: `endereco_servidor` (URL da API)
- ✅ Adicionado: `estado_carregamento` (controle de estado assíncrono)
- ✅ Adicionado: `ordens_servico` (dados reais do banco)
- ✅ Adicionado: `dados_carregados` (evita requisições duplicadas)

#### 2. Estado de Carregamento

```rust
enum EstadoCarregamento {
    Inicial,           // Ainda não começou
    Carregando,        // Requisição em andamento
    Sucesso,           // Dados carregados com sucesso
    Erro(String),      // Erro com mensagem detalhada
}
```

Este enum permite que a UI responda adequadamente a cada fase do carregamento.

#### 3. Funções de Busca Assíncrona

##### `carregar_dados_servidor()`

```rust
fn carregar_dados_servidor(&mut self, ctx: &egui::Context) {
    if self.dados_carregados {
        return; // Evita requisições duplicadas
    }
    
    self.dados_carregados = true;
    *self.estado_carregamento.lock().unwrap() = EstadoCarregamento::Carregando;
    
    let servidor = self.endereco_servidor.lock().unwrap().clone();
    let estado_clone = self.estado_carregamento.clone();
    let ordens_clone = self.ordens_servico.clone();
    let ctx_clone = ctx.clone();
    
    // Spawn em thread separada para não travar a UI
    crate::executor::spawn(move || {
        let client = crate::http_client::get_client();
        
        match client.get(format!("{}/ordens", servidor)).send() {
            Ok(response) if response.status().is_success() => {
                match response.json::<Vec<OrdemServico>>() {
                    Ok(ordens) => {
                        *ordens_clone.lock().unwrap() = ordens;
                        *estado_clone.lock().unwrap() = EstadoCarregamento::Sucesso;
                    }
                    Err(e) => {
                        *estado_clone.lock().unwrap() = 
                            EstadoCarregamento::Erro(format!("Erro ao parsear: {}", e));
                    }
                }
            }
            Ok(response) => {
                *estado_clone.lock().unwrap() = 
                    EstadoCarregamento::Erro(format!("HTTP {}", response.status()));
            }
            Err(e) => {
                *estado_clone.lock().unwrap() = 
                    EstadoCarregamento::Erro(format!("Conexão: {}", e));
            }
        }
        
        ctx_clone.request_repaint(); // Força redesenho da UI
    });
}
```

**Características:**
- ✅ **Não-bloqueante:** Usa `crate::executor::spawn()` para executar em thread separada
- ✅ **Seguro:** Usa `Arc<Mutex<>>` para compartilhamento thread-safe
- ✅ **Resiliente:** Trata 3 tipos de erro (conexão, HTTP, parsing)
- ✅ **Eficiente:** Flag `dados_carregados` evita requisições duplicadas
- ✅ **Responsivo:** `request_repaint()` atualiza a UI após completar

##### `processar_dados()`

Transforma os dados brutos (`Vec<OrdemServico>`) em estruturas prontas para visualização:

```rust
fn processar_dados(&mut self) {
    let ordens = self.ordens_servico.lock().unwrap();
    
    // 1. Conta status das OS
    let mut abertas = 0u32;
    let mut em_andamento = 0u32;
    let mut aguardando_pecas = 0u32;
    let mut finalizadas = 0u32;
    
    for os in ordens.iter() {
        match os.status {
            StatusOS::Aberta | StatusOS::Orcamento => abertas += 1,
            StatusOS::EmAndamento | StatusOS::Aprovada => em_andamento += 1,
            StatusOS::AguardandoPeca => aguardando_pecas += 1,
            StatusOS::Finalizada => finalizadas += 1,
            _ => {}
        }
    }
    
    self.status_os = vec![
        ("Abertas".to_string(), abertas),
        ("Em Andamento".to_string(), em_andamento),
        ("Aguardando Peças".to_string(), aguardando_pecas),
        ("Finalizadas".to_string(), finalizadas),
    ];
    
    // 2. Gera dados para gráfico de faturamento
    // (baseado no faturamento médio das OS)
    
    // 3. Gera dados para gráfico de OS por dia
    // (distribuição simulada baseada no total)
}
```

**Nota:** Os gráficos de "últimos 30 dias" ainda usam uma **distribuição simulada** baseada nos valores reais. Para ter dados precisos por dia, seria necessário criar um endpoint no servidor que faça agregação SQL por data.

##### `calcular_kpis()`

Calcula indicadores-chave de performance a partir dos dados reais:

```rust
fn calcular_kpis(&self) -> (f64, f64, f64) {
    let ordens = self.ordens_servico.lock().unwrap();
    
    let mut total_faturamento = 0.0;
    
    for os in ordens.iter() {
        let valor_os = os.total_pecas + os.total_servicos;
        total_faturamento += valor_os;
    }
    
    let ticket_medio = if ordens.len() > 0 {
        total_faturamento / ordens.len() as f64
    } else {
        0.0
    };
    
    let meta_mes = 150_000.0; // Configurável
    
    (total_faturamento, meta_mes, ticket_medio)
}
```

**Retorna:**
- `total_faturamento`: Soma de `(total_pecas + total_servicos)` de todas as OS
- `meta_mes`: Meta configurável (atualmente hardcoded)
- `ticket_medio`: Valor médio por ordem de serviço

## 🎨 Interface do Usuário

### Estados Visuais

#### 1. **Carregando**
```
┌─────────────────────────────────────────┐
│                                         │
│              ⏳ (spinner)               │
│    Carregando dados do servidor...     │
│                                         │
└─────────────────────────────────────────┘
```

#### 2. **Erro**
```
┌─────────────────────────────────────────┐
│                                         │
│   ❌ Erro ao carregar dados            │
│   Erro de conexão: Connection refused   │
│                                         │
│      [🔄 Tentar Novamente]             │
└─────────────────────────────────────────┘
```

#### 3. **Sucesso** (Dados Carregados)
```
┌─────────────────────────────────────────────────────────┐
│  Dashboard              [🔄 Atualizar]  Perfil: Admin   │
├─────────────────────────────────────────────────────────┤
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌─────────┐│
│  │Faturamento│ │Meta do Mês│ │Ticket     │ │Total    ││
│  │Total      │ │           │ │Médio      │ │de OS    ││
│  │R$ 45.230  │ │R$ 150.000 │ │R$ 1.508   │ │30       ││
│  └───────────┘ └───────────┘ └───────────┘ └─────────┘│
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────────────┐ │ ┌─────────────────────┐│
│  │ Faturamento Estimado    │ │ │ Status das OS       ││
│  │ (últimos 30 dias)       │ │ │                     ││
│  │                         │ │ │ Abertas      █████  ││
│  │     [GRÁFICO DE LINHA]  │ │ │ Em Andamento ███    ││
│  │                         │ │ │ Aguard. Peças █     ││
│  └─────────────────────────┘ │ │ Finalizadas  ████   ││
│                               │ └─────────────────────┘│
│  ┌─────────────────────────┐ │ ┌─────────────────────┐│
│  │ OS Abertas por dia      │ │ │ Últimas OS          ││
│  │                         │ │ │                     ││
│  │     [GRÁFICO DE LINHA]  │ │ │ [TABELA COM 5 OS]   ││
│  │                         │ │ │                     ││
│  └─────────────────────────┘ │ └─────────────────────┘│
└─────────────────────────────────────────────────────────┘
```

### Funcionalidades Interativas

#### Botão "🔄 Atualizar"
- Localização: Canto superior direito
- Ação: Reseta `dados_carregados = false`, dispara nova requisição
- Permite atualizar o dashboard manualmente

#### Tabela "Últimas OS"
- Exibe as **5 últimas** ordens de serviço (ordem reversa)
- Colunas: ID, Cliente, Valor (peças + serviços), Status
- Dados vindos diretamente do banco

#### Gráficos de Linha
- **Faturamento:** Estimativa baseada no faturamento médio
- **OS por dia:** Distribuição baseada no total de OS
- Ambos usam a biblioteca `egui_plot` com cores customizadas

## 📊 Endpoint Utilizado

### `GET /ordens`

**URL:** `http://localhost:3000/ordens`

**Response:**
```json
[
  {
    "id": 1,
    "cliente": "João da Silva",
    "status": "Finalizada",
    "total_pecas": 450.00,
    "total_servicos": 120.00,
    "observacoes": "Troca de óleo",
    ...
  },
  ...
]
```

**Tipo Rust:**
```rust
Vec<OrdemServico>
```

**Handler no Server:** `handler_listar_ordens()` em `src/server.rs`

## 🚀 Melhorias Implementadas

### Antes (Dados Mockados)
```rust
// ❌ Dados hardcoded
let vendas_recentes = vec![
    (101, "Produto A".to_string(), 150.50, "Cliente X".to_string()),
    (102, "Produto B".to_string(), 320.00, "Cliente Y".to_string()),
];

let status_os = vec![
    ("Aberta".to_string(), 5),        // ← Valores inventados
    ("Em Andamento".to_string(), 8),
    ("Finalizada".to_string(), 12),
];
```

### Depois (Dados Reais)
```rust
// ✅ Busca do banco via API
crate::executor::spawn(move || {
    let ordens = client.get(format!("{}/ordens", servidor))
        .send()
        .json::<Vec<OrdemServico>>();
    
    // Processa e conta status reais
    for os in ordens.iter() {
        match os.status {
            StatusOS::Aberta => abertas += 1,  // ← Dados reais
            ...
        }
    }
});
```

## 📝 Observações e Limitações

### Limitações Atuais

1. **Gráficos de 30 dias são estimativas:**
   - Os gráficos de faturamento e OS por dia usam **distribuição simulada** baseada nos valores totais
   - **Motivo:** Não há endpoint que retorne agregação por data
   - **Solução futura:** Criar `GET /ordens/analytics` que retorne dados já agregados por dia

2. **Sem cache de dados:**
   - Cada atualização faz nova requisição ao servidor
   - **Solução futura:** Implementar cache com TTL (Time To Live)

3. **Não exibe clientes:**
   - A tabela "Últimas OS" mostra apenas ordens de serviço
   - **Dados de clientes** poderiam ser integrados via `GET /clientes`

### Próximos Passos Sugeridos

1. **Endpoint de Analytics**
   ```rust
   // GET /ordens/analytics?periodo=30d
   struct AnalyticsResponse {
       faturamento_por_dia: Vec<(String, f64)>,  // ("2025-01-01", 1500.0)
       os_por_dia: Vec<(String, u32)>,           // ("2025-01-01", 5)
       kpis: KPIs,
   }
   ```

2. **Auto-refresh periódico**
   ```rust
   // Atualizar automaticamente a cada 5 minutos
   ctx.request_repaint_after(Duration::from_secs(300));
   ```

3. **Filtros de data**
   ```rust
   // Permitir ao usuário escolher período (7, 30, 90 dias)
   ui.horizontal(|ui| {
       if ui.button("7 dias").clicked() { ... }
       if ui.button("30 dias").clicked() { ... }
       if ui.button("90 dias").clicked() { ... }
   });
   ```

4. **Indicadores de tendência**
   ```rust
   // Mostrar se os KPIs estão subindo ou descendo
   ui.label(format!("Faturamento: R$ {:.2} {}",
       total, if tendencia_positiva { "📈" } else { "📉" }
   ));
   ```

## ✅ Resultado Final

O dashboard agora:
- ✅ **Busca dados reais** do banco de dados via API REST
- ✅ **Mostra estado de carregamento** com spinner e mensagens de erro
- ✅ **Calcula KPIs reais** (faturamento, ticket médio, total de OS)
- ✅ **Exibe status das OS** baseado em dados atuais
- ✅ **Lista as últimas 5 ordens** diretamente do banco
- ✅ **Permite atualização manual** com botão "Atualizar"
- ✅ **Não trava a interface** durante o carregamento (requisições assíncronas)

## 🎓 Padrões Utilizados

1. **Arc<Mutex<T>>**: Compartilhamento thread-safe de estado
2. **Estado Assíncrono**: `EstadoCarregamento` para UI responsiva
3. **Separation of Concerns**: Busca → Processamento → Visualização
4. **Error Handling**: Tratamento de 3 tipos de erro (conexão, HTTP, parsing)
5. **Singleton HTTP Client**: Reutilização de conexões via `http_client::get_client()`
6. **Reactive UI**: `request_repaint()` para atualizar após operações assíncronas

---

**Data:** 2025-01-30  
**Versão:** 1.8.0  
**Status:** ✅ Implementado e testado (compilação bem-sucedida)
