# 照片打印预处理工具 · 设计文档

- 日期：2026-11-07
- 状态：已确认（v1 设计）
- 平台：Windows 10 / 11（仅桌面端）
- 定位：照片打印前的尺寸规整 + 时间/地点水印 + 批量导出工具，后续扩展为相册排版

---

## 1. 背景与目标

用户打印照片前，需要把各种来源的照片统一成标准冲印尺寸，并以美观的方式把拍摄时间和地点印在照片上，最后批量输出到独立文件夹。

v1 聚焦三件事：

1. **尺寸规整**：把照片处理成标准冲印尺寸。
2. **时间/地点水印**：自动读取 EXIF，缺失时允许手动输入，以美观的排版呈现。
3. **批量导出**：一键把处理结果写入用户指定的文件夹。

未来会扩展到「照片排版 / 相册」。v1 不实现，但必须预留干净的扩展入口。

### 成功标准

- 导入 → 调整 → 导出全流程无需联网、无需配置任何 API Key。
- 几十张 12–48MP 手机/相机照片，整体导出在秒级完成。
- 水印效果具备明确的视觉设计感，而非通用模板。

---

## 2. 功能范围

### v1 必须实现

- 导入：JPEG / PNG / HEIC(HEIF)。
- 尺寸：预设 5寸 / 6寸 / 7寸，以及自定义尺寸（mm 或 inch）。
- 裁切：填充裁切（默认）/ 完整保留 + 白边，可切换。
- 水印：时间 + 地点，4 种可切换样式，默认「角注」。
- 信息获取：时间读 EXIF `DateTimeOriginal`；地点读 EXIF GPS 并经**内置离线地名库**转中文地名；缺失时手动输入，且支持批量套用。
- 导出：批量写入用户指定文件夹，可取消，带进度与结果汇总。

### v1 明确不做（预留）

- 相册排版（仅在主导航预留入口）。
- RAW 格式。
- 直接驱动打印机 / 云同步 / 账号体系。

---

## 3. 技术选型

| 层 | 选择 | 理由 |
|---|---|---|
| 壳 | **Tauri v2** | 安装包小、内存低、启动快，Windows 原生 WebView |
| 前端 | **React + TypeScript + Vite** | 复杂界面与未来排版编辑器的生产力 |
| 状态 | **Zustand** | 轻量、易测 |
| 样式 | 手写 CSS + 设计令牌（暗房色板） | 保证设计感，不引重型 UI 库 |
| 图像编解码 | `image`（JPEG/PNG）、Windows Imaging Component（HEIC/HEIF，走系统编解码器） | 覆盖率高，HEIC 无需随包原生库 |
| 缩放 | `fast_image_resize`（Lanczos3 + SIMD） | 高质量、高性能 |
| EXIF | `kamadak-exif` | 解析时间、GPS、朝向 |
| 并行 | `rayon` | 批处理吞吐 |
| 水印光栅化 | `resvg` / `usvg` + `fontdb` | 用 SVG 描述排版，表现力强 |
| 字体 | 内置思源宋体（Source Han Serif SC，OFL） | 不依赖系统字体，效果一致 |
| 离线地名 | 内置中国省/市/区地名库 + `reverse_geocoder`（世界城市兜底） | 零 Key、离线 |

> 所有第三方库在实现阶段锁定具体版本；本表锁定的是技术方向。

---

## 4. 视觉方向：「暗房 / Darkroom」

- 底色：暖炭黑 `#141416`；面板 `#1b1b1e`；描边 `#2a2a2e`。
- 文字：暖纸白 `#e8e2d6`；次级 `#8a8578`；弱化 `#5f5b52`。
- 强调色：古铜金 `#c8a96a`。
- 字体：品牌/日期/地名用衬线（Georgia / 思源宋体）；控件用系统无衬线（Segoe UI）。
- 质感：极轻的颗粒纹理，细线分割，无大圆角、无渐变按钮、无玻璃拟态。
- 原则：克制、留白、像专业暗房/印前工具，规避通用 AI 模板感。

---

## 5. 页面结构

单窗口应用，自上而下：顶栏 / 主体 / 状态栏。

```
┌───────────────────────────────────────────────────────────────┐
│ 顶栏：DARKROOM 暗房 · 面包屑        [导入] [存预设] [导出到文件夹] │
├──────┬────────┬───────────────────────────────┬───────────────┤
│ 主   │ 胶片条  │ 画布（所见即所得）             │ 右侧属性面板   │
│ 导   │ 缩略图  │  照片 + 时间/地点水印           │  输出尺寸      │
│ 航   │        │                               │  裁切/留白     │
│      │        │                               │  时间/地点水印 │
│ 单张 │        │                               │  样式          │
│ 批量 │        │                               │               │
│ 预设 │        │ [尺寸chip]  [时间] [位置] 开关  │               │
│ 设置 │        │                               │               │
│ ──── │        │                               │               │
│ 排版 │        │                               │               │
│ 相册 │        │                               │               │
├──────┴────────┴───────────────────────────────┴───────────────┤
│ 状态栏：就绪 · 第 n/m 张 · 输出目录 · 已处理/待处理             │
└───────────────────────────────────────────────────────────────┘
```

