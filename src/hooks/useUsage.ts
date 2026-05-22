import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { Usage } from "../types";

interface UsageState {
  usage: Usage | null;
  error: string | null;
  loading: boolean;
}

/**
 * 使用率を管理するフック。
 * - マウント時に `get_usage` コマンドで初期値を取得
 * - 以降は Rust が 60 秒ごとに送る `usage-updated` イベントで更新
 */
export function useUsage(): UsageState {
  const [usage, setUsage] = useState<Usage | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    invoke<Usage>("get_usage")
      .then((u) => setUsage(u))
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));

    listen<Usage>("usage-updated", (event) => {
      setUsage(event.payload);
      setError(null);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => unlisten?.();
  }, []);

  return { usage, error, loading };
}
