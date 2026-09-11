"use client"

import { useEffect, useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  ArrowDownRight,
  ArrowUpRight,
  DollarSign,
  PieChart,
  TrendingUp,
  AlertCircle,
} from "lucide-react"
import { financeiroService, type DashboardFinanceiro } from "@/services/financeiro.service"

const fmtBRL = (n: number) =>
  new Intl.NumberFormat('pt-BR', { style: 'currency', currency: 'BRL' }).format(n)

function pct(n: number | undefined): string {
  if (typeof n !== 'number') return '—'
  return `${n >= 0 ? '+' : ''}${n.toFixed(1)}%`
}

export function FinanceKpiCards() {
  const glassStyle = "backdrop-blur-xl bg-white/5 border-white/10 shadow-xl"

  const [data, setData] = useState<DashboardFinanceiro | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelado = false
    async function load() {
      try {
        setLoading(true)
        setError(null)
        const dash = await financeiroService.dashboard()
        if (!cancelado) setData(dash)
      } catch (err) {
        if (!cancelado)
          setError(err instanceof Error ? err.message : 'Erro ao carregar')
      } finally {
        if (!cancelado) setLoading(false)
      }
    }
    load()
    return () => {
      cancelado = true
    }
  }, [])

  if (error && !data) {
    return (
      <div className="p-4 rounded-xl border border-amber-500/30 bg-amber-500/10 text-amber-300 flex items-center gap-3">
        <AlertCircle className="h-5 w-5" />
        <div>
          <p className="font-semibold text-sm">Aguardando /financeiro/dashboard</p>
          <p className="text-xs text-amber-400/80">
            Endpoint ainda não exposto pelo backend.
          </p>
        </div>
      </div>
    )
  }

  const valor = (n: number | undefined) => (loading ? '—' : fmtBRL(n ?? 0))
  const variacao = (n: number | undefined) =>
    loading ? null : pct(n)

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      <Card className={glassStyle}>
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm font-medium">Receita Total</CardTitle>
          <div className="h-8 w-8 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center justify-center">
            <ArrowUpRight className="h-4 w-4" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{valor(data?.recebido_no_mes)}</div>
          <p className="flex items-center gap-1 text-xs text-emerald-400 mt-1 font-medium">
            <TrendingUp className="h-3 w-3" />
            {variacao(data?.variacao_receita_mes_anterior) ?? ' '} em relação ao mês anterior
          </p>
        </CardContent>
      </Card>

      <Card className={glassStyle}>
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm font-medium">Despesas Totais</CardTitle>
          <div className="h-8 w-8 rounded-full bg-rose-500/10 text-rose-400 border border-rose-500/20 flex items-center justify-center">
            <ArrowDownRight className="h-4 w-4" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{valor(data?.despesa_no_mes)}</div>
          <p className="flex items-center gap-1 text-xs text-rose-400 mt-1 font-medium">
            <TrendingUp className="h-3 w-3" />
            {variacao(data?.variacao_despesa_mes_anterior) ?? ' '} em relação ao mês anterior
          </p>
        </CardContent>
      </Card>

      <Card className={glassStyle}>
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm font-medium">Saldo</CardTitle>
          <div className="h-8 w-8 rounded-full bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 flex items-center justify-center">
            <DollarSign className="h-4 w-4" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{valor(data?.saldo)}</div>
          <p className="text-xs text-muted-foreground mt-1">
            A pagar: {loading ? '—' : fmtBRL(data?.a_pagar ?? 0)}
          </p>
        </CardContent>
      </Card>

      <Card className={glassStyle}>
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm font-medium">Inadimplência</CardTitle>
          <div className="h-8 w-8 rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/20 flex items-center justify-center">
            <PieChart className="h-4 w-4" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {loading ? '—' : `${(data?.inadimplencia ?? 0).toFixed(1)}%`}
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            A receber: {loading ? '—' : fmtBRL(data?.a_receber ?? 0)}
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
