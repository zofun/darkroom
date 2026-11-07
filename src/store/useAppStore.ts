import { create } from "zustand";
import type {
  CropMode,
  Photo,
  ResolvedSettings,
  SizePreset,
  WatermarkConfig,
} from "../types";

export interface AppSettings {
  sizePreset: SizePreset;
  customWidthMm: number;
  customHeightMm: number;
  dpi: number;
  cropMode: CropMode;
  borderColor: string;
  watermark: WatermarkConfig;
  exportFormat: "jpeg" | "png";
  quality: number;
  outputDir: string;
}

const MONTHS = [
  "一月",
  "二月",
  "三月",
  "四月",
  "五月",
  "六月",
  "七月",
  "八月",
  "九月",
  "十月",
  "十一月",
  "十二月",
];

export function prettyDate(raw?: string): string {
  if (!raw) return "";
  const m = raw.match(/(\d{4})[:-](\d{2})[:-](\d{2})/);
  if (!m) return raw;
  const month = MONTHS[Number(m[2]) - 1] ?? "";
  return `${m[1]} · ${month}`;
}

export function hexToRgb(hex: string): [number, number, number] {
  const h = hex.replace("#", "");
  const full =
    h.length === 3
      ? h
          .split("")
          .map((c) => c + c)
          .join("")
      : h;
  const n = parseInt(full, 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

export function sizeMm(
  preset: SizePreset,
  customW: number,
  customH: number
): { widthMm: number; heightMm: number } {
  switch (preset) {
    case "5":
      return { widthMm: 88.9, heightMm: 127.0 };
    case "7":
      return { widthMm: 127.0, heightMm: 177.8 };
    case "custom":
      return { widthMm: customW, heightMm: customH };
    default:
      return { widthMm: 101.6, heightMm: 152.4 };
  }
}

const defaultWatermark: WatermarkConfig = {
  style: "cornernote",
  showDate: true,
  showLocation: true,
  showTime: false,
  dateFormat: "YYYY · M月",
  font: "Songti",
  colorToken: "warm",
  scale: 1,
  position: "bl",
};

const defaultSettings: AppSettings = {
  sizePreset: "6",
  customWidthMm: 101.6,
  customHeightMm: 152.4,
  dpi: 300,
  cropMode: "fill",
  borderColor: "#ffffff",
  watermark: defaultWatermark,
  exportFormat: "jpeg",
  quality: 92,
  outputDir: "",
};

interface AppState {
  photos: Photo[];
  selectedId: string | null;
  settings: AppSettings;
  setPhotos: (photos: Photo[]) => void;
  addPhotos: (photos: Photo[]) => void;
  select: (id: string) => void;
  updatePhoto: (id: string, patch: Partial<Photo>) => void;
  setSettings: (patch: Partial<AppSettings>) => void;
  setWatermark: (patch: Partial<WatermarkConfig>) => void;
  applyInfoToAll: () => void;
  effectiveDate: (id: string) => string;
  effectivePlace: (id: string) => string;
  effectiveCrop: (id: string) => CropMode;
  resolved: (id: string | null) => ResolvedSettings;
  sizeLabel: () => string;
}

export const useAppStore = create<AppState>((set, get) => ({
  photos: [],
  selectedId: null,
  settings: defaultSettings,

  setPhotos: (photos) =>
    set({ photos, selectedId: photos[0]?.id ?? null }),

  addPhotos: (photos) =>
    set((s) => ({
      photos: [...s.photos, ...photos],
      selectedId: s.selectedId ?? photos[0]?.id ?? null,
    })),

  select: (id) => set({ selectedId: id }),

  updatePhoto: (id, patch) =>
    set((s) => ({
      photos: s.photos.map((p) => (p.id === id ? { ...p, ...patch } : p)),
    })),

  setSettings: (patch) => set((s) => ({ settings: { ...s.settings, ...patch } })),

  setWatermark: (patch) =>
    set((s) => ({
      settings: {
        ...s.settings,
        watermark: { ...s.settings.watermark, ...patch },
      },
    })),

  applyInfoToAll: () => {
    const { photos, selectedId } = get();
    if (!selectedId) return;
    const date = get().effectiveDate(selectedId);
    const place = get().effectivePlace(selectedId);
    set({
      photos: photos.map((p) => ({
        ...p,
        overrideDate: date,
        overridePlace: place,
      })),
    });
  },

  effectiveDate: (id) => {
    const p = get().photos.find((x) => x.id === id);
    if (!p) return "";
    return p.overrideDate !== undefined ? p.overrideDate : prettyDate(p.datetime);
  },

  effectivePlace: (id) => {
    const p = get().photos.find((x) => x.id === id);
    if (!p) return "";
    return p.overridePlace !== undefined ? p.overridePlace : p.place ?? "";
  },

  effectiveCrop: (id) => {
    const p = get().photos.find((x) => x.id === id);
    return p?.overrideCrop ?? get().settings.cropMode;
  },

  resolved: (id) => {
    const s = get().settings;
    const { widthMm, heightMm } = sizeMm(
      s.sizePreset,
      s.customWidthMm,
      s.customHeightMm
    );
    const cropMode = id ? get().effectiveCrop(id) : s.cropMode;
    return {
      size: { widthMm, heightMm, dpi: s.dpi },
      cropMode,
      borderEnabled: cropMode === "fit",
      borderColor: hexToRgb(s.borderColor),
      borderWidthMm: 0,
      watermark: s.watermark,
      quality: s.quality,
      format: s.exportFormat,
    };
  },

  sizeLabel: () => {
    const s = get().settings;
    if (s.sizePreset === "custom") {
      return `${Math.round(s.customWidthMm)}×${Math.round(
        s.customHeightMm
      )}mm`;
    }
    return `${s.sizePreset}寸`;
  },
}));
