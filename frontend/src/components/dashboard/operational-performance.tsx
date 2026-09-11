"use client"

import { useEffect, useState } from "react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { AlertTriangle, TrendingDown, TrendingUp } from "lucide-react"
import {
  dashboardService,
  type DashboardExecutivo,
} from "@/services/dashboard.service"

function fmtHoras(h: number): string {
  if (h < 1) return `${Math.round(h * 60)} min`
  if (h < 24) return `${h.toFixed(1)} h`
  return `${(h / 24).toFixed(1)} dia(s)`
}

export function OperationalPerformance() {
  const glassStyle =
    "backdrop-blur-xl bg-white/5 border-white/10 shadow-xl h-full flex flex-col"

  const [data, setData] = useState<DashboardExecutivo | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelado = false
    async function load() {
      try {
        setLoading(true)
        setError(null)
        const dash = await dashboardService.dashboardExecutivo()
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

  const tempoMedio = data?.tempo_medio_resolucao_horas ?? null
  const topClientes = data?.top_clientes ?? []
  const tecnicos = data?.tecnicos ?? []

  // Calcula ocupação média dos técnicos
  const ocupMedia =
    tecnicos.length > 0
      ? tecnicos.reduce((acc, t) => acc + (t.ocupacao_percent || 0), 0) /
        tecnicos.length
      : null

  return (
    <Card className={glassStyle}>
      <CardHeader>
        <CardTitle>Performance da Equipe</CardTitle>
        <CardDescription>
          {loading
            ? 'Carregando...'
            : data
              ? 'Dados agregados do /dashboard/executivo'
              : 'Aguardando endpoint /dashboard/executivo'}
        </CardDescription>
      </CardHeader>
      <CardContent className="flex-1 flex flex-col gap-4">
        {error && (
          <div className="p-3 rounded-lg border border-amber-500/30 bg-amber-500/10 text-amber-300 flex items-center gap-2 text-sm">
            <AlertTriangle className="h-4 w-4" />
            {error}
          </div>
        )}

        <div className="grid gap-4 md:grid-cols-2">
          <Metric
            label="Tempo Médio de Reparo (MTTR)"
            value={tempoMedio !== null ? fmtHoras(tempoMedio) : '—'}
            sub={
              tempoMedio === null
                ? 'Indisponível'
                : 'Calculado pelo backend'
            }
            color="emerald"
            percent={
              tempoMedio === null
                ? 0
                : Math.max(10, Math.min(100, 100 - tempoMedio * 4))
            }
          />
          <Metric
            label="OS Concluídas no Mês"
            value={
              data?.os_finalizadas_mes !== undefined
                ? `${data.os_finalizadas_mes}`
                : '—'
            }
            sub={
              data
                ? `${data.os_canceladas_mes ?? 0} canceladas`
                : 'Indisponível'
            }
            color="emerald"
            percent={
              data?.os_finalizadas_mes
                ? Math.min(
                    100,
                    Math.round(
                      (data.os_finalizadas_mes /
                        Math.max(1, data.os_finalizadas_mes + (data.os_canceladas_mes ?? 0))) *
                        100,
                    ),
                  )
                : 0
            }
          />
          <Metric
            label="Ocupação dos Técnicos"
            value={
              ocupMedia !== null ? `${ocupMedia.toFixed(0)}%` : '—'
            }
            sub={
              ocupMedia === null
                ? 'Indisponível'
                : ocupMedia > 85
                  ? 'Carga alta'
                  : 'Saudável'
            }
            color={ocupMedia !== null && ocupMedia > 85 ? 'amber' : 'cyan'}
            percent={ocupMedia ?? 0}
            icon={ocupMedia !== null && ocupMedia > 85 ? TrendingUp : TrendingDown}
          />
          <Metric
            label="SLA Cumprimento"
            value={
              typeof data?.sla_cumprimento_percent === 'number'
                ? `${data.sla_cumprimento_percent.toFixed(0)}%`
                : '—'
            }
            sub={
              typeof data?.sla_cumprimento_percent === 'number'
                ? 'No mês'
                : 'Indisponível'
            }
            color="purple"
            percent={data?.sla_cumprimento_percent ?? 0}
          />
        </div>

        {/* Top clientes (dados reais quando back expor) */}
        <div className="mt-2 pt-3 border-t border-white/5">
          <p className="text-xs font-semibold text-muted-foreground mb-2 uppercase tracking-wider">
            Top clientes
          </p>
          {topClientes.length === 0 ? (
            <p className="text-xs text-muted-foreground/70">
              {loading ? 'Carregando...' : 'Sem dados disponíveis.'}
            </p>
          ) : (
            <ul className="space-y-1.5">
              {topClientes.slice(0, 3).map((c) => (
                <li
                  key={c.cliente_id}
                  className="flex items-center justify-between text-sm"
                >
                  <span className="text-foreground truncate">{c.cliente_nome}</span>
                  <span className="text-emerald-400 font-semibold">
                    {new Intl.NumberFormat('pt-BR', {
                      style: 'currency',
                      currency: 'BRL',
                    }).format(c.total_gasto)}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

function Metric({
  label,
  value,
  sub,
  percent,
  color,
  icon: Icon,
}: {
  label: string
  value: string
  sub?: string
  percent: number
  color: 'emerald' | 'amber' | 'cyan' | 'purple'
  icon?: React.ComponentType<{ className?: string }>
}) {
  const barColor: Record<typeof color, string> = {
    emerald: 'bg-emerald-400',
    amber: 'bg-amber-400',
    cyan: 'bg-cyan-400',
    purple: 'bg-purple-400',
  }
  return (
    <div className="flex flex-col gap-2 rounded-lg border border-white/5 p-3 bg-black/10">
      <div className="text-xs font-medium text-muted-foreground">{label}</div>
      <div className="flex items-end gap-2">
        <span className="text-xl font-bold text-foreground">{value}</span>
        {Icon && <Icon className="h-4 w-4 text-amber-400 mb-1" />}
      </div>
      <div className="h-1.5 w-full bg-white/10 rounded-full overflow-hidden">
        <div
          className={`h-full ${barColor[color]}`}
          style={{ width: `${Math.max(0, Math.min(100, percent))}%` }}
        />
      </div>
      {sub && <p className="text-xs text-muted-foreground/80">{sub}</p>}
    </div>
  )
}
