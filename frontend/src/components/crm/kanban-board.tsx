"use client"

import { useEffect, useMemo, useState } from "react"
import { Card, CardContent } from "@/components/ui/card"
import {
  AlertCircle,
  ChevronLeft,
  ChevronRight,
  Mail,
  Phone,
  Wallet,
  Users,
} from "lucide-react"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { clientesService, type Cliente } from "@/services/clientes.service"

const PAGE_SIZE = 10

export function KanbanBoard() {
  // Este componente foi REESCRITO: o antigo chamava `/crm/deals` (que NÃO existe).
  // Agora é uma listagem de clientes consumindo `GET /clientes` + `GET /clientes/{id}/resumo`.

  const [busca, setBusca] = useState('')
  const [clientes, setClientes] = useState<Cliente[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [page, setPage] = useState(1)

  useEffect(() => {
    let cancelado = false
    async function load() {
      try {
        setLoading(true)
        const data = await clientesService.listar()
        if (!cancelado) setClientes(data)
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

  const filtrados = useMemo(() => {
    const q = busca.toLowerCase().trim()
    if (!q) return clientes
    return clientes.filter(
      (c) =>
        c.nome.toLowerCase().includes(q) ||
        c.email?.toLowerCase().includes(q) ||
        c.telefone?.toLowerCase().includes(q) ||
        c.cpf_cnpj?.toLowerCase().includes(q),
    )
  }, [clientes, busca])

  const totalPaginas = Math.max(1, Math.ceil(filtrados.length / PAGE_SIZE))
  const paginaAtual = Math.min(page, totalPaginas)
  const slice = useMemo(
    () =>
      filtrados.slice((paginaAtual - 1) * PAGE_SIZE, paginaAtual * PAGE_SIZE),
    [filtrados, paginaAtual],
  )

  const fmtBRL = (n: number) =>
    new Intl.NumberFormat('pt-BR', { style: 'currency', currency: 'BRL' }).format(n)

  if (error && !clientes.length) {
    return (
      <div className="flex flex-1 items-center justify-center p-6">
        <div className="max-w-md rounded-xl border border-amber-500/30 bg-amber-500/10 p-6 text-amber-200">
          <AlertCircle className="h-5 w-5 mb-2" />
          <p className="font-semibold">Não foi possível carregar clientes</p>
          <p className="text-sm text-amber-300/80 mt-1">{error}</p>
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-1 flex-col gap-4 min-h-0">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Users className="h-4 w-4 text-cyan-400" />
          {loading ? (
            'Carregando...'
          ) : (
            <>
              <span className="font-semibold text-foreground">
                {filtrados.length}
              </span>{' '}
              cliente{filtrados.length === 1 ? '' : 's'} cadastrado
              {filtrados.length === 1 ? '' : 's'}
            </>
          )}
        </div>
        <div className="relative w-full sm:w-[280px]">
          <Input
            type="search"
            placeholder="Buscar por nome, email, telefone ou CPF/CNPJ..."
            value={busca}
            onChange={(e) => {
              setBusca(e.target.value)
              setPage(1)
            }}
            className="bg-background"
          />
        </div>
      </div>

      <div className="flex-1 overflow-auto rounded-xl border border-white/10 bg-white/5 backdrop-blur-md">
        <table className="w-full text-sm text-left">
          <thead className="sticky top-0 z-10 bg-white/5 text-xs uppercase text-muted-foreground border-b border-white/10 backdrop-blur-md">
            <tr>
              <th className="px-4 py-3">Cliente</th>
              <th className="px-4 py-3">Contato</th>
              <th className="px-4 py-3">CPF/CNPJ</th>
              <th className="px-4 py-3 text-right">Crédito Disponível</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-white/5">
            {loading && (
              <tr>
                <td
                  colSpan={4}
                  className="px-4 py-10 text-center text-muted-foreground"
                >
                  Carregando clientes do MySQL…
                </td>
              </tr>
            )}
            {!loading && slice.length === 0 && (
              <tr>
                <td
                  colSpan={4}
                  className="px-4 py-10 text-center text-muted-foreground"
                >
                  {busca
                    ? 'Nenhum cliente encontrado para essa busca.'
                    : 'Nenhum cliente cadastrado ainda.'}
                </td>
              </tr>
            )}
            {slice.map((c) => (
              <tr key={c.id} className="hover:bg-white/5 transition-colors">
                <td className="px-4 py-3 font-medium text-foreground">
                  <div className="flex items-center gap-3">
                    <div className="h-8 w-8 rounded-full bg-gradient-to-tr from-cyan-500/30 to-blue-600/30 border border-cyan-500/30 flex items-center justify-center text-cyan-300 text-xs font-bold">
                      {c.nome.slice(0, 2).toUpperCase()}
                    </div>
                    {c.nome}
                  </div>
                </td>
                <td className="px-4 py-3 text-muted-foreground">
                  <div className="flex flex-col gap-0.5">
                    {c.email && (
                      <span className="inline-flex items-center gap-1.5">
                        <Mail className="h-3.5 w-3.5" />
                        {c.email}
                      </span>
                    )}
                    {c.telefone && (
                      <span className="inline-flex items-center gap-1.5">
                        <Phone className="h-3.5 w-3.5" />
                        {c.telefone}
                      </span>
                    )}
                  </div>
                </td>
                <td className="px-4 py-3 text-muted-foreground font-mono text-xs">
                  {c.cpf_cnpj ?? '—'}
                </td>
                <td className="px-4 py-3 text-right">
                  <span className="inline-flex items-center gap-1.5 font-semibold text-emerald-400">
                    <Wallet className="h-3.5 w-3.5" />
                    {fmtBRL(Number(c.credito_disponivel) || 0)}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {totalPaginas > 1 && (
        <div className="flex items-center justify-between text-sm">
          <div className="text-muted-foreground">
            Página{' '}
            <span className="text-foreground font-semibold">{paginaAtual}</span>{' '}
            de {totalPaginas}
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setPage((p) => Math.max(1, p - 1))}
              disabled={paginaAtual === 1}
              className="gap-1"
            >
              <ChevronLeft className="h-4 w-4" />
              Anterior
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setPage((p) => Math.min(totalPaginas, p + 1))}
              disabled={paginaAtual >= totalPaginas}
              className="gap-1"
            >
              Próxima
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}