- **主导航**：单张处理（默认）/ 批量处理 / 预设模板 / 设置；底部以虚线 + `SOON` 标记 **「排版·相册」**，即未来扩展入口。
- **胶片条**：竖向缩略图，快速切换与多选。
- **画布**：实时预览裁切与水印。
- **右侧属性面板**：分区可折叠，未来新功能以新增分区方式扩展，不改骨架。
- **底部画布工具条**：尺寸 chip 快速切换 + 时间/位置水印开关。

---

## 6. 数据模型

```ts
type SizePreset = "5寸" | "6寸" | "7寸" | "custom";

interface PrintSize {
  preset: SizePreset;
  widthMm: number;   // 5寸: 88.9 × 127.0, 6寸: 101.6 × 152.4, 7寸: 127.0 × 177.8（界面按整数 mm 显示）
  heightMm: number;
  dpi: number;       // 默认 300
}

type CropMode = "fill" | "fit";   // 填充裁切 | 完整保留+白边

type WatermarkStyle = "engraved" | "siderail" | "cornernote" | "footerband";

interface WatermarkConfig {
  style: WatermarkStyle;          // 默认 cornernote
  showDate: boolean;
  showLocation: boolean;
  showTime: boolean;              // 是否显示具体时分（默认否）
  dateFormat: string;             // 例：YYYY · M月 / YYYY.MM.DD
  font: string;                   // 内置字体标识
  colorToken: string;             // 预设色：暖白/古铜/纸白/墨黑
  scale: number;                  // 相对画面比例，默认 1.0
  position: "br" | "bl" | "tr" | "tl";  // 角注等样式的锚点
}

interface ExportConfig {
  format: "jpeg" | "png";
  quality: number;                // JPEG，默认 92
  filenamePattern: string;        // 默认 "{name}_{size}"
  keepExif: boolean;              // 保留拍摄时间等
  stripGps: boolean;              // 默认 true
  outputDir: string;              // 默认 打印输出_YYYYMMDD
}

interface PhotoItem {
  id: string;
  path: string;
  width: number;
  height: number;
  exif: {
    datetime?: string;
    gps?: { lat: number; lng: number };
    orientation?: number;
  };
  placeName?: string;             // 离线库结果或用户输入
  thumbnailPath?: string;
  proxyPath?: string;             // 编辑用中尺寸代理
  status: "ok" | "missing-info" | "error";
  error?: string;
  overrides?: {                   // 单张覆盖，优先于全局
    datetime?: string;
    placeName?: string;
    cropMode?: CropMode;
    cropFocus?: { x: number; y: number };
    watermark?: Partial<WatermarkConfig>;
  };
}

interface ProjectSettings {
  size: PrintSize;
  cropMode: CropMode;             // 默认 fill
  border: { enabled: boolean; color: string; widthMm: number };
  watermark: WatermarkConfig;
  export: ExportConfig;
}
```

- **全局设置** 作用于所有照片。
- **单张覆盖** 优先于全局，用于个别照片的时间/地点/裁切调整。
- 预设（Preset）= 命名的 `ProjectSettings` 片段，存为 JSON。

---

## 7. 处理管线

```
读取文件
  → 解析 EXIF（时间 / GPS / 朝向）
  → 时间规整；GPS → 离线地名（失败则坐标/空，可编辑）
  → 生成缩略图（磁盘缓存）+ 编辑代理图（内存 LRU）
  → [导出时] 解码全尺寸
      → 按 EXIF 朝向旋正
      → 裁切填充 或 完整保留 + 白边
      → 高质量缩放至目标像素（宽高 = 尺寸mm/25.4 × DPI）
      → 生成水印 SVG → resvg 光栅化 → 合成
      → 写入 DPI / EXIF
      → 编码（JPEG/PNG）
      → 临时文件 + 原子改名
```

**关键设计**：渲染核心抽象为 `compose(canvas, layers[]) → bitmap`。

- 单张导出 = 1 个照片图层 + 1 个文字图层。
- 未来相册页 = 多个照片图层 + 文字图层，**复用同一导出引擎**，这是扩展的关键。

目标像素示例（300 DPI）：

- 5寸 3.5×5″ → 1050 × 1500 px
- 6寸 4×6″ → 1200 × 1800 px
- 7寸 5×7″ → 1500 × 2100 px

---

## 8. 水印规格（4 种样式）

默认：**角注（cornernote）**。所有样式的字体、颜色、比例、位置均可调；v1 提供 4 个内置预设。

1. **宋刻 / engraved**：右下，衬线日期 + 金色细线 + 疏排地名。
2. **侧栏 / siderail**：右侧竖排暗条，胶片边缘编号感；会压占右侧少量画面。
3. **角注 / cornernote（默认）**：左下，金色小点 + 日期 + 小字地名，最克制。
4. **底注 / footerband**：底部渐隐压暗条，日期居左、地名居右，最易读。

实现方式：以画面宽高为基准按比例布局，生成 SVG 后由 Rust 光栅化；前端预览使用同一几何规则，保证一致性。

