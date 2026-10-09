# GAR System — Ideias, diagnóstico de UI e direção arquitetural

> Documento de visão e decisões propostas para evoluir o GAR System para um ERP/CRM profissional de assistência técnica. Este arquivo orienta a implementação; não significa que os itens já estejam implementados.

## 1. Diagnóstico da interface atual

Com base nas capturas de tela fornecidas, a interface tem identidade visual reconhecível (tema escuro, azul/ciano e logo GAR System), mas ainda transmite aparência de protótipo/desktop técnico, em vez de um ERP comercial maduro.

### Problemas observáveis
- **Grande área sem conteúdo:** a tela de Movimentações mostra “WIP: Tela de Movimentações de Estoque”. Isso é um placeholder explícito, não uma funcionalidade concluída.
- **Hierarquia visual fraca:** título, abas e conteúdo estão comprimidos na parte superior; a área útil não oferece contexto, ações ou estados vazios úteis.
- **Navegação com aparência antiga:** itens parecem pequenos botões cinza independentes, com espaçamento irregular e estado ativo pouco evidente.
- **Identidade inconsistente:** logo, barra de título, sidebar e conteúdo não parecem pertencer ao mesmo sistema de design.
- **Falta de contexto persistente:** dentro do ERP não há uma área clara e consistente para identificar usuário autenticado, empresa atual e hora local.
- **Ações pouco evidentes:** ações como cadastrar peça precisam ser apresentadas como ações primárias.
- **Estados insuficientes:** telas sem dados devem distinguir carregamento, lista vazia, erro de API, falta de permissão e conteúdo indisponível.
- **Funcionalidades futuras misturadas à navegação:** itens não implementados não devem parecer módulos prontos. Devem ser ocultos em produção ou apresentados com status explícito.
- **Possível mistura de interfaces:** a captura parece vir de UI desktop com aparência de egui, enquanto o projeto também possui frontend Next.js. Confirmar no repositório qual executável é o caminho principal.

### Objetivo visual
Adotar uma interface de ERP contemporânea, organizada e eficiente: superfícies escuras em grafite/azul-marinho, acentos azul/ciano contidos, tipografia legível, espaçamento sistemático, ícones consistentes, tabelas profissionais e ações claramente hierarquizadas. Evitar efeitos excessivos, grandes áreas vazias e cartões decorativos sem valor operacional.

## 2. Decisão arquitetural proposta: não usar egui como UI principal

### Recomendação
**Não investir em egui como interface principal do ERP.** A direção arquitetural já definida é **Next.js para Web e Tauri v2 + Next.js para Desktop**, mantendo Rust/Axum, MySQL, RBAC e multi-tenant no backend.

A UI egui existente deve ser tratada como legado/compatibilidade até que o inventário confirme se ainda é necessária. Não a apague nem desative abruptamente: primeiro verifique binários, comandos de execução, usuários e dependências. Evite duas interfaces distintas para o mesmo produto.

### Motivos
- **Consistência visual:** uma base de componentes e padrões para Web e Desktop.
- **Velocidade de evolução:** tabelas, formulários, filtros, diálogos e dashboards são naturais no ecossistema web.
- **Reutilização:** o desktop pode empacotar a aplicação Next.js com Tauri v2.
- **Ecossistema:** acessibilidade, design systems, formulários e testes de interface têm ferramentas maduras.
- **Backend preservado:** Rust/Axum continua responsável por regras de negócio, validação, autenticação e autorização; MySQL pela persistência.

### Custos e riscos
- Definir corretamente como o frontend será servido no Tauri e como ele alcança a API.
- Manter autenticação coerente entre Web e Desktop; não misturar cookie HttpOnly com armazenamento inseguro de tokens.
- Testar CORS, HTTPS, sessão, atualizações e empacotamento.
- Não duplicar regras de negócio no frontend.
- Não migrar tudo de uma vez: primeiro consolidar Next.js e provar um fluxo completo.

### Política para egui legado
1. Identificar referências ao binário egui e suas telas.
2. Confirmar se ainda é executado ou empacotado.
3. Documentar funções existentes apenas nele.
4. Reimplementar e testar essas funções na UI principal antes de retirar qualquer caminho legado.
5. Propor remoção em mudança separada após comprovar paridade funcional.

## 3. Shell global do ERP: conteúdo persistente

