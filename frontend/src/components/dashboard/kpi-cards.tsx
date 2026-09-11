"use client"

import { useEffect, useState } from "react"
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  DollarSign,
  FileText,
  CheckCircle2,
  Users,
  AlertTriangle,
  TrendingUp,
  TrendingDown,
} from "lucide-react"
import { dashboardService, type DashboardExecutivo } from "@/services/dashboard.service"
import { clientesService } from "@/services/clientes.service"
import { osService } from "@/services/os.service"

interface KpiData {
  receita: number
  clientesTotal: number
  osAbertas: number
  slaCumprimento: number | null
  variacaoReceita: number | null
}

const fmtBRL = (n: number) =>
  new Intl.NumberFormat('pt-BR', { style: 'currency', currency: 'BRL' }).format(n)

export function KpiCards() {
  const [data, setData] = useState<KpiData | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [source, setSource] = useState<'dashboard' | 'agregado'>('dashboard')

  const glassStyle =
    "backdrop-blur-xl bg-white/5 border-white/10 shadow-xl hover:bg-white/10 transition-colors"

  useEffect(() => {
    let cancelado = false
    async function load() {
      try {
        setLoading(true)
        setError(null)

        // 1) Tenta /dashboard/executivo (agregado oficial)
        const dash = await dashboardService.dashboardExecutivo()
        if (cancelado) return

        if (dash) {
          setSource('dashboard')
          setData({
            receita: dash.receita_mes ?? 0,
            clientesTotal: dash.clientes_total ?? 0,
            osAbertas: (dash.os_abertas ?? 0) + (dash.os_em_andamento ?? 0),
            slaCumprimento:
              typeof dash.sla_cumprimento_percent === 'number'
                ? dash.sla_cumprimento_percent
                : null,
            variacaoReceita:
              typeof dash.variacao_receita_mes_anterior_percent === 'number'
                ? dash.variacao_receita_mes_anterior_percent
                : null,
          })
          return
        }

        // 2) Fallback: agrega de /ordens + /clientes.
        const [osList, clientesList] = await Promise.all([
          osService.listar(),
          clientesService.listar(),
        ])
        if (cancelado) return

        const emAberto = osList.filter(
          (o) => o.status !== 'Finalizada' && o.status !== 'Cancelada',
        ).length

        setSource('agregado')
        setData({
          receita: 0,
          clientesTotal: clientesList.length,
          osAbertas: emAberto,
          slaCumprimento: null,
          variacaoReceita: null,
        })
      } catch (err) {
        if (!cancelado) {
          setError(err instanceof Error ? err.message : 'Erro ao carregar')
        }
      } finally {
        if (!cancelado) setLoading(false)
      }
    }
    load()
    return () => {
      cancelado = true
    }
  }, [])

  if (loading) {
    return (
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {[1, 2, 3, 4].map((i) => (
          <Card
            key={i}
            className={`${glassStyle} animate-pulse h-28 flex flex-col justify-between p-4`}
          >
            <div className="h-4 bg-white/10 rounded w-1/2" />
            <div className="h-8 bg-white/20 rounded w-3/4" />
          </Card>
        ))}
      </div>
    )
  }

  if (error && !data) {
    return (
      <div className="p-4 rounded-xl border border-amber-500/30 bg-amber-500/10 text-amber-300 flex items-center gap-3">
        <AlertTriangle className="h-5 w-5 shrink-0 text-amber-400" />
        <div>
          <p className="font-semibold text-sm">Aguardando sincronização</p>
          <p className="text-xs text-amber-400/80">{error}</p>
        </div>
      </div>
    )
  }

  const Variacao = ({ v }: { v: number | null }) => {
    if (v === null) return null
    const positive = v >= 0
    const Icon = positive ? TrendingUp : TrendingDown
    return (
      <p
        className={`text-xs mt-1 font-medium flex items-center gap-1 ${
          positive ? 'text-emerald-400' : 'text-rose-400'
        }`}
      >
        <Icon className="h-3 w-3" />
        {positive ? '+' : ''}
        {v.toFixed(1)}% vs mês anterior
      </p>
    )
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      <Card className={glassStyle}>
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            Receita do Mês
          </CardTitle>
          <DollarSign className="h-4 w-4 text-emerald-400" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-foreground">
            {fmtBRL(data?.receita ?? 0)}
          </div>
          <Variacao v={data?.variacaoReceita ?? null} />
          {source === 'agregado' && (
            <p className="text-xs text-muted-foreground mt-1">
              Aguardando /dashboard/executivo
            </p>
          )}
        </CardContent>
      </Card>

      <Card className={glassStyle}>
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            Clientes Cadastrados
          </CardTitle>
          <Users className="h-4 w-4 text-blue-400" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-foreground">
            {data?.clientesTotal ?? 0}
          </div>
          <p className="text-xs text-blue-400 mt-1 font-medium">Base ativa</p>
        </CardContent>
      </Card>

      <Card className={glassStyle}>
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            OS Em Aberto
          </CardTitle>
          <FileText className="h-4 w-4 text-amber-400" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-foreground">
            {data?.osAbertas ?? 0}
          </div>
          <p className="text-xs text-amber-400 mt-1 font-medium">
            Requer atendimento
          </p>
        </CardContent>
      </Card>

      <Card className={glassStyle}>
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            SLA de Atendimento
          </CardTitle>
          <CheckCircle2 className="h-4 w-4 text-purple-400" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-foreground">
            {data?.slaCumprimento === null
              ? '—'
              : `${data?.slaCumprimento?.toFixed(0)}%`}
          </div>
          <p className="text-xs text-purple-400 mt-1 font-medium">
            {data?.slaCumprimento === null
              ? 'Disponível após /dashboard/executivo'
              : 'Cumprimento no mês'}
          </p>
        </CardContent>
      </Card>
    </div>
  )
}

// Re-export para conveniência de tipos.
export type { DashboardExecutivo }
