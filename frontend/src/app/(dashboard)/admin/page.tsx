"use client"

import { useEffect, useMemo, useState } from "react"
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Users,
  Shield,
  Lock,
  UserPlus,
  Search,
  CheckCircle2,
  AlertTriangle,
  X,
} from "lucide-react"
import {
  usuariosService,
  AlterarPapelNaoImplementadoError,
  type InfoUsuario,
} from "@/services/usuarios.service"

const PAPEIS = [
  'Administrador',
  'Gerencia',
  'Tecnico',
  'Financeiro',
  'Comercial',
  'Estoquista',
] as const
type Papel = (typeof PAPEIS)[number]

export default function AdminPage() {
  const [busca, setBusca] = useState('')
  const [usuarios, setUsuarios] = useState<InfoUsuario[]>([])
  const [loading, setLoading] = useState(true)
  const [erro, setErro] = useState<string | null>(null)

  // === Modais ===
  const [openCreate, setOpenCreate] = useState(false)
  const [openEdit, setOpenEdit] = useState<InfoUsuario | null>(null)

  const carregar = async () => {
    try {
      setLoading(true)
      setErro(null)
      const data = await usuariosService.listar()
      setUsuarios(data)
    } catch (e: any) {
      setErro(e?.message ?? 'Falha ao carregar usuários')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    carregar()
  }, [])

  const filtrados = useMemo(() => {
    const q = busca.toLowerCase().trim()
    if (!q) return usuarios
    return usuarios.filter((u) =>
      u.nome_usuario.toLowerCase().includes(q),
    )
  }, [usuarios, busca])

  const ativos = usuarios.length
  const glassStyle = "backdrop-blur-xl bg-white/5 border-white/10 shadow-xl"

  if (erro && !usuarios.length) {
    return (
      <div className="flex flex-1 items-center justify-center p-6">
        <div className="max-w-md rounded-xl border border-amber-500/30 bg-amber-500/10 p-6 text-amber-200">
          <AlertTriangle className="h-5 w-5 mb-2" />
          <p className="font-semibold">Não foi possível carregar usuários</p>
          <p className="text-sm text-amber-300/80 mt-1">{erro}</p>
          <p className="text-xs text-muted-foreground mt-3">
            Exige permissão <code>empresa.usuario.list</code> no JWT.
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-1 flex-col gap-6 p-4 pt-0 md:p-6 md:pt-0">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">
            Administração & Segurança
          </h1>
          <p className="text-muted-foreground">
            Gestão de usuários, permissões (RBAC), registros de auditoria e
            configurações da plataforma.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <div className="relative">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              type="search"
              placeholder="Buscar por usuário..."
              value={busca}
              onChange={(e) => setBusca(e.target.value)}
              className="w-full bg-background pl-8 md:w-[250px] lg:w-[300px]"
            />
          </div>
          <Button
            onClick={() => setOpenCreate(true)}
            className="shrink-0 gap-2 bg-gradient-to-r from-cyan-500 to-blue-600 text-white hover:opacity-90 font-semibold shadow-md shadow-cyan-500/20"
          >
            <UserPlus className="h-4 w-4" />
            <span className="hidden sm:inline">Convidar Usuário</span>
          </Button>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        <Card className={glassStyle}>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">
              Usuários Cadastrados
            </CardTitle>
            <Users className="h-4 w-4 text-cyan-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {loading ? '—' : `${ativos} ${ativos === 1 ? 'usuário' : 'usuários'}`}
            </div>
            <p className="text-xs text-emerald-400 mt-1 flex items-center gap-1 font-medium">
              <CheckCircle2 className="h-3 w-3" />
              Sincronizado com MySQL
            </p>
          </CardContent>
        </Card>

        <Card className={glassStyle}>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">
              Papéis Disponíveis
            </CardTitle>
            <Shield className="h-4 w-4 text-emerald-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{PAPEIS.length} papéis</div>
            <p className="text-xs text-muted-foreground mt-1">
              {PAPEIS.join(' · ')}
            </p>
          </CardContent>
        </Card>

        <Card className={glassStyle}>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">
              Segurança & Conformidade
            </CardTitle>
            <Lock className="h-4 w-4 text-cyan-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-cyan-400">JWT + bcrypt</div>
            <p className="text-xs text-slate-300 mt-1 font-medium">
              Cookie HttpOnly · SameSite=Strict · Rate limit em /login
            </p>
          </CardContent>
        </Card>
      </div>

      <Card className={glassStyle}>
        <CardHeader>
          <CardTitle>Usuários Cadastrados</CardTitle>
          <CardDescription>
            Gerencie privilégios e controle de acesso dos membros da organização.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm text-left">
              <thead className="text-xs uppercase bg-white/5 text-muted-foreground border-b border-white/10">
                <tr>
                  <th className="px-4 py-3">ID</th>
                  <th className="px-4 py-3">Usuário</th>
                  <th className="px-4 py-3 text-right">Ações</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/5">
                {loading && (
                  <tr>
                    <td
                      colSpan={3}
                      className="px-4 py-6 text-center text-muted-foreground"
                    >
                      Carregando do MySQL…
                    </td>
                  </tr>
                )}
                {!loading && filtrados.length === 0 && (
                  <tr>
                    <td
                      colSpan={3}
                      className="px-4 py-6 text-center text-muted-foreground"
                    >
                      Nenhum usuário encontrado.
                    </td>
                  </tr>
                )}
                {filtrados.map((u) => (
                  <tr
                    key={u.id}
                    className="hover:bg-white/5 transition-colors"
                  >
                    <td className="px-4 py-3 font-mono text-cyan-400">
                      #{u.id}
                    </td>
                    <td className="px-4 py-3 font-semibold text-foreground">
                      {u.nome_usuario}
                    </td>
                    <td className="px-4 py-3 text-right">
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => setOpenEdit(u)}
                        className="h-8 text-xs hover:bg-white/10"
                      >
                        Editar
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {/* Modal: Convidar Usuário */}
      {openCreate && (
        <CreateUserModal
          onClose={() => setOpenCreate(false)}
          onCreated={() => {
            setOpenCreate(false)
            carregar()
          }}
        />
      )}

      {/* Modal: Editar Papel */}
      {openEdit && (
        <EditRoleModal
          user={openEdit}
          onClose={() => setOpenEdit(null)}
          onUpdated={() => {
            setOpenEdit(null)
            carregar()
          }}
        />
      )}
    </div>
  )
}

function CreateUserModal({
  onClose,
  onCreated,
}: {
  onClose: () => void
  onCreated: () => void
}) {
  const [submitting, setSubmitting] = useState(false)
  const [feedback, setFeedback] = useState<{
    type: 'ok' | 'err'
    msg: string
  } | null>(null)
  const [form, setForm] = useState({ nome_usuario: '', senha: '' })

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (form.senha.length < 8) {
      setFeedback({
        type: 'err',
        msg: 'A senha deve ter no mínimo 8 caracteres.',
      })
      return
    }
    setSubmitting(true)
    setFeedback(null)
    try {
      const criado = await usuariosService.criar({
        nome_usuario: form.nome_usuario.trim(),
        senha: form.senha,
      })
      if (criado) {
        setFeedback({ type: 'ok', msg: `Usuário #${criado.id} criado.` })
        setTimeout(onCreated, 600)
      } else {
        setFeedback({
          type: 'err',
          msg: 'Backend não confirmou. Verifique permissão.',
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
    <ModalShell title="Convidar Usuário" onClose={() => !submitting && onClose()}>
      <form onSubmit={handleSubmit} className="p-5 flex flex-col gap-4">
        <Field
          label="Nome de usuário *"
          value={form.nome_usuario}
          onChange={(v) => setForm((f) => ({ ...f, nome_usuario: v }))}
          required
        />
        <Field
          label="Senha (mínimo 8 caracteres) *"
          type="password"
          value={form.senha}
          onChange={(v) => setForm((f) => ({ ...f, senha: v }))}
          required
        />

        {feedback && <FeedbackBox feedback={feedback} />}

        <div className="flex justify-end gap-2 pt-2">
          <Button
            type="button"
            variant="outline"
            onClick={onClose}
            disabled={submitting}
          >
            Cancelar
          </Button>
          <Button
            type="submit"
            disabled={submitting}
            className="bg-gradient-to-r from-cyan-500 to-blue-600 text-white font-semibold"
          >
            {submitting ? 'Criando...' : 'Criar usuário'}
          </Button>
        </div>
      </form>
    </ModalShell>
  )
}

function EditRoleModal({
  user,
  onClose,
  onUpdated,
}: {
  user: InfoUsuario
  onClose: () => void
  onUpdated: () => void
}) {
  const [submitting, setSubmitting] = useState(false)
  const [feedback, setFeedback] = useState<{
    type: 'ok' | 'err'
    msg: string
  } | null>(null)
  const [papel, setPapel] = useState<Papel>('Tecnico')

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setSubmitting(true)
    setFeedback(null)
    try {
      await usuariosService.alterarPapel(user.nome_usuario, papel)
      setFeedback({ type: 'ok', msg: `Papel alterado para ${papel}.` })
      setTimeout(onUpdated, 700)
    } catch (err) {
      if (err instanceof AlterarPapelNaoImplementadoError) {
        setFeedback({
          type: 'err',
          msg: 'Backend ainda não implementou alteração de papel (501 STUB). Disponível em sprint futura.',
        })
      } else {
        setFeedback({
          type: 'err',
          msg: err instanceof Error ? err.message : 'Erro ao alterar.',
        })
      }
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <ModalShell
      title={`Editar papel de ${user.nome_usuario}`}
      onClose={() => !submitting && onClose()}
    >
      <form onSubmit={handleSubmit} className="p-5 flex flex-col gap-4">
        <label className="flex flex-col gap-1 text-sm">
          <span className="text-xs text-muted-foreground font-medium">
            Novo papel
          </span>
          <select
            value={papel}
            onChange={(e) => setPapel(e.target.value as Papel)}
            className="h-8 w-full rounded-lg border border-input bg-[#0A0F1E]/50 px-2.5 text-sm"
          >
            {PAPEIS.map((p) => (
              <option key={p} value={p}>
                {p}
              </option>
            ))}
          </select>
        </label>

        {feedback && <FeedbackBox feedback={feedback} />}

        <div className="flex justify-end gap-2 pt-2">
          <Button
            type="button"
            variant="outline"
            onClick={onClose}
            disabled={submitting}
          >
            Cancelar
          </Button>
          <Button
            type="submit"
            disabled={submitting}
            className="bg-gradient-to-r from-cyan-500 to-blue-600 text-white font-semibold"
          >
            {submitting ? 'Salvando...' : 'Salvar papel'}
          </Button>
        </div>
      </form>
    </ModalShell>
  )
}

function ModalShell({
  title,
  onClose,
  children,
}: {
  title: string
  onClose: () => void
  children: React.ReactNode
}) {
  return (
    <div
      role="dialog"
      aria-modal="true"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md rounded-2xl border border-white/10 bg-[#0F172A]/95 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between p-5 border-b border-white/10">
          <h2 className="text-lg font-semibold">{title}</h2>
          <button
            onClick={onClose}
            className="text-muted-foreground hover:text-foreground"
            aria-label="Fechar"
          >
            <X className="h-5 w-5" />
          </button>
        </div>
        {children}
      </div>
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

function FeedbackBox({
  feedback,
}: {
  feedback: { type: 'ok' | 'err'; msg: string }
}) {
  return (
    <div
      className={`p-2 rounded text-xs ${
        feedback.type === 'ok'
          ? 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/30'
          : 'bg-rose-500/10 text-rose-300 border border-rose-500/30'
      }`}
    >
      {feedback.msg}
    </div>
  )
}
