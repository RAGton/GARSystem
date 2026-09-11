export default function OrcamentosPage() {
  return (
    <div className="flex flex-1 flex-col gap-6 p-4 md:p-8">
      <div className="flex flex-col gap-2">
        <h1 className="text-3xl font-extrabold tracking-tight">📄 Orçamentos</h1>
        <p className="text-muted-foreground">
          Módulo em construção. Aguardando <code className="font-mono text-xs bg-muted px-1 py-0.5 rounded">GET /orcamentos</code> do backend (ver <code className="font-mono text-xs bg-muted px-1 py-0.5 rounded">docs/openapi.json</code>).
        </p>
      </div>
      <div className="rounded-lg border border-dashed p-12 text-center text-sm text-muted-foreground">
        Aguardando integração (passo B).
      </div>
    </div>
  );
}
