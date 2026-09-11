"use client"

import React, { useState, useEffect } from "react"
import { Bell, Search, Menu, Sun, Moon, LogOut, Shield } from "lucide-react"
import { usePathname, useRouter } from "next/navigation"
import { Button } from "@/components/ui/button"
import { useAuthStore } from "@/stores/auth.store"

interface TopbarProps {
  isCollapsed: boolean
  onToggle: () => void
}

export function Topbar({ onToggle }: TopbarProps) {
  const pathname = usePathname()
  const router = useRouter()
  const logout = useAuthStore(state => state.logout)
  const user = useAuthStore(state => state.user)

  const [isDark, setIsDark] = useState(true)

  useEffect(() => {
    // Inicializar o tema dark por padrão ou conforme preferência armazenada
    const isDarkTheme = document.documentElement.classList.contains('dark') || true
    setIsDark(isDarkTheme)
    if (isDarkTheme) {
      document.documentElement.classList.add('dark')
    }
  }, [])

  const toggleTheme = () => {
    const nextDark = !isDark
    setIsDark(nextDark)
    if (nextDark) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }

  const handleLogout = () => {
    logout()
    router.push('/login')
  }

  // Breadcrumb mapping em Português. Aliases (/crm, /servicos) apontam pro mesmo label.
  const breadcrumbMap: Record<string, string> = {
    "/": "Dashboard",
    "/dashboard": "Dashboard",
    "/clientes": "Clientes",
    "/crm": "Clientes",
    "/os": "Ordens de Serviço",
    "/servicos": "Ordens de Serviço",
    "/orcamentos": "Orçamentos",
    "/estoque": "Estoque",
    "/financeiro": "Financeiro",
    "/relatorios": "Relatórios",
    "/admin": "Administração"
  }

  const currentPathName = breadcrumbMap[pathname] || "Dashboard"

  return (
    <header className="sticky top-0 z-30 flex h-16 items-center gap-4 border-b bg-background/80 backdrop-blur-md px-4 sm:px-6 shadow-sm">
      <div className="flex flex-1 items-center gap-4 justify-between">
        <div className="flex items-center gap-4">
          <Button 
            variant="ghost" 
            size="icon" 
            className="sm:hidden"
            onClick={onToggle}
          >
            <Menu className="h-5 w-5" />
            <span className="sr-only">Menu</span>
          </Button>

          <div className="hidden sm:flex items-center text-sm text-muted-foreground font-medium">
            <span>GARSystem</span>
            <span className="mx-2 text-muted-foreground/50">/</span>
            <span className="text-foreground font-semibold">{currentPathName}</span>
          </div>
        </div>

        <div className="hidden lg:flex items-center gap-2 flex-1 justify-center max-w-2xl mx-auto">
          <CommandButton icon={Search} label="Pesquisar" shortcut="⌘K" />
          <CommandButton label="Criar OS" shortcut="⌘O" />
          <CommandButton label="Criar Orçamento" shortcut="⌘B" />
          <CommandButton label="Buscar NFe" shortcut="⌘F" />
        </div>
        
        <div className="flex items-center gap-3">
          {/* Botão de Alternar Tema (Dark / Light) */}
          <Button 
            variant="ghost" 
            size="icon" 
            onClick={toggleTheme}
            title={isDark ? "Mudar para Modo Claro" : "Mudar para Modo Escuro"}
            className="relative text-muted-foreground hover:text-foreground hover:bg-accent/50"
          >
            {isDark ? <Sun className="h-5 w-5 text-amber-400" /> : <Moon className="h-5 w-5 text-slate-700" />}
            <span className="sr-only">Alternar Tema</span>
          </Button>

          {/* Notificações */}
          <Button variant="ghost" size="icon" className="relative text-muted-foreground hover:text-foreground">
            <Bell className="h-5 w-5" />
            <span className="absolute top-1.5 right-1.5 h-2 w-2 rounded-full bg-cyan-400 animate-pulse" />
            <span className="sr-only">Notificações</span>
          </Button>
          
          <div className="h-4 w-[1px] bg-border mx-1" />

          {/* Perfil & Botão Deslogar */}
          <div className="flex items-center gap-2">
            <div className="h-8 w-8 rounded-full bg-gradient-to-tr from-cyan-500 to-blue-600 flex items-center justify-center text-white font-bold text-xs shadow-md shadow-cyan-500/20 ring-2 ring-background">
              {(user?.username?.substring(0, 2) ?? user?.papel?.substring(0, 2) ?? 'GB').toUpperCase()}
            </div>
            
            <Button 
              variant="outline" 
              size="sm" 
              onClick={handleLogout}
              className="gap-2 text-xs font-semibold text-rose-500 hover:text-rose-400 hover:bg-rose-950/30 border-rose-500/20 hover:border-rose-500/40"
              title="Deslogar do Sistema"
            >
              <LogOut className="h-3.5 w-3.5" />
              <span className="hidden md:inline">Sair</span>
            </Button>
          </div>
        </div>
      </div>
    </header>
  )
}

function CommandButton({ icon: Icon, label, shortcut }: { icon?: any, label: string, shortcut: string }) {
  return (
    <button className="group flex items-center gap-2 px-3 py-1.5 rounded-md hover:bg-accent/40 border border-transparent hover:border-border transition-all text-sm text-muted-foreground hover:text-foreground bg-transparent">
      {Icon && <Icon className="w-4 h-4 opacity-70 group-hover:opacity-100 text-cyan-400" />}
      <span>{label}</span>
      <kbd className="hidden sm:inline-flex items-center gap-1 px-1.5 font-mono text-[10px] font-medium text-muted-foreground/70 bg-muted/40 rounded border border-muted/20 group-hover:text-foreground transition-colors">
        {shortcut}
      </kbd>
    </button>
  )
}
