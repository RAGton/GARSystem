"use client"

import { useState } from "react"
import { FinanceKpiCards } from "@/components/financeiro/finance-kpi-cards"
import { CashFlowChart } from "@/components/financeiro/cash-flow-chart"
import { ProfitChart } from "@/components/financeiro/profit-chart"
import { Button } from "@/components/ui/button"
import {
  Download,
  Filter,
  Plus,
  Info,
  X,
} from "lucide-react"

export default function FinanceiroPage() {
  const [toast, setToast] = useState<{
    type: 'info' | 'warn'
    msg: string
  } | null>(null)

  const showToast = (msg: string) => {
    setToast({ type: 'info', msg })
    setTimeout(() => setToast(null), 4000)
  }

  return (
    <div className="flex flex-1 flex-col gap-6 p-4 pt-0 md:p-6 md:pt-0">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">
            Gestão Financeira
          </h1>
          <p className="text-muted-foreground">
            Monitore fluxo de caixa, receitas, despesas e faturamento em tempo
            real.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            onClick={() =>
              showToast(
                'Exportação de DRE em construção. Aguardando /financeiro/dre no backend.',
              )
            }
            className="gap-2 border-white/10"
          >
            <Download className="h-4 w-4" />
            <span className="hidden sm:inline">Exportar DRE</span>
          </Button>
          <Button
            variant="outline"
            size="icon"
            onClick={() =>
              showToast(
                'Filtro avançado em construção. Use o intervalo acima dos gráficos.',
              )
            }
            className="shrink-0 border-white/10"
          >
            <Filter className="h-4 w-4" />
          </Button>
          <Button
            onClick={() =>
              showToast(
                'Lançamento manual em construção. Aguardando POST /financeiro/lancamentos no backend.',
              )
            }
            className="shrink-0 gap-2 bg-gradient-to-r from-cyan-500 to-blue-600 text-white hover:opacity-90 font-semibold shadow-md shadow-cyan-500/20"
          >
            <Plus className="h-4 w-4" />
            <span className="hidden sm:inline">Novo Lançamento</span>
          </Button>
        </div>
      </div>

      {/* Toast inline (banner) — shadcn toast não está instalado. */}
      {toast && (
        <div
          role="status"
          className="flex items-center gap-2 p-3 rounded-lg border border-amber-500/30 bg-amber-500/10 text-amber-200"
        >
          <Info className="h-4 w-4 shrink-0" />
          <span className="text-sm flex-1">{toast.msg}</span>
          <button
            onClick={() => setToast(null)}
            className="text-amber-200/70 hover:text-amber-200"
            aria-label="Fechar"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
      )}

      <FinanceKpiCards />

      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-7">
        <div className="lg:col-span-4">
          <CashFlowChart />
        </div>
        <div className="lg:col-span-3">
          <ProfitChart />
        </div>
      </div>
    </div>
  )
}
