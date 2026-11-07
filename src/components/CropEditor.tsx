import { useRef, useState } from "react";

function clamp(n: number) {
  return Math.min(1, Math.max(0, n));
}

export function CropEditor({
  proxy,
  targetAR,
  targetW,
  targetH,
  sourceW,
  sourceH,
  focus,
  onFocus,
}: {
  proxy?: string;
  targetAR: number;
  targetW: number;
  targetH: number;
  sourceW: number;
  sourceH: number;
  focus: [number, number];
  onFocus: (x: number, y: number) => void;
}) {
  const [srcAR, setSrcAR] = useState<number | null>(null);
  const ref = useRef<HTMLDivElement>(null);
  const drag = useRef<{
    px: number;
    py: number;
    fx: number;
    fy: number;
    w: number;
    h: number;
  } | null>(null);

  if (!proxy) {
    return (
      <div className="crop-stage">
        <div className="stage-loading">载入裁切视图…</div>
      </div>
    );
  }

  const ar = srcAR ?? targetAR;
  const cropWf = ar > targetAR ? targetAR / ar : 1;
  const cropHf = ar > targetAR ? 1 : ar / targetAR;
  const noCrop = cropWf > 0.999 && cropHf > 0.999;

  const xPct = (1 - cropWf) * focus[0] * 100;
  const yPct = (1 - cropHf) * focus[1] * 100;
  const wPct = cropWf * 100;
  const hPct = cropHf * 100;

  const axis = ar > targetAR ? "宽度" : "高度";
  const lostPct = Math.round((ar > targetAR ? 1 - cropWf : 1 - cropHf) * 100);

  return (
    <div className="crop-stage">
      <div
        className="crop-canvas"
        ref={ref}
        style={{ cursor: noCrop ? "default" : "grab", touchAction: "none" }}
        onPointerDown={(e) => {
          if (noCrop) return;
          const el = ref.current;
          if (!el) return;
          el.setPointerCapture(e.pointerId);
          const r = el.getBoundingClientRect();
          drag.current = {
            px: e.clientX,
            py: e.clientY,
            fx: focus[0],
            fy: focus[1],
            w: r.width,
            h: r.height,
          };
        }}
        onPointerMove={(e) => {
          const d = drag.current;
          if (!d) return;
          const dx = (e.clientX - d.px) / d.w;
          const dy = (e.clientY - d.py) / d.h;
          let fx = d.fx;
          let fy = d.fy;
          if (cropWf < 1) fx = clamp(d.fx + dx / (1 - cropWf));
          if (cropHf < 1) fy = clamp(d.fy + dy / (1 - cropHf));
          onFocus(fx, fy);
        }}
        onPointerUp={() => {
          drag.current = null;
        }}
        onPointerCancel={() => {
          drag.current = null;
        }}
      >
        <img
          className="crop-img"
          src={proxy}
          alt="原图"
          draggable={false}
          onLoad={(e) => {
            const img = e.currentTarget;
            setSrcAR(img.naturalWidth / img.naturalHeight);
          }}
        />
        <div
          className="crop-dim"
          style={{ left: 0, top: 0, width: "100%", height: `${yPct}%` }}
        />
        <div
          className="crop-dim"
          style={{
            left: 0,
            top: `${yPct + hPct}%`,
            width: "100%",
            height: `${100 - yPct - hPct}%`,
          }}
        />
        <div
          className="crop-dim"
          style={{
            left: 0,
            top: `${yPct}%`,
            width: `${xPct}%`,
            height: `${hPct}%`,
          }}
        />
        <div
          className="crop-dim"
          style={{
            left: `${xPct + wPct}%`,
            top: `${yPct}%`,
            width: `${100 - xPct - wPct}%`,
            height: `${hPct}%`,
          }}
        />
        <div
          className="crop-border"
          style={{
            left: `${xPct}%`,
            top: `${yPct}%`,
            width: `${wPct}%`,
            height: `${hPct}%`,
          }}
        />
      </div>

      <div className="crop-stats">
        {noCrop ? (
          <span>比例一致，无需裁切</span>
        ) : (
          <span>
            裁掉{axis} <b>{lostPct}%</b> · 拖动选择保留区域
          </span>
        )}
        <span className="crop-sub">
          原图 {sourceW}×{sourceH} → 成品 {targetW}×{targetH}
        </span>
      </div>
    </div>
  );
}
