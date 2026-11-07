import { invoke } from "@tauri-apps/api/core";
import type { BatchItem, PhotoMeta, ResolvedSettings } from "../types";

export const api = {
  pickFolder: () => invoke<string | null>("pick_folder"),
  listPhotos: (dir: string) => invoke<string[]>("list_photos", { dir }),
  readMeta: (path: string) => invoke<PhotoMeta>("read_photo_meta", { path }),
  makeThumb: (path: string, max: number) =>
    invoke<string>("make_thumb", { path, max }),
  renderPreview: (args: {
    path: string;
    settings: ResolvedSettings;
    date: string;
    place: string;
    orientation: number;
    maxDim: number;
  }) => invoke<string>("render_preview", args),
  exportBatch: (
    items: BatchItem[],
    settings: ResolvedSettings,
    outDir: string
  ) => invoke<string[]>("export_batch", { items, settings, outDir }),
  cancel: () => invoke<void>("cancel"),
};
