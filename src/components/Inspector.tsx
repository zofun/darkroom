import type { ReactNode } from "react";
import { useAppStore } from "../store/useAppStore";
import { api } from "../lib/api";
import type { CropMode, WatermarkStyle } from "../types";

const STYLES: { id: WatermarkStyle; name: string; sub: string }[] = [
  { id: "engraved", name: "宋刻", sub: "右下 · 衬线细线" },
  { id: "siderail", name: "侧栏", sub: "右侧竖排" },
  { id: "cornernote", name: "角注", sub: "左下 · 默认" },
  { id: "footerband", name: "底注", sub: "底部压暗条" },
];

const COLORS = [
  { token: "warm", hex: "#f4e8ca" },
  { token: "gold", hex: "#c8a96a" },
  { token: "paper", hex: "#e8e2d6" },
  { token: "ink", hex: "#1c1a17" },
];

function Toggle({
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

export function Inspector({
  onExport,
  onCancel,
  exporting,
}: {
  onExport: () => void;
  onCancel: () => void;
  exporting: boolean;
}) {
  const photos = useAppStore((s) => s.photos);
  const selectedId = useAppStore((s) => s.selectedId);
  const settings = useAppStore((s) => s.settings);
  const setSettings = useAppStore((s) => s.setSettings);
  const setWatermark = useAppStore((s) => s.setWatermark);
  const updatePhoto = useAppStore((s) => s.updatePhoto);
  const applyInfoToAll = useAppStore((s) => s.applyInfoToAll);
  const effectiveDate = useAppStore((s) => s.effectiveDate);
  const effectivePlace = useAppStore((s) => s.effectivePlace);

  const selected = photos.find((p) => p.id === selectedId);
  const hasSelection = Boolean(selected);
  const date = selected ? effectiveDate(selected.id) : "";
  const place = selected ? effectivePlace(selected.id) : "";

  const chooseOutput = async () => {
    const dir = await api.pickFolder();
    if (dir) setSettings({ outputDir: dir });
  };

  const cropMode: CropMode = settings.cropMode;

  return (
    <aside className="inspector">
      <div className="sec">
        <h4>输出尺寸</h4>
        <div className="seg" style={{ marginBottom: 8 }}>
          {(["5", "6", "7", "custom"] as const).map((k) => (
            <button
              key={k}
              className={settings.sizePreset === k ? "on" : ""}
              onClick={() => setSettings({ sizePreset: k })}
            >
              {k === "custom" ? "自定义" : `${k}寸`}
            </button>
          ))}
        </div>
        {settings.sizePreset === "custom" && (
          <div className="row">
            <input
              className="field narrow"
              type="number"
              value={settings.customWidthMm}
              onChange={(e) =>
                setSettings({ customWidthMm: Number(e.target.value) || 0 })
              }
            />
            <span className="k">×</span>
            <input
              className="field narrow"
              type="number"
              value={settings.customHeightMm}
              onChange={(e) =>
                setSettings({ customHeightMm: Number(e.target.value) || 0 })
              }
            />
            <span className="k">mm</span>
          </div>
        )}
        <div className="row">
          <span className="k">DPI</span>
          <input
            className="field narrow"
            type="number"
            value={settings.dpi}
            onChange={(e) =>
              setSettings({ dpi: Number(e.target.value) || 300 })
            }
          />
        </div>
      </div>

      <div className="sec">
        <h4>裁切与留白</h4>
        <div className="seg">
          <button
            className={cropMode === "fill" ? "on" : ""}
            onClick={() => setSettings({ cropMode: "fill" })}
          >
            填充裁切
          </button>
          <button
            className={cropMode === "fit" ? "on" : ""}
            onClick={() => setSettings({ cropMode: "fit" })}
          >
            完整保留
          </button>
        </div>
        {cropMode === "fit" && (
          <div className="row" style={{ marginTop: 8 }}>
            <span className="k">白边颜色</span>
            <input
              type="color"
              value={settings.borderColor}
              onChange={(e) => setSettings({ borderColor: e.target.value })}
              style={{ width: 38, height: 24, background: "none", border: "1px solid #3a3a3f" }}
            />
          </div>
        )}
      </div>

      <div className="sec">
        <h4>时间 / 位置</h4>
        <div className="row">
          <span className="k">时间</span>
          <input
            className="field"
            value={date}
            placeholder="缺失，手动输入"
            disabled={!hasSelection}
            onChange={(e) =>
              selected &&
              updatePhoto(selected.id, { overrideDate: e.target.value })
            }
          />
        </div>
        <div className="row">
          <span className="k">位置</span>
          <input
            className="field"
            value={place}
            placeholder="缺失，手动输入"
            disabled={!hasSelection}
            onChange={(e) =>
              selected &&
              updatePhoto(selected.id, { overridePlace: e.target.value })
            }
          />
        </div>
        <div className="row">
          <Toggle
            on={settings.watermark.showDate}
            onClick={() =>
              setWatermark({ showDate: !settings.watermark.showDate })
            }
          >
            显示时间
          </Toggle>
          <Toggle
            on={settings.watermark.showLocation}
            onClick={() =>
              setWatermark({ showLocation: !settings.watermark.showLocation })
            }
          >
            显示位置
          </Toggle>
        </div>
        <button
          className="btn tiny"
          disabled={!hasSelection}
          onClick={() => applyInfoToAll()}
        >
          当前时间/位置套用到全部
        </button>
      </div>

      <div className="sec">
        <h4>水印样式</h4>
        <div className="styles">
          {STYLES.map((st) => (
            <button
              key={st.id}
              className={
                "style-btn" + (settings.watermark.style === st.id ? " on" : "")
              }
              onClick={() => setWatermark({ style: st.id })}
            >
              {st.name}
              <small>{st.sub}</small>
            </button>
          ))}
        </div>
        <div className="row" style={{ marginTop: 10 }}>
          <span className="k">颜色</span>
          <span className="swatches">
            {COLORS.map((c) => (
              <button
                key={c.token}
                className={
                  "sw" + (settings.watermark.colorToken === c.token ? " on" : "")
                }
                style={{ background: c.hex }}
                onClick={() => setWatermark({ colorToken: c.token })}
              />
            ))}
          </span>
        </div>
        <div className="row">
          <span className="k">大小</span>
          <input
            type="range"
            min={0.6}
            max={1.8}
            step={0.05}
            value={settings.watermark.scale}
            onChange={(e) =>
              setWatermark({ scale: Number(e.target.value) })
            }
            style={{ flex: 1, accentColor: "#c8a96a" }}
          />
        </div>
      </div>

      <div className="sec">
        <h4>导出</h4>
        <div className="row">
          <span className="k">格式</span>
          <span className="seg" style={{ width: 120 }}>
            <button
              className={settings.exportFormat === "jpeg" ? "on" : ""}
              onClick={() => setSettings({ exportFormat: "jpeg" })}
            >
              JPEG
            </button>
            <button
              className={settings.exportFormat === "png" ? "on" : ""}
              onClick={() => setSettings({ exportFormat: "png" })}
            >
              PNG
            </button>
          </span>
        </div>
        {settings.exportFormat === "jpeg" && (
          <div className="row">
            <span className="k">质量</span>
            <input
              type="range"
              min={60}
              max={100}
              value={settings.quality}
              onChange={(e) => setSettings({ quality: Number(e.target.value) })}
              style={{ flex: 1, accentColor: "#c8a96a" }}
            />
            <span className="k">{settings.quality}</span>
          </div>
        )}
        <div className="row">
          <span className="k">输出</span>
          <button className="btn tiny" onClick={chooseOutput}>
            {settings.outputDir ? "更换文件夹" : "选择文件夹"}
          </button>
        </div>
        <div className="muted-note">
          {settings.outputDir || "尚未选择输出文件夹（导出时会提示）"}
        </div>
      </div>

      <div className="export-box">
        {exporting ? (
          <button className="btn danger" onClick={onCancel}>
            取消导出
          </button>
        ) : (
          <button
            className="btn primary"
            disabled={photos.length === 0}
            onClick={onExport}
          >
            导出到文件夹
          </button>
        )}
      </div>
    </aside>
  );
}
