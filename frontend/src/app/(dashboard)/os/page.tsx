"use client"

import { useState } from "react"
import { OsKanbanBoard } from "@/components/os/os-kanban-board"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Plus, Search, Filter, X } from "lucide-react"
import { osService } from "@/services/os.service"
import { clientesService } from "@/services/clientes.service"

export default function OsPage() {
  const [openDialog, setOpenDialog] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [feedback, setFeedback] = useState<{
    type: 'ok' | 'err'
    msg: string
  } | null>(null)

  const [form, setForm] = useState({
    cliente_nome: '',
    equipamento: '',
    defeito_relatado: '',
  })

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setSubmitting(true)
    setFeedback(null)
    try {
      // Como /ordens não está paginado e exige OrdemServico completo,
      // criamos com um payload mínimo que satisfaz o mínimo de campos do openapi.
      // O back completa o resto.
      const clientes = await clientesService.listar()
      const cliente =
        clientes.find(
          (c) =>
            c.nome.toLowerCase() === form.cliente_nome.toLowerCase().trim(),
        ) ?? clientes[0]
      if (!cliente) {
        setFeedback({
          type: 'err',
          msg: 'Nenhum cliente cadastrado. Cadastre um cliente antes de criar a OS.',
        })
        setSubmitting(false)
        return
      }

      const criada = await osService.criar({
        cliente: cliente.nome,
        equipamento: form.equipamento.trim() || 'A definir',
        defeito_relatado: form.defeito_relatado.trim() || 'A diagnosticar',
        status: 'Aberta',
        situacao: 'Orcamento',
        parecer_tecnico: '',
        numero_serie_equipamento: '',
        observacoes: '',
        nome_tecnico_responsavel: '',
        atendente: '',
        horario_abertura: new Date().toISOString(),
        telefone_cliente: cliente.telefone ?? '',
        data_chegada: new Date().toISOString(),
        prazo_entrega: '',
        historico_edicoes: [],
        pecas: [],
        total_pecas: 0,
        servicos: [],
        total_servicos: 0,
      } as unknown as Parameters<typeof osService.criar>[0])

      if (criada) {
        setFeedback({ type: 'ok', msg: `OS #${criada.id} criada.` })
        setForm({ cliente_nome: '', equipamento: '', defeito_relatado: '' })
        setTimeout(() => window.location.reload(), 800)
      } else {
        setFeedback({
          type: 'err',
          msg: 'Backend não confirmou criação. Verifique permissões.',
        })
      }
    } catch (err) {
      setFeedback({
        type: 'err',
        msg: err instanceof Error ? err.message : 'Erro ao criar OS.',
      })
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="flex h-[calc(100vh-3.5rem)] flex-col gap-4 p-4 md:gap-6 md:p-6 lg:h-[calc(100vh-4rem)]">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">
            Ordens de Serviço (OS)
          </h1>
          <p className="text-muted-foreground">
            Acompanhe e gerencie chamados técnicos e execução de serviços com
            controle de SLA.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <div className="relative">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              type="search"
              placeholder="Buscar por número da OS ou cliente..."
              className="w-full bg-background pl-8 md:w-[250px] lg:w-[300px]"
            />
          </div>
          <Button variant="outline" size="icon" className="shrink-0">
            <Filter className="h-4 w-4" />
          </Button>
          <Button
            onClick={() => setOpenDialog(true)}
            className="shrink-0 gap-2 bg-gradient-to-r from-cyan-500 to-blue-600 text-white hover:opacity-90 font-semibold shadow-md shadow-cyan-500/20"
          >
            <Plus className="h-4 w-4" />
            <span className="hidden sm:inline">Nova OS</span>
          </Button>
        </div>
      </div>

      <div className="flex-1 overflow-hidden min-h-0">
        <OsKanbanBoard />
      </div>

      {openDialog && (
        <div
          role="dialog"
          aria-modal="true"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
          onClick={() => !submitting && setOpenDialog(false)}
        >
          <div
            className="w-full max-w-lg rounded-2xl border border-white/10 bg-[#0F172A]/95 shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between p-5 border-b border-white/10">
              <h2 className="text-lg font-semibold">Nova OS</h2>
              <button
                onClick={() => setOpenDialog(false)}
                disabled={submitting}
                className="text-muted-foreground hover:text-foreground"
                aria-label="Fechar"
              >
                <X className="h-5 w-5" />
              </button>
            </div>

            <form onSubmit={handleSubmit} className="p-5 flex flex-col gap-4">
              <Field
                label="Cliente (nome exato)"
                value={form.cliente_nome}
                onChange={(v) => setForm((f) => ({ ...f, cliente_nome: v }))}
                required
              />
              <Field
                label="Equipamento"
                value={form.equipamento}
                onChange={(v) => setForm((f) => ({ ...f, equipamento: v }))}
                placeholder="Ex: Notebook Dell Inspiron"
              />
              <Field
                label="Defeito relatado"
                value={form.defeito_relatado}
                onChange={(v) =>
                  setForm((f) => ({ ...f, defeito_relatado: v }))
                }
                placeholder="Ex: Não liga"
              />

              {feedback && (
                <div
                  className={`p-2 rounded text-xs ${
                    feedback.type === 'ok'
                      ? 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/30'
                      : 'bg-rose-500/10 text-rose-300 border border-rose-500/30'
                  }`}
                >
                  {feedback.msg}
                </div>
              )}

              <div className="flex justify-end gap-2 pt-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setOpenDialog(false)}
                  disabled={submitting}
                >
                  Cancelar
                </Button>
                <Button
                  type="submit"
                  disabled={submitting}
                  className="bg-gradient-to-r from-cyan-500 to-blue-600 text-white font-semibold"
                >
                  {submitting ? 'Criando...' : 'Criar OS'}
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}

function Field({
  label,
  value,
  onChange,
  placeholder,
  required,
}: {
  label: string
  value: string
  onChange: (v: string) => void
  placeholder?: string
  required?: boolean
}) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="text-xs text-muted-foreground font-medium">{label}</span>
      <Input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        required={required}
        className="bg-[#0A0F1E]/50 border-[#1E293B]"
      />
    </label>
  )
}
