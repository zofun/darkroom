export function CanvasPreview({
  preview,
  loading,
}: {
  preview?: string;
  loading: boolean;
}) {
  return (
    <div className="stage">
      {preview ? (
        <img className="preview" src={preview} alt="预览" />
      ) : (
        <div className="stage-empty">
          <div className="stage-mark">DARKROOM</div>
          <p>导入照片，开始冲印</p>
        </div>
      )}
      {loading && <div className="stage-loading">冲印中…</div>}
    </div>
  );
}
