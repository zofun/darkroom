export type CropMode = "fill" | "fit";
export type WatermarkStyle = "engraved" | "siderail" | "cornernote" | "footerband";
export type WmPosition = "br" | "bl" | "tr" | "tl";
export type SizePreset = "5" | "6" | "7" | "custom";

export interface PrintSize {
  widthMm: number;
  heightMm: number;
  dpi: number;
}

export interface WatermarkConfig {
  style: WatermarkStyle;
  showDate: boolean;
  showLocation: boolean;
  showTime: boolean;
  dateFormat: string;
  font: string;
  colorToken: string;
  readability: "none" | "shadow" | "plate";
  scale: number;
  position: WmPosition;
}

export interface ResolvedSettings {
  size: PrintSize;
  cropMode: CropMode;
  borderEnabled: boolean;
  borderColor: [number, number, number];
  borderWidthMm: number;
  watermark: WatermarkConfig;
  quality: number;
  format: "jpeg" | "png";
}

export interface PhotoMeta {
  path: string;
  width: number;
  height: number;
  datetime?: string | null;
  place?: string | null;
  orientation: number;
  error?: string | null;
}

export interface Photo {
  id: string;
  path: string;
  width: number;
  height: number;
  datetime?: string;
  place?: string;
  orientation: number;
  thumb?: string;
  status: "ok" | "missing-info" | "error";
  error?: string;
  overrideDate?: string;
  overridePlace?: string;
  overrideCrop?: CropMode;
  cropFocus?: [number, number];
}

export interface BatchItem {
  input: string;
  date: string;
  place: string;
  orientation: number;
  sizeLabel: string;
}

export interface ExportProgress {
  done: number;
  total: number;
  current: string;
  failed: number;
}
