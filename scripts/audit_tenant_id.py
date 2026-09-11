#!/usr/bin/env python3
"""
REGRA 11 — AUDITORIA FORENSE DO SQL EXECUTÁVEL
A única métrica aceita para multi-tenant é esta:
- 100% das queries SELECT/UPDATE/DELETE/INSERT em tabelas tenant-aware
  DEVEM ter filtro `tenant_id` ou `empresa_id` no SQL.
"""

import re
import os
import sys


def main():
    tabelas_com_tenant = {
        'alertas', 'alertas_financeiros', 'alertas_financeiros_ocorrencias',
        'alertas_ocorrencias', 'arquivo_thumbnails', 'arquivo_vinculos',
        'arquivos', 'centros_custo', 'cliente_anexos', 'cliente_contatos',
        'cliente_observacao_edicoes', 'cliente_observacoes', 'cliente_tag_atribuicoes',
        'cliente_tags', 'cliente_timeline_eventos', 'clientes', 'configuracao_financeira',
        'contas_pagar', 'contas_receber', 'cotacao_anexos', 'cotacao_itens', 'cotacoes',
        'equipamentos', 'eventos_agenda', 'historico_edicoes', 'lancamentos',
        'movimentacoes', 'movimentos_estoque', 'orcamento_aprovacoes', 'orcamento_historico',
        'orcamento_items', 'orcamentos', 'ordem_servico_pecas', 'ordem_servico_servicos',
        'ordens_servico', 'os_assinaturas', 'os_auditoria_campo', 'os_checklist_itens',
        'os_checklist_template_itens', 'os_checklist_templates', 'os_checklists', 'os_evolucoes',
        'pecas', 'plano_contas', 'servicos', 'sla_calculos', 'sla_config', 'sla_eventos',
        'transcricoes', 'workflow_definicoes', 'workflow_estados', 'workflow_movimentacoes',
        'workflow_transicoes', 'user_preferences', 'login_audit', 'tenant_branding',
        'dashboard_layouts', 'workspace_preferences',
    }

    stats = {'SELECT': [0, 0], 'UPDATE': [0, 0], 'DELETE': [0, 0], 'INSERT': [0, 0]}
    unsafe_list = []

    for root, _, fs in os.walk('src'):
        for f in fs:
            if not f.endswith('.rs'):
                continue
            fp = os.path.join(root, f)
            with open(fp) as fh:
                content = fh.read()
            # Raw strings multi-linha
            for m in re.finditer(r'r#"(.*?)"#', content, re.DOTALL):
                sql = m.group(1).strip()
                sql_first = sql.split('\n')[0].strip().upper()
                sql_type = None
                if sql_first.startswith(('SELECT', 'INSERT', 'UPDATE', 'DELETE')):
                    sql_type = sql_first.split()[0]
                if not sql_type or len(sql) < 15:
                    continue
                toca_tenant = any(re.search(rf'\b{t}\b', sql, re.IGNORECASE) for t in tabelas_com_tenant)
                if not toca_tenant:
                    continue
                tem_filtro = 'tenant_id' in sql or 'empresa_id' in sql
                stats[sql_type][0] += 1
                if tem_filtro:
                    stats[sql_type][1] += 1
                else:
                    unsafe_list.append((fp.split('/')[-1], sql_type, sql[:120].replace('\n', ' ')))
            # Strings normais
            for m in re.finditer(r'"((?:SELECT|INSERT|UPDATE|DELETE)[^"\\]*(?:\\.[^"\\]*)*)"', content):
                sql = m.group(1).strip()
                if len(sql) < 15:
                    continue
                sql_type = sql.split()[0].upper()
                toca_tenant = any(re.search(rf'\b{t}\b', sql, re.IGNORECASE) for t in tabelas_com_tenant)
                if not toca_tenant:
                    continue
                tem_filtro = 'tenant_id' in sql or 'empresa_id' in sql
                stats[sql_type][0] += 1
                if tem_filtro:
                    stats[sql_type][1] += 1
                else:
                    unsafe_list.append((fp.split('/')[-1], sql_type, sql[:120].replace('\n', ' ')))

    print("=" * 70)
    print("REGRA 11 — AUDITORIA SQL EXECUTÁVEL")
    print("=" * 70)
    print(f"{'Tipo':<8} {'Total':<7} {'+tenant_id':<12} {'%':<8}")
    print("-" * 70)
    total_all = 0
    total_safe = 0
    for t, (total, safe) in stats.items():
        pct = (safe / total * 100) if total else 0
        total_all += total
        total_safe += safe
        print(f"{t:<8} {total:<7} {safe:<12} {pct:.1f}%")
    pct = (total_safe / total_all * 100) if total_all else 0
    print(f"{'TOTAL':<8} {total_all:<7} {total_safe:<12} {pct:.2f}%")
    print()
    if unsafe_list:
        print(f"⚠ {len(unsafe_list)} queries SEM tenant_id:")
        for f, t, q in unsafe_list:
            print(f"  [{t}] {f:<30} {q[:100]}")
        sys.exit(1)
    else:
        print("✓ ZERO vazamentos")
        sys.exit(0)


if __name__ == '__main__':
    main()