Criar layout compartilhado entre todos os módulos:
1. **Barra superior:** contexto da empresa/unidade, relógio local, usuário autenticado, perfil e sair.
2. **Sidebar:** navegação agrupada, item ativo evidente, opção de recolher e módulos conforme permissões.
3. **Cabeçalho de página:** título, descrição, breadcrumb quando útil e ação principal.
4. **Área de trabalho:** tabelas, formulários, dashboards e conteúdo do módulo.
5. **Feedback global:** toasts acessíveis, erros compreensíveis e carregamento sem bloquear desnecessariamente todo o sistema.

### 3.1 Relógio persistente
Exibir data e hora local na barra superior de todas as telas autenticadas.

- Usar relógio do dispositivo para exibição local e atualizar eficientemente, sem timers duplicados por página.
- Preferir formato brasileiro legível, por exemplo: 09/10/2026 14:55, respeitando idioma/fuso configurados quando suportados.
- O relógio é informativo, não fonte de autoridade para auditoria ou transações financeiras. Datas persistidas devem seguir a política temporal do backend.
- O requisito é o shell autenticado; não é necessário adicionar relógio ao login.
- Em telas estreitas, compactar a apresentação sem quebrar navegação.
- Testar atualização, mudança de rota e retomada após suspensão da janela.

### 3.2 Identidade do usuário autenticado
Mostrar permanentemente um controle compacto com:
- nome de exibição real;
- avatar ou iniciais geradas localmente quando não houver imagem;
- papel/perfil somente se disponível e autorizado;
- empresa/tenant atual quando o contexto multiempresa exigir;
- menu com conta/preferências/sair somente para ações realmente implementadas.

Segurança:
- Identidade obtida da sessão autenticada ou endpoint confiável, nunca de nome hardcoded ou apenas de localStorage.
- API continua sendo autoridade para identidade, permissões e tenant.
- Não expor e-mail, documentos ou dados sensíveis sem necessidade.
- Ao expirar a sessão, limpar estado e encaminhar ao login com segurança.
- Sair deve invalidar a sessão no backend conforme a arquitetura real.
- Se ainda não houver endpoint de usuário atual, registrar a dependência e implementar contrato seguro; não simular dados.

### 3.3 Contexto da empresa
Em ambiente multi-tenant, deixar claro em qual empresa o usuário opera. A troca de empresa, se permitida, precisa ser explícita e validada pelo backend. Nunca basta trocar um identificador no frontend nem misturar dados entre empresas.

## 4. Sistema de design compartilhado

Normalizar tokens e componentes antes de redesenhar todos os módulos.

### Tokens
- Cores de fundo, superfície, borda, texto e estados.
- Escala de espaçamento, tipografia e alturas de controles.
- Raios, foco, elevação e transições.
- Breakpoints para desktop, notebook e janela compacta.

### Componentes
- AppShell, Topbar, Sidebar, Breadcrumbs e PageHeader.
- Botões primários, secundários, terciários e destrutivos.
- Inputs, selects, date pickers, formulários e validações.
- DataTable com ordenação, filtros e paginação quando necessários.
- Dialog/Drawer, confirmação de ações destrutivas e menus.
- Badge de status, loading/skeleton, EmptyState, ErrorState e toast.
- Acessibilidade por teclado, foco visível, labels e contraste.

Verificar dependências existentes antes de adicionar biblioteca. Evitar componentes duplicados com pequenas diferenças entre módulos.

## 5. Direção por tela

### Dashboard
- Resumo de OS abertas, atrasadas, aguardando aprovação e prontas para entrega.
- Indicadores financeiros e de estoque somente quando houver consultas confiáveis.
- Atalhos para ações frequentes e atividade recente com permissões adequadas.
- Nunca inventar números nem usar dados de demonstração em produção.

### Estoque
- Cabeçalho com título, descrição e botão primário “Nova peça”.
- Abas de peças, fornecedores, entrada de nota fiscal e movimentações apenas se os recursos existirem.
- Busca ampla, filtros funcionais e tabela legível.
- Quantidade disponível, unidade, custo/preço e localização conforme o modelo real.
- Estado vazio com orientação e ação de cadastro.
- Movimentações com tipo, quantidade, motivo, origem, usuário e data conforme domínio e permissões.
- Não apresentar “WIP” como tela de produção. Até implementar, ocultar a rota ou mostrar claramente “Em desenvolvimento” apenas em ambiente apropriado.

### Clientes/CRM
- Pesquisa rápida e tabela clara.
- Perfil de cliente com contatos, equipamentos, OS, orçamentos e histórico.
- Ações principais acessíveis sem navegar por módulos desconectados.
- Respeitar permissões e privacidade.

### Ordens de serviço
- Lista/Kanban com filtros, busca e estados reais.
- Ação “Nova OS” clara.
- Detalhes organizados em cliente/equipamento, diagnóstico, responsável, peças, orçamento/aprovação, execução, histórico e entrega conforme o domínio existente.
- Transições validadas no backend; pendências e próximas ações evidentes.

