import { useAppStore } from "../store/useAppStore";

export function FilmStrip() {
  const photos = useAppStore((s) => s.photos);
  const selectedId = useAppStore((s) => s.selectedId);
  const select = useAppStore((s) => s.select);

  return (
    <div className="film">
      <div className="film-head">
        <span>ROLL</span>
        <span>{String(photos.length).padStart(2, "0")}</span>
      </div>
      <div className="film-list">
        {photos.length === 0 && <div className="film-empty">未导入</div>}
        {photos.map((p, i) => (
          <button
            key={p.id}
            className={"thumb" + (p.id === selectedId ? " selected" : "")}
            onClick={() => select(p.id)}
            title={p.path}
          >
            {p.thumb ? (
              <img src={p.thumb} alt="" />
            ) : (
              <div className="thumb-ph" />
            )}
            <span className="thumb-idx">
              {String(i + 1).padStart(2, "0")}
            </span>
            {p.status === "missing-info" && (
              <span className="thumb-flag" title="缺少时间/地点" />
            )}
            {p.status === "error" && (
              <span className="thumb-flag err" title={p.error} />
            )}
          </button>
        ))}
      </div>
    </div>
  );
}
