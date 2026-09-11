"use client"

import { useEffect, useMemo, useState } from "react"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Package, Plus, Search, Filter, AlertTriangle, ArrowDownRight, Layers } from "lucide-react"
import { fetchApi } from "@/lib/api"

interface Peca {
  id: number
  nome: string
  codigo_interno: string
  part_number: string
  descricao: string
  fabricante: string
  localizacao: string
  estoque_atual: number
  estoque_minimo: number
  preco_custo: number
  preco_venda: number
}

type Status = 'Normal' | 'Baixo Estoque' | 'Sem Estoque'
function statusDe(p: Peca): Status {
  if (p.estoque_atual <= 0) return 'Sem Estoque'
  if (p.estoque_atual <= p.estoque_minimo) return 'Baixo Estoque'
  return 'Normal'
}

const fmtBRL = (n: number) =>
  new Intl.NumberFormat('pt-BR', { style: 'currency', currency: 'BRL' }).format(n)

export default function EstoquePage() {
  const [busca, setBusca] = useState("")
  const [pecas, setPecas] = useState<Peca[]>([])
  const [loading, setLoading] = useState(true)
  const [erro, setErro] = useState<string | null>(null)

  useEffect(() => {
    let cancelado = false
    async function carregar() {
      try {
        setLoading(true)
        const data = await fetchApi<Peca[]>('/estoque/pecas')
        if (!cancelado) setPecas(data)
      } catch (e: any) {
        if (!cancelado) setErro(e?.message ?? 'Falha ao carregar peças')
      } finally {
        if (!cancelado) setLoading(false)
      }
    }
    carregar()
    return () => { cancelado = true }
  }, [])

  const filtrados = useMemo(() => {
    const q = busca.toLowerCase().trim()
    if (!q) return pecas
    return pecas.filter(p =>
      p.nome.toLowerCase().includes(q) ||
      p.codigo_interno.toLowerCase().includes(q) ||
      p.part_number.toLowerCase().includes(q) ||
      p.descricao.toLowerCase().includes(q)
    )
  }, [pecas, busca])

  const totalUnidades = pecas.reduce((acc, p) => acc + p.estoque_atual, 0)
  const valorInventario = pecas.reduce(
    (acc, p) => acc + p.estoque_atual * p.preco_custo, 0
  )
  const qtdBaixoEstoque = pecas.filter(p => statusDe(p) === 'Baixo Estoque').length
  const qtdZerados = pecas.filter(p => statusDe(p) === 'Sem Estoque').length

  const glassStyle = "backdrop-blur-xl bg-white/5 border-white/10 shadow-xl"

  if (erro) {
    return (
      <div className="flex flex-1 items-center justify-center p-6">
        <div className="max-w-md rounded-xl border border-amber-500/30 bg-amber-500/10 p-6 text-amber-200">
          <AlertTriangle className="h-5 w-5 mb-2" />
          <p className="font-semibold">Não foi possível carregar o estoque</p>
          <p className="text-sm text-amber-300/80 mt-1">{erro}</p>
          <p className="text-xs text-muted-foreground mt-3">
            Verifique se o backend está em <code>localhost:3000</code>.
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-1 flex-col gap-6 p-4 pt-0 md:p-6 md:pt-0">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Gestão de Estoque</h1>
          <p className="text-muted-foreground">
            Controle de inventário, alertas de estoque mínimo e movimentações de produtos.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <div className="relative">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              type="search"
              placeholder="Buscar por nome, SKU ou PN..."
              value={busca}
              onChange={(e) => setBusca(e.target.value)}
              className="w-full bg-background pl-8 md:w-[250px] lg:w-[300px]"
            />
          </div>
          <Button variant="outline" size="icon" className="shrink-0 border-white/10">
            <Filter className="h-4 w-4" />
          </Button>
          <Button className="shrink-0 gap-2 bg-gradient-to-r from-cyan-500 to-blue-600 text-white hover:opacity-90 font-semibold shadow-md shadow-cyan-500/20">
            <Plus className="h-4 w-4" />
            <span className="hidden sm:inline">Novo Produto</span>
          </Button>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card className={glassStyle}>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Total de Itens em Estoque</CardTitle>
            <Package className="h-4 w-4 text-cyan-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {loading ? '—' : `${totalUnidades} unidades`}
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              {loading ? 'Calculando…' : `${pecas.length} SKUs cadastrados`}
            </p>
          </CardContent>
        </Card>

        <Card className={glassStyle}>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Valor Total do Inventário</CardTitle>
            <Layers className="h-4 w-4 text-emerald-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {loading ? '—' : fmtBRL(valorInventario)}
            </div>
            <p className="text-xs text-muted-foreground mt-1">Custo total atual</p>
          </CardContent>
        </Card>

        <Card className={glassStyle}>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Estoque Baixo</CardTitle>
            <AlertTriangle className="h-4 w-4 text-amber-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-amber-400">
              {loading ? '—' : `${qtdBaixoEstoque} ${qtdBaixoEstoque === 1 ? 'peça' : 'peças'}`}
            </div>
            <p className="text-xs text-amber-400/80 mt-1">Abaixo do mínimo recomendado</p>
          </CardContent>
        </Card>

        <Card className={glassStyle}>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">Itens Zerados</CardTitle>
            <ArrowDownRight className="h-4 w-4 text-rose-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-rose-400">
              {loading ? '—' : `${qtdZerados} ${qtdZerados === 1 ? 'peça' : 'peças'}`}
            </div>
            <p className="text-xs text-rose-400/80 mt-1 font-medium">
              {qtdZerados > 0 ? 'Necessita compra urgente' : 'Tudo OK'}
            </p>
          </CardContent>
        </Card>
      </div>

      <Card className={glassStyle}>
        <CardHeader>
          <CardTitle>Inventário de Produtos</CardTitle>
          <CardDescription>Lista completa de insumos, peças e equipamentos cadastrados.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm text-left">
              <thead className="text-xs uppercase bg-white/5 text-muted-foreground border-b border-white/10">
                <tr>
                  <th className="px-4 py-3">SKU</th>
                  <th className="px-4 py-3">Produto</th>
                  <th className="px-4 py-3">Fabricante</th>
                  <th className="px-4 py-3">Localização</th>
                  <th className="px-4 py-3">Qtd. Atual</th>
                  <th className="px-4 py-3">Qtd. Mínima</th>
                  <th className="px-4 py-3">Custo</th>
                  <th className="px-4 py-3">Venda</th>
                  <th className="px-4 py-3">Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/5">
                {loading && (
                  <tr><td colSpan={9} className="px-4 py-6 text-center text-muted-foreground">
                    Carregando do MySQL…
                  </td></tr>
                )}
                {!loading && filtrados.length === 0 && (
                  <tr><td colSpan={9} className="px-4 py-6 text-center text-muted-foreground">
                    Nenhuma peça encontrada.
                  </td></tr>
                )}
                {filtrados.map((item) => {
                  const status = statusDe(item)
                  return (
                    <tr key={item.id} className="hover:bg-white/5 transition-colors">
                      <td className="px-4 py-3 font-mono text-cyan-400 font-semibold">
                        {item.codigo_interno}
                      </td>
                      <td className="px-4 py-3 font-medium text-foreground">
                        {item.nome}
                      </td>
                      <td className="px-4 py-3 text-muted-foreground">{item.fabricante}</td>
                      <td className="px-4 py-3 text-muted-foreground">{item.localizacao}</td>
                      <td className="px-4 py-3 font-bold text-foreground">{item.estoque_atual}</td>
                      <td className="px-4 py-3 text-muted-foreground">{item.estoque_minimo}</td>
                      <td className="px-4 py-3 font-medium">{fmtBRL(item.preco_custo)}</td>
                      <td className="px-4 py-3 font-medium">{fmtBRL(item.preco_venda)}</td>
                      <td className="px-4 py-3">
                        <span className={`px-2.5 py-1 rounded-full text-xs font-semibold ${
                          status === 'Normal'
                            ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                            : status === 'Baixo Estoque'
                            ? 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                            : 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
                        }`}>
                          {status}
                        </span>
                      </td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
