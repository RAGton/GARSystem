"use client"

import { useState } from "react"
import { KanbanBoard } from "@/components/crm/kanban-board"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Plus, Search, Filter, X } from "lucide-react"
import { clientesService } from "@/services/clientes.service"

export default function CrmPage() {
  const [openDialog, setOpenDialog] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [feedback, setFeedback] = useState<{
    type: 'ok' | 'err'
    msg: string
  } | null>(null)

  const [form, setForm] = useState({
    nome: '',
    email: '',
    telefone: '',
    endereco: '',
    cpf_cnpj: '',
  })

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setSubmitting(true)
    setFeedback(null)
    try {
      const criado = await clientesService.criar({
        nome: form.nome.trim(),
        email: form.email.trim(),
        telefone: form.telefone.trim(),
        endereco: form.endereco.trim() || null,
        cpf_cnpj: form.cpf_cnpj.trim() || null,
      })
      if (criado) {
        setFeedback({ type: 'ok', msg: `Cliente #${criado.id} criado.` })
        setForm({ nome: '', email: '', telefone: '', endereco: '', cpf_cnpj: '' })
        // Recarrega a página para mostrar o novo cliente (mantém simples).
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
        msg: err instanceof Error ? err.message : 'Erro ao criar.',
      })
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="flex h-[calc(100vh-3.5rem)] flex-col gap-4 p-4 md:gap-6 md:p-6 lg:h-[calc(100vh-4rem)]">
      {/* Cabeçalho do Módulo */}
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Clientes & CRM</h1>
          <p className="text-muted-foreground">
            Gestão de clientes, contatos, observações e histórico de OS.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <div className="relative">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              type="search"
              placeholder="Buscar clientes..."
              className="w-full bg-background pl-8 md:w-[200px] lg:w-[300px]"
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
            <span className="hidden sm:inline">Nova Oportunidade</span>
          </Button>
        </div>
      </div>

      {/* Container */}
      <div className="flex-1 overflow-hidden min-h-0">
        <KanbanBoard />
      </div>

      {/* Dialog modal de criação de cliente */}
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
              <h2 className="text-lg font-semibold">Novo Cliente</h2>
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
                label="Nome *"
                value={form.nome}
                onChange={(v) => setForm((f) => ({ ...f, nome: v }))}
                required
              />
              <div className="grid grid-cols-2 gap-3">
                <Field
                  label="Email *"
                  type="email"
                  value={form.email}
                  onChange={(v) => setForm((f) => ({ ...f, email: v }))}
                  required
                />
                <Field
                  label="Telefone *"
                  value={form.telefone}
                  onChange={(v) => setForm((f) => ({ ...f, telefone: v }))}
                  required
                />
              </div>
              <Field
                label="Endereço"
                value={form.endereco}
                onChange={(v) => setForm((f) => ({ ...f, endereco: v }))}
              />
              <Field
                label="CPF/CNPJ"
                value={form.cpf_cnpj}
                onChange={(v) => setForm((f) => ({ ...f, cpf_cnpj: v }))}
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
                  {submitting ? 'Criando...' : 'Criar cliente'}
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
  type = 'text',
  required,
}: {
  label: string
  value: string
  onChange: (v: string) => void
  type?: string
  required?: boolean
}) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="text-xs text-muted-foreground font-medium">{label}</span>
      <Input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        required={required}
        className="bg-[#0A0F1E]/50 border-[#1E293B]"
      />
    </label>
  )
}
