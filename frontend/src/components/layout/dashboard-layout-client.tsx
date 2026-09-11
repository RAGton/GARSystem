"use client"

import React, { useState, useEffect } from "react"
import { Sidebar } from "./sidebar"
import { Topbar } from "./topbar"
import { WorkspaceLoader } from "./workspace-loader"
import { useAuthStore } from "@/stores/auth.store"
import { cn } from "@/lib/utils"

export function DashboardLayoutClient({ children }: { children: React.ReactNode }) {
  const [isCollapsed, setIsCollapsed] = useState(false)
  const [mounted, setMounted] = useState(false)
  const isFreshLogin = useAuthStore(state => state.isFreshLogin)

  // Prevent hydration mismatch
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setMounted(true)
  }, [])

  if (!mounted) {
    return (
      <div className="flex min-h-screen w-full bg-muted/20">
        <div className="hidden w-64 border-r bg-background sm:block" />
        <div className="flex flex-col w-full sm:pl-64">
           <header className="sticky top-0 z-30 h-16 border-b bg-background/80 backdrop-blur-md" />
           <main className="flex-1 p-6">{children}</main>
        </div>
      </div>
    )
  }

  return (
    <>
      {isFreshLogin && <WorkspaceLoader />}
      <div 
        className={cn(
          "flex min-h-screen w-full transition-opacity duration-1000",
          isFreshLogin ? "opacity-0 h-screen overflow-hidden" : "opacity-100",
          "bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-slate-100 via-slate-50 to-slate-50 dark:from-slate-900 dark:via-background dark:to-background"
        )}
      >
      <Sidebar isCollapsed={isCollapsed} onToggle={() => setIsCollapsed(!isCollapsed)} />
      <div 
        className={cn(
          "flex flex-col w-full transition-[padding] duration-300 ease-in-out",
          isCollapsed ? "sm:pl-[80px]" : "sm:pl-64"
        )}
      >
        <Topbar isCollapsed={isCollapsed} onToggle={() => setIsCollapsed(!isCollapsed)} />
        <main className="flex-1 p-4 sm:p-8 w-full max-w-7xl mx-auto">
          {children}
        </main>
      </div>
    </div>
    </>
  )
}
