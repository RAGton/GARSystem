"use client"

import { useState } from "react"
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import {
  BarChart3,
  FileSpreadsheet,
  Download,
  Calendar,
  Filter,
  Info,
  X,
} from "lucide-react"

const relatoriosDisponiveis = [
  {
    id: "REP-01",
    titulo: "Relatório de Faturamento Mensal",
    descricao:
      "Detalhamento de todas as receitas e fluxo de caixa acumulado do mês corrente.",
    tipo: "Financeiro",
    data: "Disponível após /financeiro/dashboard",
  },
  {
    id: "REP-02",
    titulo: "Desempenho de Ordens de Serviço (SLA)",
    descricao:
      "Análise percentual de cumprimento de SLA por equipe técnica e tipo de serviço.",
    tipo: "Operacional",
    data: "Disponível após /dashboard/executivo",
  },
  {
    id: "REP-03",
    titulo: "Conversão do Funil de CRM",
    descricao:
      "Taxa de conversão de leads por etapa e ticket médio de negociações.",
    tipo: "Vendas",
    data: "Em construção (módulo CRM ainda em evolução)",
  },
  {
    id: "REP-04",
    titulo: "Curva ABC e Giro de Estoque",
    descricao:
      "Classificação de insumos por valor financeiro investido e frequência de saída.",
    tipo: "Estoque",
    data: "Disponível após /estoque/pecas",
  },
]

export default function RelatoriosPage() {
  const glassStyle = "backdrop-blur-xl bg-white/5 border-white/10 shadow-xl"
  const [toast, setToast] = useState<string | null>(null)

  const showToast = (msg: string) => {
    setToast(msg)
    console.warn('[relatorios]', msg)
    setTimeout(() => setToast(null), 4000)
  }

  return (
    <div className="flex flex-1 flex-col gap-6 p-4 pt-0 md:p-6 md:pt-0">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">
            Central de Relatórios Executivos
          </h1>
          <p className="text-muted-foreground">
            Gere e exporte relatórios consolidados de vendas, finanças, serviços
            e inventário.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            onClick={() =>
              showToast(
                'Filtro de período em construção. Aguardando /relatorios endpoint.',
              )
            }
            className="gap-2 border-white/10"
          >
            <Calendar className="h-4 w-4" />
            <span className="hidden sm:inline">Filtrar Período</span>
          </Button>
          <Button
            onClick={() =>
              showToast(
                'Exportação em massa em construção. Aguardando /relatorios/exportar.',
              )
            }
            className="shrink-0 gap-2 bg-gradient-to-r from-cyan-500 to-blue-600 text-white hover:opacity-90 font-semibold shadow-md shadow-cyan-500/20"
          >
            <Download className="h-4 w-4" />
            <span className="hidden sm:inline">Exportar Tudo (ZIP)</span>
          </Button>
        </div>
      </div>

      {toast && (
        <div
          role="status"
          className="flex items-center gap-2 p-3 rounded-lg border border-amber-500/30 bg-amber-500/10 text-amber-200"
        >
          <Info className="h-4 w-4 shrink-0" />
          <span className="text-sm flex-1">{toast}</span>
          <button
            onClick={() => setToast(null)}
            className="text-amber-200/70 hover:text-amber-200"
            aria-label="Fechar"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
      )}

      <div className="grid gap-6 md:grid-cols-2">
        {relatoriosDisponiveis.map((rel) => (
          <Card
            key={rel.id}
            className={`${glassStyle} group hover:border-cyan-500/30 transition-all`}
          >
            <CardHeader className="flex flex-row items-start justify-between pb-2">
              <div className="space-y-1">
                <span className="text-xs font-semibold text-cyan-400 font-mono">
                  {rel.tipo}
                </span>
                <CardTitle className="text-xl font-bold">{rel.titulo}</CardTitle>
              </div>
              <div className="p-2.5 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
                <BarChart3 className="h-5 w-5" />
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <CardDescription className="text-sm text-slate-300">
                {rel.descricao}
              </CardDescription>
              <div className="flex items-center justify-between pt-3 border-t border-white/10 text-xs text-muted-foreground">
                <span>{rel.data}</span>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() =>
                      showToast(`Exportação Excel #${rel.id} em construção.`)
                    }
                    className="h-8 gap-1.5 text-xs border-white/10 hover:bg-white/10"
                  >
                    <FileSpreadsheet className="h-3.5 w-3.5 text-emerald-400" />
                    Excel
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() =>
                      showToast(`Exportação PDF #${rel.id} em construção.`)
                    }
                    className="h-8 gap-1.5 text-xs border-white/10 hover:bg-white/10"
                  >
                    <Download className="h-3.5 w-3.5 text-cyan-400" />
                    PDF
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <div className="text-xs text-muted-foreground/70 text-center pt-4">
        <Filter className="inline h-3 w-3 mr-1" />
        Sistema de relatórios será ligado aos serviços reais conforme
        endpoints forem expostos pelo backend.
      </div>
    </div>
  )
}
