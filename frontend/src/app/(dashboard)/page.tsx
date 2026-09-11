'use client'

import { KpiCards } from "@/components/dashboard/kpi-cards"
import { AlertsSection } from "@/components/dashboard/alerts-section"
import { TrendsChart } from "@/components/dashboard/trends-chart"
import { OperationalPerformance } from "@/components/dashboard/operational-performance"
import { RecentTimeline } from "@/components/dashboard/recent-timeline"
import { useAuthStore } from "@/stores/auth.store"
import Link from "next/link"
import {
  ArrowRight,
  CheckCircle2,
  AlertTriangle,
  ListChecks,
  Wrench,
  Users,
  FileText,
  Package,
  CircleDollarSign,
} from "lucide-react"

export default function DashboardPage() {
  const user = useAuthStore(state => state.user)

  const firstName =
    user?.username?.split('.')[0]?.replace(/\b\w/g, (c) => c.toUpperCase()) ??
    (user?.papel === 'Administrador' ? 'Administrador' : 'Gabriel')

  return (
    <div className="flex flex-1 flex-col gap-8 p-4 pt-0 md:p-6 md:pt-0">
      {/* Hero */}
      <div className="flex flex-col gap-2 py-4 md:py-6">
        <h1 className="text-4xl md:text-5xl font-extrabold tracking-tight">
          Bom dia, {firstName}{' '}
          <span className="inline-block animate-wave origin-bottom-right">👋</span>
        </h1>
        <p className="text-muted-foreground text-lg">
          Hoje é{' '}
          {new Intl.DateTimeFormat('pt-BR', {
            weekday: 'long',
            day: '2-digit',
            month: 'long',
          }).format(new Date())}
          .
        </p>
      </div>

      {/* Bloco 1 — O que aconteceu */}
      <section aria-labelledby="o-que-aconteceu" className="flex flex-col gap-4">
        <header className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-cyan-500/10 border border-cyan-500/30 text-cyan-400">
            <ListChecks className="h-5 w-5" />
          </div>
          <h2
            id="o-que-aconteceu"
            className="text-2xl font-bold tracking-tight"
          >
            O que aconteceu
          </h2>
        </header>
        <KpiCards />
        <div className="grid gap-6 md:grid-cols-2">
          <TrendsChart />
          <OperationalPerformance />
        </div>
      </section>

      {/* Bloco 2 — O que exige atenção */}
      <section aria-labelledby="o-que-exige-atencao" className="flex flex-col gap-4">
        <header className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-400">
            <AlertTriangle className="h-5 w-5" />
          </div>
          <h2
            id="o-que-exige-atencao"
            className="text-2xl font-bold tracking-tight"
          >
            O que exige atenção
          </h2>
        </header>
        <AlertsSection />
      </section>

      {/* Bloco 3 — O que faço agora */}
      <section aria-labelledby="o-que-faco-agora" className="flex flex-col gap-4">
        <header className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-emerald-500/10 border border-emerald-500/30 text-emerald-400">
            <CheckCircle2 className="h-5 w-5" />
          </div>
          <h2
            id="o-que-faco-agora"
            className="text-2xl font-bold tracking-tight"
          >
            O que faço agora
          </h2>
        </header>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <ActionCard
            href="/os"
            icon={Wrench}
            title="Abrir nova OS"
            description="Crie uma nova ordem de serviço"
          />
          <ActionCard
            href="/orcamentos"
            icon={FileText}
            title="Novo orçamento"
            description="Gere orçamento para um cliente"
          />
          <ActionCard
            href="/clientes"
            icon={Users}
            title="Gerenciar clientes"
            description="Lista, busca e resumo financeiro"
          />
          <ActionCard
            href="/estoque"
            icon={Package}
            title="Estoque crítico"
            description="Veja peças abaixo do mínimo"
          />
          <ActionCard
            href="/financeiro"
            icon={CircleDollarSign}
            title="Lançamentos"
            description="Contas a pagar e receber"
          />
        </div>
      </section>

      {/* Atividades recentes (linha inferior) */}
      <section aria-labelledby="atividades-recentes" className="flex flex-col gap-4">
        <header className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-blue-500/10 border border-blue-500/30 text-blue-400">
            <ListChecks className="h-5 w-5" />
          </div>
          <h2
            id="atividades-recentes"
            className="text-2xl font-bold tracking-tight"
          >
            Atividades recentes
          </h2>
        </header>
        <RecentTimeline />
      </section>
    </div>
  )
}

function ActionCard({
  href,
  icon: Icon,
  title,
  description,
}: {
  href: string
  icon: React.ComponentType<{ className?: string }>
  title: string
  description: string
}) {
  return (
    <Link
      href={href}
      className="group flex items-center gap-3 rounded-xl border border-white/10 bg-white/5 backdrop-blur-md p-4 transition-all hover:bg-white/10 hover:border-cyan-500/30"
    >
      <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-cyan-500/10 border border-cyan-500/30 text-cyan-400">
        <Icon className="h-5 w-5" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="text-sm font-semibold text-foreground">{title}</div>
        <div className="text-xs text-muted-foreground">{description}</div>
      </div>
      <ArrowRight className="h-4 w-4 text-muted-foreground group-hover:text-cyan-400 group-hover:translate-x-1 transition-all" />
    </Link>
  )
}
