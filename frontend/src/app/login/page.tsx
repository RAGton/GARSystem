'use client'

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { ShieldCheck, AlertCircle, Loader2 } from "lucide-react"
import Image from "next/image"
import { useState } from "react"
import { authService } from "@/services/auth.service"
import { useAuthStore, type User } from "@/stores/auth.store"
import { useRouter } from "next/navigation"

export default function LoginPage() {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  
  const setUser = useAuthStore(state => state.setUser)
  const setFreshLogin = useAuthStore(state => state.setFreshLogin)
  const router = useRouter()

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true)
    setError(null)
    
    try {
      const data = await authService.login(username, password)
      // O service já hidratou a store; setUser aqui só garante o shape completo.
      const fullUser: User = {
        papel: data.papel,
        expira_em: data.expira_em,
        permissions: [],
        roles: [],
      }
      setUser(fullUser)
      setFreshLogin(true)
      // Tenta completar com /me (best-effort).
      void authService.me()
      router.push('/')
    } catch (err: any) {
      setError(err.message || 'Erro ao conectar com o servidor.')
    } finally {
      setLoading(false)
    }
  }

  return (
    <main className="h-screen w-screen fixed inset-0 flex bg-[#0A0F1E] text-white selection:bg-[#38BDF8] selection:text-white overflow-hidden">

      {/* LEFT COLUMN: Institutional Branding (60%) */}
      <div className="relative hidden lg:flex flex-col justify-between w-[60%] p-8 lg:p-12 border-r border-[#1E293B] bg-gradient-to-br from-[#0A0F1E] to-[#111827]">
        {/* Abstract Mesh Background / Glow */}
        <div className="absolute top-[-20%] left-[-10%] w-[80%] h-[80%] rounded-full bg-[#06B6D4]/10 blur-[150px] pointer-events-none" />
        <div className="absolute bottom-[-10%] right-[-10%] w-[60%] h-[60%] rounded-full bg-[#38BDF8]/10 blur-[120px] pointer-events-none" />

        {/* Animated Triangular Mosaic */}
        <div className="absolute inset-0 z-0 overflow-hidden pointer-events-none opacity-20">
          <svg className="absolute w-[200%] h-[200%] -top-[50%] -left-[50%] animate-[spin_240s_linear_infinite]" xmlns="http://www.w3.org/2000/svg">
            <defs>
              <pattern id="mosaic" width="160" height="160" patternUnits="userSpaceOnUse">
                <polygon points="80,0 160,160 0,160" fill="none" stroke="#38BDF8" strokeWidth="0.5" className="animate-pulse" />
                <polygon points="0,0 80,160 160,0" fill="none" stroke="#06B6D4" strokeWidth="0.5" className="animate-[pulse_4s_ease-in-out_infinite]" />
              </pattern>
            </defs>
            <rect width="100%" height="100%" fill="url(#mosaic)" />
          </svg>
        </div>

        <div className="relative z-10 flex-1 flex flex-col justify-start pt-8 lg:pt-12 space-y-8">
          {/* Animated Logo Container */}
          <div className="flex items-center">
            <div className="relative group p-2">
              <Image
                src="/logo.png"
                alt="GARSystem Logo"
                width={360}
                height={100}
                className="relative object-contain drop-shadow-[0_0_25px_rgba(0,210,255,0.4)]"
                priority
              />
            </div>
          </div>

          <div className="space-y-4">
            <h2 className="text-3xl lg:text-4xl font-extrabold tracking-tight leading-tight">
              Plataforma Integrada de <br />
              <span className="text-transparent bg-clip-text bg-gradient-to-r from-[#38BDF8] via-[#00D2FF] to-[#3B82F6]">
                Gestão Empresarial Crítica
              </span>
            </h2>
          </div>
        </div>

        <div className="relative z-10 flex items-center justify-between text-sm text-slate-300 font-medium pt-6 border-t border-[#1E293B] mt-8">
          <span>© {new Date().getFullYear()} GARSystem. Todos os direitos reservados.</span>
          <div className="flex gap-4 text-xs text-slate-400">
            <span className="bg-[#1E293B] px-2 py-1 rounded text-cyan-300 font-mono">v1.0.0</span>
            <a href="#" className="hover:text-cyan-300 transition-colors self-center">Licença</a>
          </div>
        </div>
      </div>

      {/* RIGHT COLUMN: Authentication (40%) */}
      <div className="relative w-full lg:w-[40%] flex items-center justify-center bg-[#0A0F1E]">
        {/* Organic Divider / Flowing Border on left edge */}
        <div className="absolute inset-y-0 left-[-1px] w-[2px] bg-gradient-to-b from-transparent via-[#38BDF8]/50 to-transparent shadow-[0_0_20px_rgba(56,189,248,0.8)]" />

        <div className="w-full max-w-md p-6 sm:p-8">
          <div className="backdrop-blur-2xl bg-[#111827]/60 border border-[#1E293B] rounded-2xl p-6 shadow-2xl relative overflow-hidden">
            {/* Subtle glow inside the card */}
            <div className="absolute top-0 right-0 w-32 h-32 bg-[#38BDF8]/10 blur-3xl pointer-events-none" />

            <div className="relative z-10 flex flex-col gap-6">
              <div className="flex flex-col gap-1">
                <h2 className="text-2xl font-bold text-white tracking-tight">Acesso ao Sistema</h2>
                <p className="text-[#94A3B8] text-sm">Insira suas credenciais corporativas.</p>
              </div>

              {error && (
                <div className="flex items-center gap-2 p-3 text-sm text-red-200 bg-red-950/50 border border-red-900/50 rounded-lg">
                  <AlertCircle className="w-4 h-4 shrink-0" />
                  <span>{error}</span>
                </div>
              )}

              <form className="flex flex-col gap-6" onSubmit={handleLogin}>
                <div className="flex flex-col gap-2">
                  <Label htmlFor="username" className="text-[#94A3B8]">Usuário</Label>
                  <Input
                    id="username"
                    type="text"
                    required
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    placeholder="ex: admin.gar"
                    className="bg-[#0A0F1E]/50 border-[#1E293B] focus-visible:ring-[#38BDF8] text-white placeholder:text-[#475569] h-11"
                    disabled={loading}
                  />
                </div>

                <div className="flex flex-col gap-2">
                  <div className="flex justify-between items-center">
                    <Label htmlFor="password" className="text-[#94A3B8]">Senha</Label>
                    <a href="#" className="text-xs text-[#38BDF8] hover:text-[#06B6D4] transition-colors">
                      Esqueceu a senha?
                    </a>
                  </div>
                  <Input
                    id="password"
                    type="password"
                    required
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="bg-[#0A0F1E]/50 border-[#1E293B] focus-visible:ring-[#38BDF8] text-white h-11"
                    disabled={loading}
                  />
                </div>

                <Button 
                  type="submit" 
                  disabled={loading}
                  className="w-full h-11 mt-4 bg-gradient-to-r from-[#00D2FF] via-[#38BDF8] to-[#0088FF] text-white hover:opacity-95 transition-all shadow-[0_0_20px_rgba(0,210,255,0.3)] font-semibold border-0"
                >
                  {loading ? (
                    <>
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                      Autenticando...
                    </>
                  ) : (
                    'Autenticar'
                  )}
                </Button>
              </form>

              <div className="flex items-center gap-3 justify-center text-xs text-[#475569] mt-4">
                <ShieldCheck className="w-4 h-4" />
                <span>Conexão Segura e Criptografada</span>
              </div>
            </div>
          </div>
        </div>
      </div>

    </main>
  )
}
