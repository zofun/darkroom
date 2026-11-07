import { useCallback, useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { listen } from "@tauri-apps/api/event";
import { DEFAULT_FOCUS, useAppStore } from "./store/useAppStore";
import { api } from "./lib/api";
import { FilmStrip } from "./components/FilmStrip";
import { CanvasPreview } from "./components/CanvasPreview";
import { CropEditor } from "./components/CropEditor";
import { Inspector } from "./components/Inspector";
import type { ExportProgress, Photo, WatermarkStyle } from "./types";

function baseName(p: string) {
  const parts = p.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || p;
}

function ToolToggle({
  on,
  onClick,
  children,
}: {
  on: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button className={"tgl" + (on ? " on" : "")} onClick={onClick}>
      <span className="dot" />
      {children}
    </button>
  );
}

export default function App() {
  const photos = useAppStore((s) => s.photos);
  const selectedId = useAppStore((s) => s.selectedId);
  const settings = useAppStore((s) => s.settings);
  const setPhotos = useAppStore((s) => s.setPhotos);
  const setSettings = useAppStore((s) => s.setSettings);
  const setWatermark = useAppStore((s) => s.setWatermark);
  const resolved = useAppStore((s) => s.resolved);
  const effectiveDate = useAppStore((s) => s.effectiveDate);
  const effectivePlace = useAppStore((s) => s.effectivePlace);
  const effectiveFocus = useAppStore((s) => s.effectiveFocus);
  const updatePhoto = useAppStore((s) => s.updatePhoto);
  const sizeLabel = useAppStore((s) => s.sizeLabel);

  const [folder, setFolder] = useState("");
  const [preview, setPreview] = useState<string | undefined>(undefined);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [importing, setImporting] = useState(false);
  const [importCount, setImportCount] = useState(0);
  const [exporting, setExporting] = useState(false);
  const [progress, setProgress] = useState<ExportProgress | null>(null);
  const [message, setMessage] = useState("就绪");
  const [messageKind, setMessageKind] = useState<"ok" | "bad" | "">("ok");
  const [view, setView] = useState<"result" | "crop">("result");
  const [proxy, setProxy] = useState<string | undefined>(undefined);

  useEffect(() => {
    const un = listen<ExportProgress>("export-progress", (e) =>
      setProgress(e.payload)
    );
    return () => {
      un.then((f) => f());
    };
  }, []);

  const selected = photos.find((p) => p.id === selectedId);
  const focus = selected ? effectiveFocus(selected.id) : DEFAULT_FOCUS;

  useEffect(() => {
    if (settings.cropMode === "fit") setView("result");
  }, [settings.cropMode]);

  useEffect(() => {
    let alive = true;
    if (!selected) {
      setProxy(undefined);
      return;
    }
    setProxy(undefined);
    api
      .makeProxy(selected.path, selected.orientation, 1500)
      .then((d) => {
        if (alive) setProxy(d);
      })
      .catch(() => {});
    return () => {
      alive = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selected?.path, selected?.orientation]);

  useEffect(() => {
    let alive = true;
    if (!selected) {
      setPreview(undefined);
      return;
    }
    setPreviewLoading(true);
    const timer = setTimeout(async () => {
      try {
        const data = await api.renderPreview({
          path: selected.path,
          settings: resolved(selected.id),
          date: effectiveDate(selected.id),
          place: effectivePlace(selected.id),
          orientation: selected.orientation,
          maxDim: 1100,
          focus,
        });
        if (alive) setPreview(data);
      } catch (err) {
        if (alive) {
          setMessage("预览失败: " + String(err));
          setMessageKind("bad");
        }
      } finally {
        if (alive) setPreviewLoading(false);
      }
    }, 180);
    return () => {
      alive = false;
      clearTimeout(timer);
    };
  }, [selected, settings, resolved, effectiveDate, effectivePlace, focus]);

  const importFrom = useCallback(async (dir: string) => {
    setFolder(dir);
    setImporting(true);
    setImportCount(0);
    setMessage("读取照片…");
    setMessageKind("");
    try {
      const paths = await api.listPhotos(dir);
      const total = paths.length;
      if (total === 0) {
        setMessage("该文件夹没有支持的照片");
        setMessageKind("bad");
        setImporting(false);
        return;
      }
      const results: (Photo | undefined)[] = new Array(total);
      let next = 0;
      let done = 0;
      const worker = async () => {
        while (next < total) {
          const i = next++;
          const path = paths[i];
          let meta;
          try {
            meta = await api.readMeta(path);
          } catch {
            meta = {
              path,
              width: 0,
              height: 0,
              orientation: 1,
              error: "读取失败",
            };
          }
          let thumb: string | undefined;
          try {
            thumb = await api.makeThumb(path, 260, meta.orientation ?? 1);
          } catch {
            thumb = undefined;
          }
          const missing = !meta.datetime && !meta.place;
          results[i] = {
            id: path,
            path,
            width: meta.width,
            height: meta.height,
            datetime: meta.datetime ?? undefined,
            place: meta.place ?? undefined,
            orientation: meta.orientation ?? 1,
            thumb,
            status: meta.error ? "error" : missing ? "missing-info" : "ok",
            error: meta.error ?? undefined,
          };
          done += 1;
          setImportCount(done);
          setPhotos(results.filter((x): x is Photo => Boolean(x)));
        }
      };
      const workerCount = Math.min(2, total);
      await Promise.all(
        Array.from({ length: workerCount }, () => worker())
      );
      setMessage(`已导入 ${total} 张`);
      setMessageKind("ok");
    } catch (err) {
      setMessage("导入失败: " + String(err));
      setMessageKind("bad");
    } finally {
      setImporting(false);
    }
  }, [setPhotos]);

  const handleImport = async () => {
    const dir = await api.pickFolder();
    if (dir) await importFrom(dir);
  };

  const demoRan = useRef(false);
  useEffect(() => {
    if (demoRan.current) return;
    demoRan.current = true;
    api
      .demoConfig()
      .then(async (cfg) => {
        if (cfg.style) setWatermark({ style: cfg.style as WatermarkStyle });
        if (cfg.readability)
          setWatermark({
            readability: cfg.readability as "none" | "shadow" | "plate",
          });
        if (cfg.view === "crop") setView("crop");
        if (cfg.dir) {
          await importFrom(cfg.dir);
          if (cfg.date || cfg.place) {
            const photosNow = useAppStore.getState().photos;
            photosNow.forEach((p) => {
              useAppStore.getState().updatePhoto(p.id, {
                overrideDate: cfg.date,
                overridePlace: cfg.place,
              });
            });
          }
        }
      })
      .catch(() => {});
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [importFrom, setWatermark]);

  const handleExport = async () => {
    if (photos.length === 0) return;
    let outDir = settings.outputDir;
    if (!outDir) {
      const picked = await api.pickFolder();
      if (!picked) return;
      outDir = picked;
      setSettings({ outputDir: picked });
    }
    const items = photos.map((p) => ({
      input: p.path,
      date: effectiveDate(p.id),
      place: effectivePlace(p.id),
      orientation: p.orientation,
      sizeLabel: sizeLabel(),
      cropFocus: effectiveFocus(p.id),
    }));
    setExporting(true);
    setProgress(null);
    setMessage("导出中…");
    setMessageKind("");
    try {
      const errs = await api.exportBatch(items, resolved(null), outDir);
      if (errs.length > 0) {
        setMessage(`完成，${errs.length} 张失败`);
        setMessageKind("bad");
      } else {
        setMessage(`已导出 ${items.length} 张 → ${outDir}`);
        setMessageKind("ok");
      }
    } catch (err) {
      setMessage("导出失败: " + String(err));
      setMessageKind("bad");
    } finally {
      setExporting(false);
    }
  };

  const handleCancel = () => {
    api.cancel();
  };

  const size = resolved(selected?.id ?? null).size;
  const targetW = Math.round((size.widthMm / 25.4) * size.dpi);
  const targetH = Math.round((size.heightMm / 25.4) * size.dpi);
  const targetAR = size.widthMm / size.heightMm;
  const oriented =
    selected && selected.orientation >= 5 && selected.orientation <= 8
      ? { w: selected.height, h: selected.width }
      : { w: selected?.width ?? 0, h: selected?.height ?? 0 };

  return (
    <div className="app">
      <div className="topbar">
        <div className="brand">
          DARKROOM<small>暗房</small>
        </div>
        <div className="crumb">
          {folder ? (
            <>
              照片库 / <b>{baseName(folder)}</b> · {photos.length} 张
            </>
          ) : (
            "照片库 · 未导入"
          )}
        </div>
        <div className="top-actions">
          <button className="btn" onClick={handleImport} disabled={importing}>
            {importing ? `读取 ${importCount}…` : "导入文件夹"}
          </button>
          <button
            className="btn primary"
            onClick={handleExport}
            disabled={exporting || photos.length === 0}
          >
            {exporting ? "导出中…" : "导出到文件夹"}
          </button>
        </div>
      </div>

      <div className="body">
        <nav className="nav">
          <button className="nav-item active">
            <span className="nav-ic" />
            <span>单张处理</span>
          </button>
          <button className="nav-item">
            <span className="nav-ic" />
            <span>批量处理</span>
          </button>
          <button className="nav-item">
            <span className="nav-ic" />
            <span>预设模板</span>
          </button>
          <button className="nav-item">
            <span className="nav-ic" />
            <span>设置</span>
          </button>
          <button className="nav-item future">
            <span className="nav-badge">SOON</span>
            <span className="nav-ic" />
            <span>排版·相册</span>
          </button>
        </nav>

        <FilmStrip />

        <main className="canvas">
          {view === "crop" && settings.cropMode === "fill" && selected ? (
            <CropEditor
              proxy={proxy}
              targetAR={targetAR}
              targetW={targetW}
              targetH={targetH}
              sourceW={oriented.w}
              sourceH={oriented.h}
              focus={focus}
              onFocus={(x, y) =>
                updatePhoto(selected.id, { cropFocus: [x, y] })
              }
            />
          ) : (
            <CanvasPreview preview={preview} loading={previewLoading} />
          )}
          <div className="toolbar">
            <span className="tool-label">尺寸</span>
            {(["5", "6", "7"] as const).map((k) => (
              <button
                key={k}
                className={"chip" + (settings.sizePreset === k ? " on" : "")}
                onClick={() => setSettings({ sizePreset: k })}
              >
                {k}寸
              </button>
            ))}
            <button
              className={
                "chip" + (settings.sizePreset === "custom" ? " on" : "")
              }
              onClick={() => setSettings({ sizePreset: "custom" })}
            >
              自定义
            </button>
            <div className="spacer" />
            {settings.cropMode === "fill" && selected && (
              <span className="seg" style={{ width: 120, marginRight: 10 }}>
                <button
                  className={view === "result" ? "on" : ""}
                  onClick={() => setView("result")}
                >
                  成品
                </button>
                <button
                  className={view === "crop" ? "on" : ""}
                  onClick={() => setView("crop")}
                >
                  裁切
                </button>
              </span>
            )}
            <ToolToggle
              on={settings.watermark.showDate}
              onClick={() =>
                setWatermark({ showDate: !settings.watermark.showDate })
              }
            >
              时间
            </ToolToggle>
            <ToolToggle
              on={settings.watermark.showLocation}
              onClick={() =>
                setWatermark({
                  showLocation: !settings.watermark.showLocation,
                })
              }
            >
              位置
            </ToolToggle>
          </div>
        </main>

        <Inspector
          onExport={handleExport}
          onCancel={handleCancel}
          exporting={exporting}
        />
      </div>

      <div className="statusbar">
        <span className={messageKind === "ok" ? "ok" : messageKind === "bad" ? "bad" : ""}>
          {message}
        </span>
        <span>{photos.length} 张</span>
        {exporting && progress && (
          <span>
            导出 {progress.done}/{progress.total}
            {progress.failed > 0 ? ` · 失败 ${progress.failed}` : ""}
          </span>
        )}
        <span className="right">
          {settings.outputDir || folder || "未选择输出目录"}
        </span>
      </div>
    </div>
  );
}
