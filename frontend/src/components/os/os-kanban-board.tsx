"use client"

import { useState, useEffect, useMemo } from "react"
import { DragDropContext, Droppable, Draggable, DropResult } from "@hello-pangea/dnd"
import { Card, CardContent } from "@/components/ui/card"
import { AlertCircle, CheckCircle2, Clock, Wrench } from "lucide-react"
import { osService, type OrdemServico, type StatusOS } from "@/services/os.service"

type WorkOrder = {
  id: string
  title: string
  customer: string
  slaStatus: "on_track" | "at_risk" | "breached"
  technician: string
  timeRemaining: string
  rawStatus: StatusOS
}

type Column = {
  id: string
  title: string
  items: WorkOrder[]
}

/**
 * Colunas alinhadas ao enum Rust do backend (StatusOS):
 *   Aberta / Orcamento / Aprovada / EmAndamento / AguardandoPeca / Finalizada / Cancelada
 */
const OS_COLUMNS: Record<string, Column> = {
  Aberta: { id: 'Aberta', title: 'Abertas', items: [] },
  Orcamento: { id: 'Orcamento', title: 'Orçamento', items: [] },
  Aprovada: { id: 'Aprovada', title: 'Aprovadas', items: [] },
  EmAndamento: { id: 'EmAndamento', title: 'Em Andamento', items: [] },
  AguardandoPeca: { id: 'AguardandoPeca', title: 'Aguardando Peça', items: [] },
  Finalizada: { id: 'Finalizada', title: 'Finalizadas', items: [] },
  Cancelada: { id: 'Cancelada', title: 'Canceladas', items: [] },
}

/** Mapa heurístico de `prazo_entrega` → status de SLA. */
function deriveSla(os: OrdemServico): WorkOrder['slaStatus'] {
  // Se o back expor sla explícito no histórico, podemos usar — por ora,
  // derivamos a partir de prazo vs hoje.
  if (!os.prazo_entrega) return 'on_track'
  try {
    const prazo = new Date(os.prazo_entrega).getTime()
    const agora = Date.now()
    if (prazo < agora) return 'breached'
    if (prazo - agora < 1000 * 60 * 60 * 24) return 'at_risk' // <24h
    return 'on_track'
  } catch {
    return 'on_track'
  }
}

function deriveTimeRemaining(os: OrdemServico): string {
  if (!os.prazo_entrega) return 'Sem prazo'
  try {
    const prazo = new Date(os.prazo_entrega).getTime()
    const diff = prazo - Date.now()
    if (diff < 0) {
      const dias = Math.abs(Math.floor(diff / 86_400_000))
      return dias === 0 ? 'Atrasado' : `Atrasado ${dias}d`
    }
    const dias = Math.floor(diff / 86_400_000)
    if (dias > 0) return `${dias}d restantes`
    const horas = Math.floor(diff / 3_600_000)
    return horas > 0 ? `${horas}h restantes` : 'Hoje'
  } catch {
    return '—'
  }
}

function osToWorkOrder(os: OrdemServico): WorkOrder {
  // Cliente: o BACKEND retorna {cliente: "string"} (nome direto),
  // então usamos `os.cliente` como fonte primária.
  const customer = os.cliente || 'Cliente Sem Nome'
  return {
    id: String(os.id),
    title: os.defeito_relatado || os.equipamento || `OS #${os.id}`,
    customer,
    slaStatus: deriveSla(os),
    technician: os.nome_tecnico_responsavel || 'Não atribuído',
    timeRemaining: deriveTimeRemaining(os),
    rawStatus: os.status,
  }
}

