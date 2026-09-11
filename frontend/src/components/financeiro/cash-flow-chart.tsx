"use client"

import { useEffect, useMemo, useState } from "react"
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts"
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card"
import { AlertCircle } from "lucide-react"
import { financeiroService } from "@/services/financeiro.service"

interface PontoMes {
  name: string
  receita: number
  despesa: number
}

function isoMonthsAgo(n: number): string {
  const d = new Date()
  d.setMonth(d.getMonth() - n)
  return d.toISOString().slice(0, 10)
}
function isoToday(): string {
  return new Date().toISOString().slice(0, 10)
}
function isoMonthLabel(ym: string): string {
  const [, m, y] = ym.split('-')
  const meses = ['Jan', 'Fev', 'Mar', 'Abr', 'Mai', 'Jun', 'Jul', 'Ago', 'Set', 'Out', 'Nov', 'Dez']
  const idx = parseInt(m, 10) - 1
  return `${meses[idx] ?? m}/${y.slice(2)}`
}

export function CashFlowChart() {
  const glassStyle =
    "backdrop-blur-xl bg-white/5 border-white/10 shadow-xl h-full flex flex-col"

  const [data, setData] = useState<PontoMes[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelado = false
    async function load() {
      try {
        setLoading(true)
        setError(null)
        const de = isoMonthsAgo(6)
        const ate = isoToday()
        const fluxo = await financeiroService.fluxoCaixa(de, ate)
        if (cancelado) return

        type FluxoPonto = { data: string; receitas: number; despesas: number }
        type FluxoRes =
          | { pontos?: FluxoPonto[]; items?: FluxoPonto[] }
          | FluxoPonto[]
          | null

        const raw: FluxoRes = fluxo as FluxoRes
        let pontos: FluxoPonto[] = []
        if (Array.isArray(raw)) pontos = raw
        else if (raw && Array.isArray((raw as { pontos?: FluxoPonto[] }).pontos))
          pontos = (raw as { pontos: FluxoPonto[] }).pontos
        else if (raw && Array.isArray((raw as { items?: FluxoPonto[] }).items))
          pontos = (raw as { items: FluxoPonto[] }).items

        const acc = new Map<string, PontoMes>()
        for (const p of pontos) {
          const ym = p.data?.slice(0, 7)
          if (!ym) continue
          const cur = acc.get(ym) ?? {
            name: isoMonthLabel(`${ym}-01`),
            receita: 0,
            despesa: 0,
          }
          cur.receita += Number(p.receitas) || 0
          cur.despesa += Number(p.despesas) || 0
          acc.set(ym, cur)
        }
        const sorted = Array.from(acc.entries())
          .sort(([a], [b]) => a.localeCompare(b))
          .map(([, v]) => v)

        setData(sorted)
      } catch (err) {
        if (!cancelado)
          setError(err instanceof Error ? err.message : 'Erro')
      } finally {
        if (!cancelado) setLoading(false)
      }
    }
    load()
    return () => {
      cancelado = true
    }
  }, [])

  const empty = useMemo(
    () => !loading && data.length === 0,
    [loading, data.length],
  )

  return (
    <Card className={glassStyle}>
      <CardHeader>
        <CardTitle>Visão Geral do Fluxo de Caixa</CardTitle>
        <CardDescription>
          Comparativo mensal de receitas vs despesas
        </CardDescription>
      </CardHeader>
      <CardContent className="flex-1 pb-4 min-h-[300px] flex flex-col">
        {error && (
          <div className="p-3 rounded-lg border border-amber-500/30 bg-amber-500/10 text-amber-300 flex items-center gap-2 text-sm mb-3">
            <AlertCircle className="h-4 w-4" />
            {error}
          </div>
        )}
        {empty ? (
          <div className="flex flex-1 items-center justify-center text-sm text-muted-foreground text-center px-6">
            Sem dados no período. Aguardando
            <code className="mx-1 px-1.5 py-0.5 rounded bg-white/5 font-mono text-xs">
              /financeiro/fluxo-caixa
            </code>
            .
          </div>
        ) : (
          <div className="flex-1 min-h-[280px]">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart
                data={data}
                margin={{ top: 10, right: 10, left: -20, bottom: 0 }}
              >
                <CartesianGrid
                  strokeDasharray="3 3"
                  vertical={false}
                  stroke="rgba(255,255,255,0.1)"
                />
                <XAxis
                  dataKey="name"
                  stroke="#94A3B8"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                />
                <YAxis
                  stroke="#94A3B8"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                  tickFormatter={(v: number) =>
                    v >= 1000 ? `R$${(v / 1000).toFixed(0)}k` : `R$${v}`
                  }
                />
                <Tooltip
                  cursor={{ fill: 'rgba(255,255,255,0.05)' }}
                  contentStyle={{
                    borderRadius: '12px',
                    border: '1px solid rgba(255,255,255,0.15)',
                    backgroundColor: '#0F172A',
                    color: '#fff',
                  }}
                  formatter={(v) =>
                    new Intl.NumberFormat('pt-BR', {
                      style: 'currency',
                      currency: 'BRL',
                    }).format(Number(v) || 0)
                  }
                />
                <Legend wrapperStyle={{ fontSize: '12px' }} />
                <Bar
                  dataKey="receita"
                  name="Receita"
                  fill="#10B981"
                  radius={[4, 4, 0, 0]}
                  maxBarSize={40}
                />
                <Bar
                  dataKey="despesa"
                  name="Despesa"
                  fill="#F43F5E"
                  radius={[4, 4, 0, 0]}
                  maxBarSize={40}
                />
              </BarChart>
            </ResponsiveContainer>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
