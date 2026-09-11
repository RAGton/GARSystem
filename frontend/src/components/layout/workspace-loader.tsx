"use client"

import React, { useEffect, useState } from "react"
import { motion, AnimatePresence } from "framer-motion"
import { useAuthStore } from "@/stores/auth.store"

export function WorkspaceLoader() {
  const user = useAuthStore(state => state.user)
  const setFreshLogin = useAuthStore(state => state.setFreshLogin)
  
  const [phase, setPhase] = useState(0) // 0: init, 1: brand, 2: loading, 3: welcome, 4: exit
  
  useEffect(() => {
    // Sequence the phases
    const t1 = setTimeout(() => setPhase(1), 200) // Show logo
    const t2 = setTimeout(() => setPhase(2), 1200) // Show loaders
    const t3 = setTimeout(() => setPhase(3), 2800) // Show welcome
    const t4 = setTimeout(() => setPhase(4), 4500) // Exit animation
    const t5 = setTimeout(() => setFreshLogin(false), 5000) // Unmount loader

    return () => {
      clearTimeout(t1)
      clearTimeout(t2)
      clearTimeout(t3)
      clearTimeout(t4)
      clearTimeout(t5)
    }
  }, [setFreshLogin])

  return (
    <AnimatePresence>
      {phase < 4 && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0, scale: 1.05 }}
          transition={{ duration: 0.6, ease: [0.22, 1, 0.36, 1] }}
          className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-[#0A0F1E] text-white overflow-hidden"
        >
          {/* Subtle glow background */}
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] rounded-full bg-[#38BDF8]/5 blur-[120px] pointer-events-none" />

          {/* PHASE 1 & 2: Branding & Loading */}
          <AnimatePresence mode="wait">
            {phase >= 1 && phase < 3 && (
              <motion.div 
                key="branding-loading"
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -20 }}
                transition={{ duration: 0.5 }}
                className="flex flex-col items-center gap-12"
              >
                <div className="text-3xl font-bold tracking-tighter flex items-center gap-2">
                  GARSystem <span className="text-transparent bg-clip-text bg-gradient-to-r from-[#38BDF8] to-[#06B6D4]">Enterprise</span>
                </div>
                
                {phase >= 2 && (
                  <motion.div 
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="flex flex-col gap-3 text-sm text-[#94A3B8]"
                  >
                    <LoadingItem text="Carregando permissões" delay={0} />
                    <LoadingItem text="Carregando módulos" delay={0.2} />
                    <LoadingItem text="Carregando dashboards" delay={0.4} />
                    <LoadingItem text="Carregando preferências" delay={0.6} />
                    <LoadingItem text="Carregando workspace" delay={0.8} />
                  </motion.div>
                )}
              </motion.div>
            )}

            {/* PHASE 3: Welcome */}
            {phase === 3 && (
              <motion.div
                key="welcome"
                initial={{ opacity: 0, scale: 0.95, filter: 'blur(10px)' }}
                animate={{ opacity: 1, scale: 1, filter: 'blur(0px)' }}
                exit={{ opacity: 0, scale: 1.05 }}
                transition={{ duration: 0.5 }}
                className="flex flex-col items-center text-center max-w-md p-8 rounded-2xl backdrop-blur-xl bg-white/5 border border-white/10 shadow-2xl"
              >
                <div className="w-16 h-16 rounded-full bg-gradient-to-tr from-[#38BDF8] to-[#06B6D4] flex items-center justify-center text-2xl font-bold text-white mb-6 shadow-lg shadow-[#38BDF8]/20">
                  {(user?.username?.charAt(0) ?? user?.papel?.charAt(0) ?? 'A').toUpperCase()}
                </div>
                
                <h2 className="text-2xl font-semibold mb-2">
                  Bem-vindo, {user?.papel === 'Administrador' ? 'Administrador' : (user?.username?.split('.')[0] ?? 'Gabriel')}
                </h2>
                
                <div className="flex flex-col gap-1 text-sm text-[#94A3B8] mb-6">
                  <p>Perfil: <span className="text-white font-medium capitalize">{user?.papel || 'Padrão'}</span></p>
                  {user?.tenant && <p>Tenant: <span className="text-white font-medium">#{user.tenant}</span></p>}
                  {user?.permissions && user.permissions.length > 0 && (
                    <p>Permissões: <span className="text-white font-medium">{user.permissions.length}</span></p>
                  )}
                </div>
                
                <div className="text-xs text-[#64748B] flex gap-2">
                  <span>Módulos:</span>
                  <span className="text-[#94A3B8]">CRM • Financeiro • OS • Estoque</span>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </motion.div>
      )}
    </AnimatePresence>
  )
}

function LoadingItem({ text, delay }: { text: string; delay: number }) {
  return (
    <motion.div
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ delay, duration: 0.4 }}
      className="flex items-center gap-3"
    >
      <motion.div
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        transition={{ delay: delay + 0.2, type: "spring" }}
        className="text-[#38BDF8]"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
          <polyline points="20 6 9 17 4 12" />
        </svg>
      </motion.div>
      <span>{text}</span>
    </motion.div>
  )
}
