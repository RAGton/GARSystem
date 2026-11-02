// src/os/etapa.rs
//
// Máquina de Estados Unificada da Ordem de Serviço (Fase 1 — Núcleo Operacional)
// Substitui a dicotomia status/situacao por um ciclo de vida linear, formal e auditável.

use crate::servicos::PapelUsuario;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EtapaOS {
    #[serde(alias = "RECEBIDO", alias = "Recebido", alias = "recebido")]
    Recebido,
    #[serde(alias = "EM_DIAGNOSTICO", alias = "EmDiagnostico", alias = "em_diagnostico")]
    EmDiagnostico,
    #[serde(alias = "ORCAMENTO_GERADO", alias = "OrcamentoGerado", alias = "orcamento_gerado")]
    OrcamentoGerado,
    #[serde(alias = "APROVADO", alias = "Aprovado", alias = "aprovado")]
    Aprovado,
    #[serde(alias = "EM_EXECUCAO", alias = "EmExecucao", alias = "em_execucao")]
    EmExecucao,
    #[serde(alias = "AGUARDANDO_PECA", alias = "AguardandoPeca", alias = "aguardando_peca")]
    AguardandoPeca,
    #[serde(alias = "TESTES_CONTROLE_QUALIDADE", alias = "TestesControleQualidade", alias = "testes_controle_qualidade")]
    TestesControleQualidade,
    #[serde(alias = "PRONTO_RETIRADA", alias = "ProntoRetirada", alias = "pronto_retirada")]
    ProntoRetirada,
    #[serde(alias = "FINALIZADO", alias = "Finalizado", alias = "finalizado")]
    Finalizado,
    #[serde(alias = "CANCELADO", alias = "Cancelado", alias = "cancelado")]
    Cancelado,
    #[serde(alias = "REPROVADO", alias = "Reprovado", alias = "reprovado")]
    Reprovado,
}

impl Default for EtapaOS {
    fn default() -> Self {
        Self::Recebido
    }
}

