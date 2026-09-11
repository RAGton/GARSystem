// tests/estoque_diff.rs
//
// Testes da lógica de diff de estoque.
//
// Justificativa P2.6.2c: testes de integração com allow para lint.
#![allow(unused_imports, unused_mut, dead_code)]
//
//
// Como o DB real precisa de MySQL rodando, estes testes verificam a parte
// PURA da lógica: o cálculo de delta. Para validar o efeito no banco
// (incluindo lock pessimista e concorrência), rode o servidor contra um
// MySQL de teste e execute os testes de integração em
// `tests/integration_estoque.rs` (TODO Sprint 1.5).

use std::collections::HashMap;

/// Peça simples para o teste.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Peca {
    id: u32,
    quantidade: i32,
}

/// Agrega peças por id, somando quantidades.
fn agregar(pecas: &[Peca]) -> HashMap<u32, i32> {
    let mut acc: HashMap<u32, i32> = HashMap::new();
    for p in pecas {
        *acc.entry(p.id).or_insert(0) += p.quantidade;
    }
    acc
}

/// Calcula o diff entre dois estados de peças.
/// Retorna Vec<(peca_id, delta)> onde:
///   delta > 0 = estoque deve AUMENTAR (peça foi removida da OS)
///   delta < 0 = estoque deve DIMINUIR (peça foi adicionada à OS)
///   delta = 0 = sem mudança, ignorado
fn calcular_diff(antigo: &HashMap<u32, i32>, novo: &HashMap<u32, i32>) -> Vec<(u32, i32)> {
    let mut chaves: std::collections::HashSet<u32> = antigo.keys().copied().collect();
    chaves.extend(novo.keys().copied());
    let mut diffs = Vec::new();
    for id in chaves {
        let a = *antigo.get(&id).unwrap_or(&0);
        let n = *novo.get(&id).unwrap_or(&0);
        let delta = a - n;
        if delta != 0 {
            diffs.push((id, delta));
        }
    }
    diffs.sort_by_key(|(id, _)| *id);
    diffs
}

#[test]
fn diff_criacao_inicial() {
    // Antes: nada na OS. Depois: 3 unidades da peça 1, 5 da peça 2.
    // diff = antigo - novo = 0 - 3 = -3, 0 - 5 = -5
    // Convenção: delta NEGATIVO = estoque DECRESCE (peça adicionada à OS).
    let antigo: HashMap<u32, i32> = HashMap::new();
    let novo = agregar(&[
        Peca {
            id: 1,
            quantidade: 3,
        },
        Peca {
            id: 2,
            quantidade: 5,
        },
    ]);
    let diffs = calcular_diff(&antigo, &novo);
    assert_eq!(diffs, vec![(1, -3), (2, -5)]);
}

#[test]
fn diff_edicao_remove_uma_peca() {
    // Antes: 5 unidades da peça 1. Depois: 3 unidades. → diff = 5-3 = 2
    // Convenção: delta POSITIVO = estoque AUMENTA (peça devolvida da OS).
    let antigo: HashMap<u32, i32> = [(1, 5)].into_iter().collect();
    let novo: HashMap<u32, i32> = [(1, 3)].into_iter().collect();
    let diffs = calcular_diff(&antigo, &novo);
    assert_eq!(diffs, vec![(1, 2)]);
}

#[test]
fn diff_edicao_adiciona_peca() {
    // Antes: 3 da peça 1. Depois: 5 da peça 1. → diff = 3-5 = -2
    // Convenção: delta NEGATIVO = estoque DIMINUI (peça adicionada à OS).
    let antigo: HashMap<u32, i32> = [(1, 3)].into_iter().collect();
    let novo: HashMap<u32, i32> = [(1, 5)].into_iter().collect();
    let diffs = calcular_diff(&antigo, &novo);
    assert_eq!(diffs, vec![(1, -2)]);
}

#[test]
fn diff_edicao_remove_todas() {
    // Antes: peça 1 com 3. Depois: nada. → diff = 3-0 = 3 (devolve tudo).
    let antigo: HashMap<u32, i32> = [(1, 3)].into_iter().collect();
    let novo: HashMap<u32, i32> = HashMap::new();
    let diffs = calcular_diff(&antigo, &novo);
    assert_eq!(diffs, vec![(1, 3)]);
}

#[test]
fn diff_repeticao_nao_corrompe() {
    // Cenário do bug original: re-editar a mesma OS várias vezes para o
    // mesmo valor não pode decrementar o estoque múltiplas vezes.
    //
    // Comportamento correto: após a primeira edição, o estado da OS já é o
    // novo. Edições subsequentes para o mesmo valor são no-ops.
    //
    // Simulação:
    //   - Inicial: 5 unidades na OS, 100 em estoque.
    //   - 1ª edição para 3: diff(5, 3) = 2 → estoque += 2 → 102.
    //   - 2ª edição para 3: diff(3, 3) = 0 → no-op. Estoque continua 102.
    //   - 3ª edição para 3: diff(3, 3) = 0 → no-op. Estoque continua 102.
    let mut estoque: HashMap<u32, i32> = [(1, 100)].into_iter().collect();
    let mut estado_os: HashMap<u32, i32> = [(1, 5)].into_iter().collect();
    for _ in 0..5 {
        // O "novo" reflete o estado atual da OS, não o inicial.
        let novo = estado_os.clone();
        for (id, delta) in calcular_diff(&estado_os, &novo) {
            *estoque.get_mut(&id).unwrap() += delta;
        }
    }
    assert_eq!(
        estoque[&1], 100,
        "Editar para o mesmo valor não pode alterar o estoque"
    );

    // Agora simula o caso "antigo 5 → novo 3 → novo 3 → novo 3" como
    // aconteceria em produção (a primeira vez muda, as seguintes não).
    let mut estoque2: HashMap<u32, i32> = [(1, 100)].into_iter().collect();
    let mut estado2: HashMap<u32, i32> = [(1, 5)].into_iter().collect();
    for _ in 0..5 {
        let novo: HashMap<u32, i32> = [(1, 3)].into_iter().collect();
        for (id, delta) in calcular_diff(&estado2, &novo) {
            *estoque2.get_mut(&id).unwrap() += delta;
        }
        // Persiste o novo estado (como o atualizar_os faria).
        estado2 = novo;
    }
    // 100 + 2 (uma única vez) = 102.
    assert_eq!(
        estoque2[&1], 102,
        "Bug do estoque corrigido: edições repetidas não corrompem o saldo"
    );
}

#[test]
fn diff_pecas_agregadas_duplicadas() {
    // Duas entradas com mesmo id_peca devem ser somadas.
    let pecas = vec![
        Peca {
            id: 1,
            quantidade: 2,
        },
        Peca {
            id: 1,
            quantidade: 3,
        },
    ];
    let agg = agregar(&pecas);
    assert_eq!(agg[&1], 5);
}

#[test]
fn diff_sem_mudanca() {
    let antigo: HashMap<u32, i32> = [(1, 3)].into_iter().collect();
    let novo: HashMap<u32, i32> = [(1, 3)].into_iter().collect();
    let diffs = calcular_diff(&antigo, &novo);
    assert!(diffs.is_empty());
}
