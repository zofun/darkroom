# 照片打印预处理工具 实现计划（总览）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建 Windows 桌面端照片打印预处理工具：尺寸规整、时间/地点水印、批量导出，并为未来的相册排版预留渲染扩展点。

**Architecture:** Tauri v2 壳 + React/TS 前端 + Rust 图像管线。核心逻辑（尺寸、裁切、EXIF、地名、水印、导出、批处理）放在纯 Rust 库中，用 `cargo test` 独立验证；Tauri 命令只做薄桥接；前端负责暗房界面与实时预览，水印预览复用 Rust 生成的 SVG 以保证与导出一致。

**Tech Stack:** Tauri v2, React + TypeScript + Vite, Zustand, Rust（image, kamadak-exif, resvg/usvg/fontdb, rayon, walkdir, tempfile）, Windows Imaging Component（HEIC）。

**设计来源:** `docs/superpowers/specs/2026-11-07-photo-print-preprocessor-design.md`

---

## 计划分卷（按顺序执行）

| 阶段 | 文件 | 内容 | 可独立验证 |
|---|---|---|---|
| Phase 0 | 本文件 · Task 1 | 初始化 Tauri v2 + React/TS + Vite | `npm run tauri dev` 弹窗 |
| Phase 1 | `2026-11-07-p1-rust-core.md` | Rust 核心：模型、尺寸、裁切、EXIF、地名、水印、HEIC、管线、导出、批处理 | `cargo test` |
| Phase 1b | `2026-11-07-p2-watermark-heic.md` | 水印 SVG、HEIC 解码 | `cargo test` |
| Phase 1c | `2026-11-07-p3-pipeline-export-batch.md` | 管线、导出、批处理 | `cargo test` |
| Phase 2–4 | 本次直接实现 | Tauri 桥接、前端界面、release 打包 | `npm test` + `npx tauri build` |

> 执行规则：每个 Task 内按 Step 顺序做，先写失败测试 → 运行确认失败 → 最小实现 → 运行确认通过 → 提交。任何一步失败就停下来修复，不要跳到下一步。

---

## 全局文件结构

```
photo-maker/
├─ package.json
├─ vite.config.ts
├─ index.html
├─ src/
│  ├─ main.tsx
│  ├─ App.tsx
│  ├─ types.ts
│  ├─ lib/api.ts
│  ├─ store/useAppStore.ts
│  ├─ store/useAppStore.test.ts
│  ├─ styles/tokens.css
│  ├─ styles/global.css
│  └─ components/
│     ├─ AppShell.tsx
│     ├─ FilmStrip.tsx
│     ├─ CanvasPreview.tsx
│     ├─ Inspector.tsx
│     ├─ ImportButton.tsx
│     └─ ExportDialog.tsx
├─ src-tauri/
│  ├─ Cargo.toml
│  ├─ tauri.conf.json
│  ├─ assets/fonts/SourceHanSerifSC.otf
│  ├─ assets/geo/places.json
│  └─ src/
│     ├─ main.rs
│     ├─ lib.rs
│     ├─ commands.rs
│     └─ core/
│        ├─ mod.rs
│        ├─ model.rs
│        ├─ size.rs
│        ├─ geometry.rs
│        ├─ exif.rs
│        ├─ geo.rs
│        ├─ watermark.rs
│        ├─ heic.rs
│        ├─ pipeline.rs
│        ├─ export.rs
│        └─ batch.rs
└─ docs/superpowers/
```

**责任边界:** `core/*` 不依赖 Tauri，纯库，可被未来相册页面复用；`commands.rs` 只做参数转换与事件转发；前端不做任何像素处理，只做展示与状态。

---

## Phase 0 · 脚手架

### Task 1: 初始化 Tauri v2 + React/TS + Vite

**Files:**
- Create: 全项目脚手架（package.json, vite.config.ts, index.html, src/, src-tauri/）

- [ ] **Step 1: 用 Vite 生成 React/TS 前端到临时目录**

Run:
```powershell
npm create vite@latest app -- --template react-ts
```
Expected: 生成 `app/` 目录，含 `package.json`、`src/main.tsx`。

- [ ] **Step 2: 把脚手架内容移到仓库根目录**

Run:
```powershell
Get-ChildItem -Path app -Force | Move-Item -Destination .
Remove-Item -Recurse -Force app
```
Expected: 根目录出现 `package.json`、`index.html`、`src/`、`vite.config.ts`。

- [ ] **Step 3: 安装依赖并初始化 Tauri**

Run:
```powershell
npm install
npm install -D @tauri-apps/cli@latest
npx tauri init --app-name DARKROOM --window-title "DARKROOM 暗房" --frontend-dist ../dist --dev-url http://localhost:5173 --before-dev-command "npm run dev" --before-build-command "npm run build"
```
Expected: 生成 `src-tauri/`，含 `Cargo.toml`、`tauri.conf.json`、`src/main.rs`、`src/lib.rs`。

- [ ] **Step 4: 验证开发模式能启动**

Run:
```powershell
npm run tauri dev
```
Expected: 编译成功并弹出标题为「DARKROOM 暗房」的窗口；终端显示 Vite dev server 已启动。关闭窗口结束。

- [ ] **Step 5: Commit**

```powershell
git add -A
git commit -m "chore: scaffold tauri v2 + react-ts project"
```

---

## 后续

- Phase 1a：`docs/superpowers/plans/2026-11-07-p1-rust-core.md`
- Phase 1b：`docs/superpowers/plans/2026-11-07-p2-watermark-heic.md`
- Phase 1c：`docs/superpowers/plans/2026-11-07-p3-pipeline-export-batch.md`
- Phase 2–4（Tauri 桥接、前端、打包）已在本次实现中直接完成。