时间显示：默认仅日期（如 `2024 · 六月`），`showTime` 开启后追加时分。
地点显示：默认「市 · 区」或用户填写的地名。

---

## 9. 导出规格

- 格式：JPEG（质量 92）默认；可选 PNG。
- DPI：默认 300，写入文件元数据。
- 命名：`{原文件名}_{尺寸}`，如 `IMG_0001_6寸.jpg`；冲突自动追加序号 `_1`、`_2`。
- 输出目录：用户选择；默认 `打印输出_YYYYMMDD`。
- EXIF：写入 DPI、保留拍摄时间；**默认剥离 GPS**（隐私），可在设置中关闭。
- 写入：先写临时文件，成功后原子改名，避免半成品。
- 进度：通过 Tauri 事件回传（已完成/总数/当前文件名），支持取消。
- 结束：汇总成功数、失败数、失败原因。

---

## 10. 性能策略

- 缩略图与编辑预览均基于**代理图**，避免反复解码全尺寸。
- 缩略图磁盘缓存 + 内存 LRU。
- `rayon` 并行处理，但按内存预算限制并发（48MP 单张解码可达上百 MB，默认按 `min(核心数, 内存/单张预算)` 控制）。
- 全尺寸解码仅在导出阶段发生，处理完即释放。
- 重活全部在 Rust 侧线程中执行，绝不在 UI 线程做解码/缩放。
- 字体、地名库启动时加载并常驻，避免重复 IO。
- 验收：几十张 12–48MP，整体秒级完成。

---

## 11. 错误处理

| 场景 | 行为 |
|---|---|
| 损坏/不支持的文件 | 标记该项 `error`，批量继续，汇总中列出 |
| 无 EXIF 时间 | 标记 `missing-info`，可单张或批量套用 |
| 无 GPS | 留空或显示坐标，可手动改名 |
| 离线地名未命中 | 显示坐标/空白 + 可编辑 |
| 输出文件已存在 | 默认自动加序号，可切换为覆盖/跳过 |
| 批量中途取消 | 优雅停止，报告已完成数量，不产生半成品 |
| 磁盘/权限错误 | 明确报错，保留已成功文件 |
| 字体缺失 | 内置字体，正常回退，不报错 |

---

## 12. 扩展设计：相册（预留，不在 v1）

- 左侧导航「排版·相册」进入**独立工作区路由**，与单张/批量互不干扰。
- 相册页复用 `compose(canvas, layers[])`：一页 = 多照片层 + 文字/装饰层。
- 尺寸/水印/导出配置复用现有模型，新增「页面模型」即可。
- 数据驱动的水印样式与预设，便于后续增加模板。

---

## 13. 项目结构（规划）

```
photo-maker/
├─ src/                        # React 前端
│  ├─ app/                     # AppShell、路由
│  ├─ features/
│  │  ├─ import/
│  │  ├─ single/               # 单张工作台
│  │  ├─ batch/
│  │  └─ export/
│  ├─ components/              # 胶片条、Inspector 面板等
│  ├─ store/                   # Zustand
│  └─ styles/                  # 设计令牌（暗房色板）
├─ src-tauri/
│  ├─ src/
│  │  ├─ commands/             # 导入/EXIF/缩略图/预览/导出/取消
│  │  ├─ pipeline/             # 解码→旋正→裁切→缩放→水印→编码
│  │  ├─ exif.rs
│  │  ├─ geo/                  # 离线地名
│  │  ├─ watermark/            # SVG 生成 + 光栅化
│  │  ├─ cache.rs
│  │  └─ export.rs
│  ├─ assets/fonts/            # 内置思源宋体
│  ├─ assets/geo/              # 离线地名库
│  └─ tauri.conf.json
└─ docs/superpowers/specs/     # 设计文档
```

---

## 14. 测试策略

**Rust 单元测试**

- 尺寸换算：5/6/7寸 @300DPI → 期望像素。
- 裁切/白边几何：横图、竖图、极端比例。
- EXIF 解析：时间、GPS、朝向。
- 地名匹配：已知坐标 → 期望行政区。
- 水印 SVG 几何：各样式锚点/比例正确。
- 命名冲突：重复文件名自动序号。

**Rust 集成测试**

- 样例图跑通完整管线，断言输出宽高、正确旋正、水印像素存在。

**前端测试（Vitest）**

- Inspector 面板逻辑、全局设置与单张覆盖的优先级。
- 状态切换、批量套用。

**视觉回归**

- 4 种水印样式做 golden image 对比。

**端到端（可选）**

- `tauri-driver` 跑「导入 → 调参 → 导出」主流程。

---

## 15. 假设与约束

- 仅 Windows 10/11 桌面端。
- 离线运行（除首次安装外无需网络）。
- 用户可接受地名精度到「市/区」，具体景点名可手动补充。
- HEIC 解码使用 Windows 系统 HEIF 编解码器（Win11 内置；Win10 需安装 HEIF 图像扩展）；不可用时给出明确提示，不影响 JPEG/PNG。
- 内置字体使用 OFL 许可的思源宋体，可再分发。
