import React from "react"
import { DashboardLayoutClient } from "@/components/layout/dashboard-layout-client"

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <DashboardLayoutClient>
      {children}
    </DashboardLayoutClient>
  )
}
