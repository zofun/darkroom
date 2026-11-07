import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore, prettyDate, sizeMm } from "./useAppStore";
import type { Photo } from "../types";

const basePhoto: Photo = {
  id: "1",
  path: "a.jpg",
  width: 100,
  height: 100,
  orientation: 1,
  status: "ok",
  datetime: "2024:06:12 10:30:00",
};

describe("useAppStore", () => {
  beforeEach(() => {
    useAppStore.setState({ photos: [], selectedId: null });
  });

  it("formats exif datetime into a pretty date", () => {
    expect(prettyDate("2024:06:12 10:30:00")).toBe("2024 · 六月");
    expect(prettyDate("自由填写")).toBe("自由填写");
    expect(prettyDate(undefined)).toBe("");
  });

  it("selects the first photo when photos are set", () => {
    useAppStore.getState().setPhotos([basePhoto]);
    expect(useAppStore.getState().selectedId).toBe("1");
  });

  it("per-photo override wins over exif datetime", () => {
    useAppStore.getState().setPhotos([basePhoto]);
    expect(useAppStore.getState().effectiveDate("1")).toBe("2024 · 六月");
    useAppStore.getState().updatePhoto("1", { overrideDate: "手动填写" });
    expect(useAppStore.getState().effectiveDate("1")).toBe("手动填写");
  });

  it("per-photo override wins over detected place", () => {
    useAppStore
      .getState()
      .setPhotos([{ ...basePhoto, place: "杭州市" }]);
    expect(useAppStore.getState().effectivePlace("1")).toBe("杭州市");
    useAppStore.getState().updatePhoto("1", { overridePlace: "西湖" });
    expect(useAppStore.getState().effectivePlace("1")).toBe("西湖");
  });

  it("applies info from selected photo to all", () => {
    useAppStore.getState().setPhotos([
      basePhoto,
      { ...basePhoto, id: "2", datetime: undefined },
    ]);
    useAppStore.getState().updatePhoto("1", {
      overrideDate: "2024 · 六月",
      overridePlace: "西湖",
    });
    useAppStore.getState().applyInfoToAll();
    const photos = useAppStore.getState().photos;
    expect(photos[1].overrideDate).toBe("2024 · 六月");
    expect(photos[1].overridePlace).toBe("西湖");
  });

  it("computes standard print sizes", () => {
    expect(sizeMm("6", 0, 0)).toEqual({ widthMm: 101.6, heightMm: 152.4 });
    expect(sizeMm("5", 0, 0)).toEqual({ widthMm: 88.9, heightMm: 127.0 });
    expect(sizeMm("7", 0, 0)).toEqual({ widthMm: 127.0, heightMm: 177.8 });
    expect(sizeMm("custom", 100, 150)).toEqual({
      widthMm: 100,
      heightMm: 150,
    });
  });
});