impl EtapaOS {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Recebido => "RECEBIDO",
            Self::EmDiagnostico => "EM_DIAGNOSTICO",
            Self::OrcamentoGerado => "ORCAMENTO_GERADO",
            Self::Aprovado => "APROVADO",
            Self::EmExecucao => "EM_EXECUCAO",
            Self::AguardandoPeca => "AGUARDANDO_PECA",
            Self::TestesControleQualidade => "TESTES_CONTROLE_QUALIDADE",
            Self::ProntoRetirada => "PRONTO_RETIRADA",
            Self::Finalizado => "FINALIZADO",
            Self::Cancelado => "CANCELADO",
            Self::Reprovado => "REPROVADO",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "RECEBIDO" => Some(Self::Recebido),
            "EM_DIAGNOSTICO" | "DIAGNOSTICO" => Some(Self::EmDiagnostico),
            "ORCAMENTO_GERADO" | "ORCAMENTO" => Some(Self::OrcamentoGerado),
            "APROVADO" => Some(Self::Aprovado),
            "EM_EXECUCAO" | "EXECUCAO" | "EM_ANDAMENTO" => Some(Self::EmExecucao),
            "AGUARDANDO_PECA" => Some(Self::AguardandoPeca),
            "TESTES_CONTROLE_QUALIDADE" | "TESTES" => Some(Self::TestesControleQualidade),
            "PRONTO_RETIRADA" | "PRONTO" => Some(Self::ProntoRetirada),
            "FINALIZADO" | "CONCLUIDO" => Some(Self::Finalizado),
            "CANCELADO" => Some(Self::Cancelado),
            "REPROVADO" => Some(Self::Reprovado),
            _ => None,
        }
    }

    pub fn nome_amigavel(&self) -> &'static str {
        match self {
            Self::Recebido => "Recebido",
            Self::EmDiagnostico => "Em Diagnóstico",
            Self::OrcamentoGerado => "Orçamento Gerado",
            Self::Aprovado => "Aprovado",
            Self::EmExecucao => "Em Execução",
            Self::AguardandoPeca => "Aguardando Peça",
            Self::TestesControleQualidade => "Controle de Qualidade",
            Self::ProntoRetirada => "Pronto para Retirada",
            Self::Finalizado => "Finalizado",
            Self::Cancelado => "Cancelado",
            Self::Reprovado => "Reprovado",
        }
    }

    pub fn eh_terminal(&self) -> bool {
        matches!(self, Self::Finalizado | Self::Cancelado | Self::Reprovado)
    }

    /// Retorna as possíveis próximas etapas no grafo operacional puro.
    pub fn proximas_etapas_possiveis(&self) -> &'static [EtapaOS] {
        match self {
            Self::Recebido => &[Self::EmDiagnostico, Self::Cancelado],
            Self::EmDiagnostico => &[Self::OrcamentoGerado, Self::Cancelado],
            Self::OrcamentoGerado => &[Self::Aprovado, Self::Reprovado, Self::Cancelado],
            Self::Aprovado => &[Self::EmExecucao, Self::Cancelado],
            Self::EmExecucao => &[Self::AguardandoPeca, Self::TestesControleQualidade, Self::Cancelado],
            Self::AguardandoPeca => &[Self::EmExecucao, Self::Cancelado],
            Self::TestesControleQualidade => &[Self::ProntoRetirada, Self::EmExecucao, Self::Cancelado],
            Self::ProntoRetirada => &[Self::Finalizado, Self::Cancelado],
            Self::Finalizado | Self::Cancelado | Self::Reprovado => &[],
        }
    }

    /// Filtra as transições válidas pelo papel do usuário.
    pub fn transicoes_permitidas(&self, papel: PapelUsuario) -> Vec<EtapaOS> {
        self.proximas_etapas_possiveis()
            .iter()
            .copied()
            .filter(|&destino| validar_papel_para_transicao(*self, destino, papel).is_ok())
            .collect()
    }

    /// Mapeia para os enums legados (garantindo compatibilidade reversa com o banco)
    pub fn para_status_legado(&self) -> (&'static str, &'static str) {
        match self {
            Self::Recebido => ("Aberta", "Orcamento"),
            Self::EmDiagnostico => ("Aberta", "Orcamento"),
            Self::OrcamentoGerado => ("Aberta", "AguardandoAutorizacao"),
            Self::Aprovado => ("Aberta", "Aprovado"),
            Self::EmExecucao => ("EmAndamento", "EmAndamento"),
            Self::AguardandoPeca => ("AguardandoPeca", "AutorizadoAguardandoPeca"),
            Self::TestesControleQualidade => ("EmAndamento", "ServicoConcluido"),
            Self::ProntoRetirada => ("Aberta", "AguardandoRetirada"),
            Self::Finalizado => ("Finalizada", "Faturado"),
            Self::Cancelado => ("Cancelada", "Orcamento"),
            Self::Reprovado => ("Aberta", "Reprovado"),
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ErroTransicaoOS {
    #[error("OS está no estado terminal '{0}' e não pode mais ser alterada.")]
    EstadoTerminal(String),
    #[error("Transição inválida de '{origem}' para '{destino}'.")]
    TransicaoInvalida { origem: String, destino: String },
    #[error("Usuário com papel '{papel}' não tem permissão para transicionar de '{origem}' para '{destino}'.")]
    PermissaoNegada {
        papel: String,
        origem: String,
        destino: String,
    },
}

pub fn validar_papel_para_transicao(
    origem: EtapaOS,
    destino: EtapaOS,
    papel: PapelUsuario,
) -> Result<(), ErroTransicaoOS> {
    match papel {
        PapelUsuario::Administrador | PapelUsuario::Gerencia => Ok(()),
        PapelUsuario::Comercial => match (origem, destino) {
            (EtapaOS::Recebido, EtapaOS::EmDiagnostico)
            | (EtapaOS::OrcamentoGerado, EtapaOS::Aprovado)
            | (EtapaOS::OrcamentoGerado, EtapaOS::Reprovado)
            | (EtapaOS::ProntoRetirada, EtapaOS::Finalizado)
            | (_, EtapaOS::Cancelado) => Ok(()),
            _ => Err(ErroTransicaoOS::PermissaoNegada {
                papel: format!("{:?}", papel),
                origem: origem.as_str().to_string(),
                destino: destino.as_str().to_string(),
            }),
        },
        PapelUsuario::Tecnico => match (origem, destino) {
            (EtapaOS::Recebido, EtapaOS::EmDiagnostico)
            | (EtapaOS::EmDiagnostico, EtapaOS::OrcamentoGerado)
            | (EtapaOS::Aprovado, EtapaOS::EmExecucao)
            | (EtapaOS::EmExecucao, EtapaOS::AguardandoPeca)
            | (EtapaOS::AguardandoPeca, EtapaOS::EmExecucao)
            | (EtapaOS::EmExecucao, EtapaOS::TestesControleQualidade)
            | (EtapaOS::TestesControleQualidade, EtapaOS::ProntoRetirada)
            | (EtapaOS::TestesControleQualidade, EtapaOS::EmExecucao) => Ok(()),
            _ => Err(ErroTransicaoOS::PermissaoNegada {
                papel: format!("{:?}", papel),
                origem: origem.as_str().to_string(),
                destino: destino.as_str().to_string(),
            }),
        },
        PapelUsuario::Estoquista => match (origem, destino) {
            (EtapaOS::EmExecucao, EtapaOS::AguardandoPeca)
            | (EtapaOS::AguardandoPeca, EtapaOS::EmExecucao) => Ok(()),
            _ => Err(ErroTransicaoOS::PermissaoNegada {
                papel: format!("{:?}", papel),
                origem: origem.as_str().to_string(),
                destino: destino.as_str().to_string(),
            }),
        },
        PapelUsuario::Financeiro => match (origem, destino) {
            (EtapaOS::ProntoRetirada, EtapaOS::Finalizado) => Ok(()),
            _ => Err(ErroTransicaoOS::PermissaoNegada {
                papel: format!("{:?}", papel),
                origem: origem.as_str().to_string(),
                destino: destino.as_str().to_string(),
            }),
        },
    }
}

pub fn validar_transicao(
    origem: EtapaOS,
    destino: EtapaOS,
    papel: PapelUsuario,
) -> Result<(), ErroTransicaoOS> {
    if origem.eh_terminal() {
        return Err(ErroTransicaoOS::EstadoTerminal(origem.as_str().to_string()));
    }
    if origem == destino {
        return Ok(());
    }
    if !origem.proximas_etapas_possiveis().contains(&destino) {
        return Err(ErroTransicaoOS::TransicaoInvalida {
            origem: origem.as_str().to_string(),
            destino: destino.as_str().to_string(),
        });
    }
    validar_papel_para_transicao(origem, destino, papel)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ciclo_completo_feliz() {
        let papel_tec = PapelUsuario::Tecnico;
        let papel_com = PapelUsuario::Comercial;

        // Recebido -> EmDiagnostico (Tecnico)
        assert!(validar_transicao(EtapaOS::Recebido, EtapaOS::EmDiagnostico, papel_tec).is_ok());

        // EmDiagnostico -> OrcamentoGerado (Tecnico)
        assert!(validar_transicao(EtapaOS::EmDiagnostico, EtapaOS::OrcamentoGerado, papel_tec).is_ok());

        // OrcamentoGerado -> Aprovado (Comercial)
        assert!(validar_transicao(EtapaOS::OrcamentoGerado, EtapaOS::Aprovado, papel_com).is_ok());

        // Aprovado -> EmExecucao (Tecnico)
        assert!(validar_transicao(EtapaOS::Aprovado, EtapaOS::EmExecucao, papel_tec).is_ok());

        // EmExecucao -> TestesControleQualidade (Tecnico)
        assert!(validar_transicao(EtapaOS::EmExecucao, EtapaOS::TestesControleQualidade, papel_tec).is_ok());

        // TestesControleQualidade -> ProntoRetirada (Tecnico)
        assert!(validar_transicao(EtapaOS::TestesControleQualidade, EtapaOS::ProntoRetirada, papel_tec).is_ok());

        // ProntoRetirada -> Finalizado (Comercial)
        assert!(validar_transicao(EtapaOS::ProntoRetirada, EtapaOS::Finalizado, papel_com).is_ok());
    }

    #[test]
    fn bloqueia_pulo_de_etapa() {
        // Recebido direto para Finalizado deve falhar
        let res = validar_transicao(EtapaOS::Recebido, EtapaOS::Finalizado, PapelUsuario::Administrador);
        assert_eq!(
            res,
            Err(ErroTransicaoOS::TransicaoInvalida {
                origem: "RECEBIDO".to_string(),
                destino: "FINALIZADO".to_string()
            })
        );
    }

    #[test]
    fn bloqueia_papel_sem_permissao() {
        // Comercial não pode avançar para EmExecucao
        let res = validar_transicao(EtapaOS::Aprovado, EtapaOS::EmExecucao, PapelUsuario::Comercial);
        assert!(matches!(res, Err(ErroTransicaoOS::PermissaoNegada { .. })));
    }

    #[test]
    fn estado_terminal_imutavel() {
        let res = validar_transicao(EtapaOS::Finalizado, EtapaOS::EmExecucao, PapelUsuario::Administrador);
        assert_eq!(
            res,
            Err(ErroTransicaoOS::EstadoTerminal("FINALIZADO".to_string()))
        );
    }

    #[test]
    fn retorno_de_testes_para_execucao() {
        // Se falhar no controle de qualidade, técnico pode retornar para EmExecucao
        assert!(validar_transicao(
            EtapaOS::TestesControleQualidade,
            EtapaOS::EmExecucao,
            PapelUsuario::Tecnico
        ).is_ok());
    }
}
