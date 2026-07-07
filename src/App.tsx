import { Button } from "@/components/ui/button";

function App() {
  return (
    <div className="flex h-screen bg-background text-foreground">
      <aside className="flex w-60 shrink-0 flex-col border-r bg-sidebar">
        <div className="flex items-center gap-2 px-4 py-3">
          <img src="/logo.svg" alt="" className="size-6 rounded" />
          <span className="text-sm font-semibold">Inkstone</span>
        </div>
        <div className="flex-1 overflow-y-auto px-2 py-1">
          {/* 文档列表（M1） */}
          <p className="px-2 py-8 text-center text-xs text-muted-foreground">
            还没有文档
          </p>
        </div>
      </aside>
      <main className="flex flex-1 items-center justify-center">
        <div className="flex flex-col items-center gap-4">
          <img src="/logo.svg" alt="Inkstone" className="size-16 rounded-2xl" />
          <p className="text-sm text-muted-foreground">
            选择左侧文档，或从这里开始
          </p>
          <Button disabled>新建文档</Button>
        </div>
      </main>
    </div>
  );
}

export default App;
