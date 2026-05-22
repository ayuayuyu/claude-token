import { useUsage } from "./hooks/useUsage";
import { UsageCard } from "./components/UsageCard";
import "./styles.css";

function App() {
  const { usage, error, loading } = useUsage();

  return (
    // data-tauri-drag-region: 枠なしウィンドウをドラッグで移動できるようにする。
    <main className="widget" data-tauri-drag-region>
      <header className="title" data-tauri-drag-region>
        Claude Usage
      </header>

      {loading && <p className="status">読み込み中…</p>}
      {!loading && error && <p className="status error">⚠ {error}</p>}
      {!loading && !error && usage && <UsageCard usage={usage} />}
    </main>
  );
}

export default App;