### Orçamentos, cotações e financeiro
- Totais claros, moeda consistente e histórico de alterações.
- Separar cotação de fornecedor, orçamento ao cliente, pedido de compra e pagamento.
- Confirmar operações críticas com resumo do impacto.
- Valores calculados no backend.
- Nunca sinalizar sucesso sem confirmação de persistência.

### Administração
- Separar usuários, papéis, empresa, integrações e auditoria.
- Mostrar módulos conforme RBAC real.
- Não apresentar funcionalidades futuras como navegação operacional normal.

## 6. Fluxos e estados obrigatórios
Toda tela que consulta ou altera dados precisa definir: carregando, resultados, vazia, falha de API, não autenticado, sem permissão, salvando, sucesso confirmado, validação e conflito quando aplicável.

Botões sem ação, handlers vazios, mocks em produção e mensagens falsas de sucesso são defeitos. Cada ação precisa de endpoint/serviço real ou ser explicitamente marcada como indisponível.

## 7. Plano incremental

### Fase 0 — descoberta
- [ ] Confirmar executável/interface realmente usada.
- [ ] Mapear Next.js, Tauri, egui legado e backend.
- [ ] Inventariar rotas, telas, ações e contratos de API.
- [ ] Confirmar autenticação, sessão, usuário atual e tenant.
- [ ] Criar tracker de execução com evidências.

### Fase 1 — base visual
- [ ] Consolidar tokens de design.
- [ ] Implementar AppShell compartilhado.
- [ ] Implementar Topbar persistente com relógio e identidade real do usuário.
- [ ] Refinar Sidebar e PageHeader.
- [ ] Validar responsividade, teclado e contraste.

### Fase 2 — prova vertical
- [ ] Modernizar a tela de estoque.
- [ ] Ligar busca, listagem, cadastro, edição e movimentações à API real.
- [ ] Implementar loading, empty state, erro e sucesso.
- [ ] Testar persistência e permissões.
- [ ] Garantir relógio e usuário visíveis ao navegar.

### Fase 3 — expansão
- [ ] Aplicar componentes compartilhados a clientes, OS, orçamento, financeiro, relatórios e administração, se existirem.
- [ ] Validar cada módulo com testes de integração.
- [ ] Não redesenhar todas as telas simultaneamente.

### Fase 4 — produção
- [ ] Executar lint, typecheck, testes e builds reais.
- [ ] Fazer smoke tests dos fluxos completos.
- [ ] Verificar segurança, isolamento multi-tenant e sessão.
- [ ] Validar empacotamento Web/Desktop e configuração de produção.
- [ ] Documentar limitações e critérios não atendidos.

## 8. Critérios de aceite
- Todas as telas autenticadas compartilham um shell consistente.
- Data/hora local e usuário autenticado permanecem visíveis em todas as telas internas.
- Identidade e empresa vêm de fontes confiáveis; nada hardcoded.
- Navegação, botões e formulários mantêm suas funções.
- Estados de carregamento, vazio e erro são claros.
- Dados persistem; mocks não fingem ser produção.
- UI e API respeitam autenticação, RBAC e isolamento multi-tenant.
- Web e Desktop compartilham a base visual quando a arquitetura permite.
- Build, testes e smoke tests aplicáveis passam e seus resultados são documentados.
- Nenhuma tela é declarada pronta sem evidência.

## 9. Restrições
- Não apagar egui antes de comprovar paridade funcional e uso real.
- Não manter egui e Next.js como duas UIs principais divergentes sem decisão documentada.
- Não alterar schema ou API apenas por estética.
- Não executar operações destrutivas, deploy ou migrações destrutivas em produção sem autorização.
- Preservar mudanças existentes no Git e revisar diffs.
- Implementar em lotes pequenos, testando cada lote.

## 10. Primeira tarefa concreta
1. Inspecione o repositório e confirme qual UI gerou a captura.
2. Identifique frontend principal e estado real da integração com a API.
3. Faça diagnóstico curto com evidências.
4. Implemente shell global com relógio e identidade real do usuário, sem dados simulados.
5. Refine sidebar e cabeçalho com componentes compartilhados.
6. Modernize a tela de estoque como primeira tela-piloto.
7. Rode testes e registre resultados.
8. Continue para a próxima tela sem parar após apenas um relatório.

**Princípio central:** o GAR System deve parecer um produto único, não um conjunto de telas desconectadas. A modernização precisa melhorar a operação real, não mascarar funcionalidades ausentes.
