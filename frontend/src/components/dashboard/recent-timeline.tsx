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
  CheckCircle2,
  MessageSquare,
  Wrench,
  FileText,
  Clock,
} from "lucide-react"
import { osService, type OrdemServico } from "@/services/os.service"

type Evento = {
  id: string
  title: string
  time: string
  user: string
  icon: React.ComponentType<{ className?: string }>
  color: string
}

function timeAgo(iso: string): string {
  if (!iso) return ''
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

function eventoParaItem(os: OrdemServico): Evento {
  const status = os.status
  let icon = Wrench
  let color = 'text-cyan-400'
  let title = `OS #${os.id} — ${os.cliente}`

  if (status === 'Finalizada') {
    icon = CheckCircle2
    color = 'text-emerald-400'
    title = `OS #${os.id} concluída — ${os.cliente}`
  } else if (status === 'Cancelada') {
    icon = Clock
    color = 'text-rose-400'
    title = `OS #${os.id} cancelada — ${os.cliente}`
  } else if (status === 'Aprovada' || status === 'Orcamento') {
    icon = FileText
    color = 'text-blue-400'
    title = `OS #${os.id} em orçamento — ${os.cliente}`
  } else if (status === 'AguardandoPeca') {
    icon = MessageSquare
    color = 'text-amber-400'
    title = `OS #${os.id} aguardando peça — ${os.cliente}`
  } else if (status === 'EmAndamento') {
    icon = Wrench
    color = 'text-cyan-400'
    title = `OS #${os.id} em andamento — ${os.cliente}`
  } else if (status === 'Aberta') {
    icon = FileText
    color = 'text-amber-400'
    title = `OS #${os.id} aberta — ${os.cliente}`
  }

  return {
    id: String(os.id),
    title,
    time: timeAgo(os.horario_abertura),
    user: os.atendente || os.nome_tecnico_responsavel || 'Sistema',
    icon,
    color,
  }
}

export function RecentTimeline() {
  const glassStyle =
    "backdrop-blur-xl bg-white/5 border-white/10 shadow-xl h-full flex flex-col"

  const [items, setItems] = useState<Evento[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelado = false
    async function load() {
      try {
        setLoading(true)
        setError(null)
        // Sem endpoint de feed geral; usa /ordens ordenado por horario_abertura.
        const list = await osService.listar()
        if (cancelado) return
        const ordenadas = [...list]
          .sort(
            (a, b) =>
              new Date(b.horario_abertura).getTime() -
              new Date(a.horario_abertura).getTime(),
          )
          .slice(0, 8)
        setItems(ordenadas.map(eventoParaItem))
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

  return (
    <Card className={glassStyle}>
      <CardHeader>
        <CardTitle>Atividades Recentes</CardTitle>
        <CardDescription>
          Últimas movimentações de Ordens de Serviço
        </CardDescription>
      </CardHeader>
      <CardContent className="flex-1">
        {loading && (
          <div className="space-y-4">
            {[1, 2, 3, 4].map((i) => (
              <div key={i} className="flex gap-3">
                <div className="h-6 w-6 rounded-full bg-white/5 animate-pulse" />
                <div className="flex-1 space-y-2">
                  <div className="h-3 bg-white/10 rounded w-3/4 animate-pulse" />
                  <div className="h-2 bg-white/5 rounded w-1/2 animate-pulse" />
                </div>
              </div>
            ))}
          </div>
        )}

        {!loading && error && (
          <div className="p-3 rounded-lg border border-amber-500/30 bg-amber-500/10 text-amber-300 text-sm">
            {error}
          </div>
        )}

        {!loading && !error && items.length === 0 && (
          <div className="flex flex-col items-center justify-center py-8 text-center text-muted-foreground">
            <Clock className="h-10 w-10 opacity-50 mb-2" />
            <p className="text-sm font-medium">Nenhuma atividade recente.</p>
            <p className="text-xs">Crie uma OS ou orçamento para começar.</p>
          </div>
        )}

        {!loading && !error && items.length > 0 && (
          <div className="space-y-5">
            {items.map((item, index) => {
              const Icon = item.icon
              return (
                <div key={item.id} className="relative flex gap-3">
                  {index !== items.length - 1 && (
                    <div className="absolute left-[11px] top-6 h-full w-[2px] bg-white/10" />
                  )}
                  <div className="relative mt-1 flex h-6 w-6 flex-none items-center justify-center rounded-full bg-black/20 shadow-sm ring-1 ring-white/10">
                    <Icon className={`h-3.5 w-3.5 ${item.color}`} />
                  </div>
                  <div className="flex flex-col gap-0.5 min-w-0">
                    <p className="text-sm font-medium leading-none text-foreground">
                      {item.title}
                    </p>
                    <div className="flex items-center gap-2 text-xs text-muted-foreground mt-1">
                      <span className="truncate">{item.user}</span>
                      <span>•</span>
                      <span>{item.time}</span>
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
