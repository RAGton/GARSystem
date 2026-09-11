"use client"

import { useEffect, useState } from "react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  AlertCircle,
  AlertTriangle,
  Info,
  ArrowRight,
  CheckCircle2,
} from "lucide-react"
import { dashboardService, type DashboardAlerta } from "@/services/dashboard.service"

function iconFor(sev: DashboardAlerta['severidade']) {
  if (sev === 'critical') return AlertCircle
  if (sev === 'warning') return AlertTriangle
  return Info
}

function colorFor(sev: DashboardAlerta['severidade']) {
  if (sev === 'critical') return 'bg-rose-500/20 text-rose-400'
  if (sev === 'warning') return 'bg-amber-500/20 text-amber-400'
  return 'bg-blue-500/20 text-blue-400'
}

function timeAgo(iso: string): string {
  try {
    const d = new Date(iso)
    const diff = (Date.now() - d.getTime()) / 1000
    if (diff < 60) return 'agora mesmo'
    if (diff < 3600) return `há ${Math.floor(diff / 60)} min`
    if (diff < 86400) return `há ${Math.floor(diff / 3600)} h`
    return d.toLocaleDateString('pt-BR')
  } catch {
    return ''
  }
}

export function AlertsSection() {
  const glassStyle =
    "backdrop-blur-xl bg-white/5 border-white/10 shadow-xl h-full flex flex-col"

  const [alerts, setAlerts] = useState<DashboardAlerta[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelado = false
    async function load() {
      try {
        setLoading(true)
        setError(null)
        // Carrega TODOS os alertas (sem filtro de severidade); mantém limite razoável.
        const data = await dashboardService.alertas()
        if (!cancelado) setAlerts(data)
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

  // Mostra os 5 mais críticos primeiro.
  const ordenados = [...alerts].sort((a, b) => {
    const ordem = { critical: 0, warning: 1, info: 2 } as const
    return (ordem[a.severidade] ?? 3) - (ordem[b.severidade] ?? 3)
  }).slice(0, 5)

  return (
    <Card className={glassStyle}>
      <CardHeader>
        <CardTitle>Alertas e Ações</CardTitle>
        <CardDescription>
          Itens que requerem atenção imediata
        </CardDescription>
      </CardHeader>
      <CardContent className="flex-1 flex flex-col gap-3">
        {loading && (
          <div className="space-y-3">
            {[1, 2, 3].map((i) => (
              <div
                key={i}
                className="h-16 rounded-lg bg-white/5 animate-pulse"
              />
            ))}
          </div>
        )}

        {!loading && error && (
          <div className="p-3 rounded-lg border border-amber-500/30 bg-amber-500/10 text-amber-300 flex items-center gap-2 text-sm">
            <AlertTriangle className="h-4 w-4" />
            <div>
              <p className="font-semibold">Endpoint indisponível</p>
              <p className="text-xs text-amber-300/80">
                Aguardando <code>/alertas</code> no backend.
              </p>
            </div>
          </div>
        )}

        {!loading && !error && ordenados.length === 0 && (
          <div className="flex flex-1 flex-col items-center justify-center text-center py-8 text-muted-foreground">
            <CheckCircle2 className="h-10 w-10 text-emerald-400/70 mb-2" />
            <p className="text-sm font-medium">Nenhum alerta ativo.</p>
            <p className="text-xs">Tudo sob controle.</p>
          </div>
        )}

        {!loading && !error && ordenados.length > 0 && (
          <div className="flex flex-col gap-3">
            {ordenados.map((alert) => {
              const Icon = iconFor(alert.severidade)
              return (
                <div
                  key={alert.id}
                  className="flex items-start gap-3 rounded-lg border border-white/5 p-3 transition-colors hover:bg-white/10"
                >
                  <div
                    className={`mt-0.5 rounded-full p-1.5 shadow-sm ${colorFor(alert.severidade)}`}
                  >
                    <Icon className="h-4 w-4" />
                  </div>
                  <div className="flex-1 space-y-1 min-w-0">
                    <p className="text-sm font-medium leading-none text-foreground truncate">
                      {alert.mensagem}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {alert.entidade} · {timeAgo(alert.created_at)}
                    </p>
                  </div>
                  <button
                    type="button"
                    aria-label="Ver detalhes"
                    className="flex h-8 w-8 items-center justify-center rounded-full hover:bg-white/10 shrink-0 text-muted-foreground hover:text-foreground transition-colors"
                  >
                    <ArrowRight className="h-4 w-4" />
                  </button>
                </div>
              )
            })}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