export function OsKanbanBoard() {
  const [columns, setColumns] = useState<Record<string, Column>>(OS_COLUMNS)
  const [isBrowser, setIsBrowser] = useState(false)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    setIsBrowser(true)
    let cancelado = false
    async function loadWorkOrders() {
      try {
        setLoading(true)
        const osList = await osService.listar()
        if (cancelado) return

        // Inicializa colunas vazias (clone raso).
        const newCols: Record<string, Column> = {}
        for (const k of Object.keys(OS_COLUMNS)) {
          newCols[k] = { ...OS_COLUMNS[k], items: [] }
        }
        for (const os of osList) {
          const wo = osToWorkOrder(os)
          const col = newCols[wo.rawStatus]
          if (col) col.items.push(wo)
          else newCols.Aberta.items.push(wo)
        }
        setColumns(newCols)
      } catch (err) {
        console.error('[os-kanban] erro:', err)
      } finally {
        if (!cancelado) setLoading(false)
      }
    }
    loadWorkOrders()
    return () => {
      cancelado = true
    }
  }, [])

  const onDragEnd = (result: DropResult) => {
    if (!result.destination) return
    const { source, destination } = result

    if (source.droppableId === destination.droppableId) {
      const column = columns[source.droppableId]
      const copiedItems = [...column.items]
      const [removed] = copiedItems.splice(source.index, 1)
      copiedItems.splice(destination.index, 0, removed)

      setColumns({
        ...columns,
        [source.droppableId]: { ...column, items: copiedItems },
      })
      return
    }

    // Movendo entre colunas
    const sourceColumn = columns[source.droppableId]
    const destColumn = columns[destination.droppableId]
    if (!sourceColumn || !destColumn) return

    const sourceItems = [...sourceColumn.items]
    const destItems = [...destColumn.items]
    const [removed] = sourceItems.splice(source.index, 1)

    if (!removed) return
    destItems.splice(destination.index, 0, removed)

    setColumns({
      ...columns,
      [source.droppableId]: { ...sourceColumn, items: sourceItems },
      [destination.droppableId]: { ...destColumn, items: destItems },
    })

    // Persistência: o back ainda não expõe POST /os/{id}/mover (BACKEND_MAPPING §4).
    // Por ora: log + console.warn avisando. Hookar `osService.mover(id, { transicao_id })`
    // quando o endpoint subir.
    const movedOs = removed
    console.warn(
      `[os-kanban] OS #${movedOs.id} movida de "${source.droppableId}" → "${destination.droppableId}". ` +
        `Persistência ainda não liberada (endpoint /os/{id}/mover pendente no back).`,
    )
  }

  if (!isBrowser) return null

  if (loading) {
    return (
      <div className="flex h-full w-full gap-4 overflow-x-auto pb-4">
        {[1, 2, 3, 4, 5, 6, 7].map((i) => (
          <div
            key={i}
            className="flex min-w-[280px] flex-col rounded-xl bg-muted/30 p-4 animate-pulse"
          >
            <div className="h-6 w-1/2 bg-white/10 rounded mb-4" />
            <div className="h-32 bg-white/5 rounded" />
          </div>
        ))}
      </div>
    )
  }

  return (
    <div className="flex h-full w-full overflow-x-auto pb-4 gap-4">
      <DragDropContext onDragEnd={onDragEnd}>
        {Object.entries(columns).map(([columnId, column]) => (
          <div
            key={columnId}
            className="flex min-w-[280px] flex-col rounded-xl bg-muted/40 p-4"
          >
            <div className="mb-4 flex items-center justify-between">
              <h3 className="font-semibold text-foreground">{column.title}</h3>
              <span className="flex h-6 w-6 items-center justify-center rounded-full bg-background text-xs font-semibold shadow-sm">
                {column.items.length}
              </span>
            </div>

            <Droppable droppableId={columnId}>
              {(provided, snapshot) => (
                <div
                  {...provided.droppableProps}
                  ref={provided.innerRef}
                  className={`flex flex-1 flex-col gap-3 rounded-lg ${
                    snapshot.isDraggingOver ? 'bg-muted/60' : ''
                  } transition-colors min-h-[180px]`}
                >
                  {column.items.length === 0 ? (
                    <div className="flex flex-1 items-center justify-center border border-dashed border-muted-foreground/20 rounded-lg p-6 text-xs text-muted-foreground text-center">
                      Nenhuma OS nesta fase
                    </div>
                  ) : (
                    column.items.map((item, index) => (
                      <Draggable
                        key={item.id}
                        draggableId={item.id}
                        index={index}
                      >
                        {(provided, snapshot) => (
                          <div
                            ref={provided.innerRef}
                            {...provided.draggableProps}
                            {...provided.dragHandleProps}
                            style={{ ...provided.draggableProps.style }}
                          >
                            <Card
                              className={`group shadow-sm hover:shadow-md transition-shadow backdrop-blur-md bg-white/5 border-white/10 border-l-4 ${
                                item.slaStatus === 'breached'
                                  ? 'border-l-rose-500'
                                  : item.slaStatus === 'at_risk'
                                    ? 'border-l-amber-500'
                                    : 'border-l-emerald-500'
                              } ${
                                snapshot.isDragging
                                  ? 'rotate-2 scale-105 border-primary/50'
                                  : ''
                              }`}
                            >
                              <CardContent className="p-4 flex flex-col gap-3">
                                <div className="flex justify-between items-start gap-2">
                                  <div className="space-y-1 min-w-0">
                                    <div className="text-xs font-semibold text-cyan-400">
                                      OS #{item.id}
                                    </div>
                                    <div className="font-medium leading-tight text-foreground truncate">
                                      {item.title}
                                    </div>
                                  </div>
                                  <div
                                    className={`flex shrink-0 items-center gap-1 rounded-md px-2 py-1 text-xs font-medium ${
                                      item.slaStatus === 'breached'
                                        ? 'bg-rose-500/20 text-rose-300 border border-rose-500/30'
                                        : item.slaStatus === 'at_risk'
                                          ? 'bg-amber-500/20 text-amber-300 border border-amber-500/30'
                                          : 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30'
                                    }`}
                                  >
                                    {item.slaStatus === 'breached' ? (
                                      <AlertCircle className="h-3 w-3" />
                                    ) : item.slaStatus === 'at_risk' ? (
                                      <Clock className="h-3 w-3" />
                                    ) : (
                                      <CheckCircle2 className="h-3 w-3" />
                                    )}
                                    {item.timeRemaining}
                                  </div>
                                </div>

                                <div className="text-sm text-muted-foreground mt-1 truncate">
                                  {item.customer}
                                </div>

                                <div className="flex items-center justify-between pt-3 border-t border-white/10 mt-1">
                                  <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground min-w-0">
                                    <Wrench className="h-3.5 w-3.5 text-cyan-400 shrink-0" />
                                    <span className="truncate">
                                      {item.technician}
                                    </span>
                                  </div>
                                </div>
                              </CardContent>
                            </Card>
                          </div>
                        )}
                      </Draggable>
                    ))
                  )}
                  {provided.placeholder}
                </div>
              )}
            </Droppable>
          </div>
        ))}
      </DragDropContext>
    </div>
  )
}
