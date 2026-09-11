"use client"

import React from "react"
import { motion, AnimatePresence } from "framer-motion"
import {
  LayoutDashboard,
  Users,
  Wrench,
  Package,
  CircleDollarSign,
  FileText,
  BarChart3,
  Settings,
  ChevronLeft,
  ChevronRight,
} from "lucide-react"
import Link from "next/link"
import Image from "next/image"
import { usePathname } from "next/navigation"
import { cn } from "@/lib/utils"

// Roles permitidos por rota (server-side enforcement acontece via
// `check_perm(claims, ...)` no Rust; isto é só UX).
const navigationGroups = [
  {
    title: "VISÃO GERAL",
    items: [
      { name: "Dashboard", href: "/", icon: LayoutDashboard }
    ]
  },
  {
    title: "NEGÓCIOS",
    items: [
      { name: "Clientes", href: "/clientes", icon: Users },
      { name: "Ordens de Serviço", href: "/os", icon: Wrench },
      { name: "Orçamentos", href: "/orcamentos", icon: FileText }
    ]
  },
  {
    title: "OPERACIONAL",
    items: [
      { name: "Estoque", href: "/estoque", icon: Package },
      { name: "Financeiro", href: "/financeiro", icon: CircleDollarSign }
    ]
  },
  {
    title: "SISTEMA",
    items: [
      { name: "Relatórios", href: "/relatorios", icon: BarChart3 },
      { name: "Administração", href: "/admin", icon: Settings }
    ]
  }
]

interface SidebarProps {
  isCollapsed: boolean
  onToggle: () => void
}

export function Sidebar({ isCollapsed, onToggle }: SidebarProps) {
  const pathname = usePathname()

  return (
    <motion.aside
      initial={false}
      animate={{ width: isCollapsed ? 80 : 256 }}
      className="fixed inset-y-0 left-0 z-20 hidden flex-col border-r bg-background sm:flex shadow-sm"
      transition={{ duration: 0.3, ease: "easeInOut" }}
    >
      <div className="flex h-16 shrink-0 items-center justify-between px-4 border-b">
        <div className="flex items-center gap-3 overflow-hidden">
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-gradient-to-tr from-cyan-500/20 to-blue-600/20 border border-cyan-500/30 p-1 shadow-sm shadow-cyan-500/10">
            <Image src="/icon.png" alt="GARSystem Icon" width={28} height={28} className="object-contain drop-shadow-[0_0_8px_rgba(0,210,255,0.5)]" />
          </div>
          <AnimatePresence>
            {!isCollapsed && (
              <motion.span
                initial={{ opacity: 0, width: 0 }}
                animate={{ opacity: 1, width: "auto" }}
                exit={{ opacity: 0, width: 0 }}
                className="whitespace-nowrap font-semibold tracking-tight"
                transition={{ duration: 0.2 }}
              >
                GARSystem
              </motion.span>
            )}
          </AnimatePresence>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto overflow-x-hidden py-4 custom-scrollbar">
        <nav className="space-y-6 px-3">
          {navigationGroups.map((group, i) => (
            <div key={i} className="space-y-1">
              {!isCollapsed && (
                <motion.div 
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className="px-3 mb-2 text-xs font-semibold tracking-wider text-muted-foreground"
                >
                  {group.title}
                </motion.div>
              )}
              {group.items.map((item) => {
                // Ativa em pathname exato OU em rotas-irmãs (ex: /crm e /clientes são aliases)
                const aliasMap: Record<string, string[]> = {
                  "/clientes": ["/crm"],
                  "/os": ["/servicos"],
                  "/": [],
                }
                const aliases = aliasMap[item.href] ?? []
                const isActive =
                  pathname === item.href || aliases.includes(pathname)
                return (
                  <Link
                    key={item.name}
                    href={item.href}
                    title={isCollapsed ? item.name : undefined}
                    className={cn(
                      "group flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-all relative overflow-hidden",
                      isActive 
                        ? "bg-primary/10 text-primary font-semibold" 
                        : "text-muted-foreground hover:bg-muted/50 hover:text-foreground"
                    )}
                  >
                    <item.icon className={cn("h-5 w-5 shrink-0 transition-colors", isActive ? "text-primary" : "text-muted-foreground group-hover:text-foreground")} />
                    
                    <AnimatePresence>
                      {!isCollapsed && (
                        <motion.span
                          initial={{ opacity: 0, width: 0 }}
                          animate={{ opacity: 1, width: "auto" }}
                          exit={{ opacity: 0, width: 0 }}
                          className="whitespace-nowrap"
                        >
                          {item.name}
                        </motion.span>
                      )}
                    </AnimatePresence>

                    {/* Active Indicator */}
                    {isActive && (
                      <motion.div 
                        layoutId="active-indicator"
                        className="absolute left-0 top-0 h-full w-[3px] rounded-r-md bg-primary"
                        transition={{ duration: 0.2 }}
                      />
                    )}
                  </Link>
                )
              })}
            </div>
          ))}
        </nav>
      </div>

      <div className="p-4 border-t flex justify-end">
        <button
          onClick={onToggle}
          className="flex h-8 w-8 items-center justify-center rounded-md border bg-background text-muted-foreground hover:bg-muted hover:text-foreground transition-colors shadow-sm"
          aria-label={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          {isCollapsed ? <ChevronRight className="h-4 w-4" /> : <ChevronLeft className="h-4 w-4" />}
        </button>
      </div>
    </motion.aside>
  )
}
