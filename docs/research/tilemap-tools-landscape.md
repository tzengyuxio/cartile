# Tilemap 工具與函式庫分析

> 分析日期：2026-03-21

## 概述

本文整理目前主流的 tile map 編輯器與函式庫，分析其技術棧、維護狀態、痛點，以及自行開發 tilemap 系統的可行性。

---

## 主流 Tilemap 編輯器比較

### 獨立編輯器

| 工具 | 開發語言 | 最新版本 | 維護狀態 | 授權 |
|------|----------|----------|----------|------|
| **[Tiled](https://www.mapeditor.org/)** | C++ / Qt | v1.12（2026-03） | 活躍維護 | GPL / 商業 |
| **[LDtk](https://ldtk.io/)** | Haxe + Electron | v1.5.3（2025-01） | 更新放緩，個人維護 | MIT |
| **[Ogmo Editor 3 CE](https://ogmo-editor-3.github.io/)** | Haxe + Electron | v3.4.0 | 基本停滯，維護模式 | MIT |

### 引擎內建 Tilemap 系統

| 引擎 | 系統名稱 | 特色 |
|------|----------|------|
| **Unity** | Tilemap + Tile Palette | 內建 Rule Tile、Animated Tile 等擴充 |
| **Godot 4** | TileMap / TileMapLayer | 大幅重新設計，支援 terrain autotiling、physics layers |
| **Unreal** | Paper2D Tile Map | 較少人使用，2D 支援非主力 |

### 各語言 / 框架的 Tilemap 函式庫

| 平台 | 函式庫 | 說明 |
|------|--------|------|
| **Rust (Bevy)** | `bevy_ecs_tilemap` | ECS 架構，效能優異 |
| **Rust** | `tiled` crate | 讀取 Tiled `.tmx` / `.json` 格式 |
| **JavaScript (PixiJS)** | `@pixi/tilemap` | PixiJS 生態的 tilemap 渲染 |
| **JavaScript (Phaser)** | 內建 Tilemap | 直接支援 Tiled JSON 匯入 |
| **Java (libGDX)** | TiledMap | 載入與渲染 Tiled 格式 |
| **C# (MonoGame)** | MonoGame.Extended | Tilemap 支援模組 |
| **Python** | PyTMX | Tiled `.tmx` parser |
| **C++** | tmxlite | 輕量 Tiled 格式 parser |
| **C#** | TiledCS | Tiled 格式 parser |

---

## LDtk vs Ogmo Editor 技術細節

### LDtk

- **開發語言**：Haxe，編譯到 JavaScript，透過 Electron 打包為桌面應用
- **作者**：Sébastien Bénard（Dead Cells 開發者）
- **格式**：JSON，結構乾淨易解析
- **特色**：Auto-layer rules assistant、Multi-worlds、typed Haxe API
- **生態**：Godot / Rust / JS 都有社群 loader，但 Haxe 生態圈整體較小
- **風險**：個人維護，長期持續性有疑慮

### Ogmo Editor 3

- **開發語言**：最初用 TypeScript，後改用 **Haxe 重寫**（原因：開發者偏好 Haxe 的 C# 風格語法）
- **打包**：同樣使用 Electron
- **狀態**：社群版（CE）維護，但活動量極低
- **特色**：輕量、project-based workflow
- **知名用戶**：Celeste 使用了早期版本的 Ogmo Editor

---

## 現存痛點分析

根據社群討論（[itch.io: Why are 2D tilemaps still a pain in 2025?](https://itch.io/t/4996279/-why-are-2d-tilemaps-still-a-pain-in-2025)）以及開發者回饋，目前 tilemap 工具仍有以下未被良好解決的問題：

### 1. Auto-tiling 規則設定繁瑣

47-tile blob tileset 的規則設定極其痛苦，terrain transition（地形過渡）的組合呈指數爆炸。現有工具（包括 LDtk 的 auto-layer）只解決了部分場景。

### 2. Y-sorting 與層級管理

Top-down 遊戲中物件遮擋關係（z-order）的處理至今沒有一致且直覺的方案。多層 tile 的管理在所有編輯器中都不夠優雅。

### 3. Tileset 缺乏標準規範

每個美術產出的 tileset 格式各異——tile 大小、padding、spacing、命名慣例都不統一，導致匯入流程碎片化，每次換素材都要重新設定。

### 4. 動畫 Tile 工作流破碎

多數編輯器對 animated tile 的支援停留在基礎階段，預覽、編輯、匯出的體驗都不流暢。

### 5. Editor 與 Runtime 之間的斷裂

編輯器產出的資料格式與遊戲引擎實際需要的資料結構之間，幾乎總是需要一層轉換（importer / converter），增加開發摩擦。

### 6. 跨引擎可攜性差

Tilemap 資料與工作流高度綁定特定引擎或編輯器，換引擎幾乎等於從頭來過。

---

## 針對特定技術棧的推薦

目標平台：**Godot / PixiJS / Rust**

### 使用現成工具

| 方案 | 優點 | 缺點 | 推薦程度 |
|------|------|------|----------|
| **Tiled** | 生態最成熟，三個平台都有 parser | UI 略顯老舊、auto-tiling 不如 LDtk | ★★★★★ |
| **LDtk** | UX 最佳、JSON 格式乾淨 | Haxe 生態小、長期維護有疑慮 | ★★★★☆ |
| **Godot 內建 TileMap** | 與 Godot 整合最緊密 | 只限 Godot，無法跨平台共用 | ★★★☆☆ |

**結論**：如果需要跨 Godot / PixiJS / Rust 使用，**Tiled 是最穩妥的選擇**。

### 自行開發的可行方向

如果要打造有差異化的工具，以下是值得切入的方向：

| 方向 | 說明 | 技術棧建議 |
|------|------|------------|
| **引擎無關的 tilemap runtime library** | 統一資料格式 + 各平台 loader，解決跨引擎可攜性 | Rust core + FFI / WASM |
| **智慧 auto-tiling** | 自動推斷 tile 連接規則，降低 blob tileset 設定成本 | Rule-based engine 或 ML |
| **Web-native 輕量編輯器** | 不綁 Electron，可嵌入開發流程或跑在瀏覽器 | Rust (WASM) + Web UI |
| **Code-first tilemap DSL** | 用設定檔 / 腳本定義規則，對程式導向開發者更友好 | TOML/YAML schema + CLI |

**最有潛力的組合**：Rust (WASM) 核心 + 輕量 Web UI + 統一 JSON schema + 各引擎 loader，同時解決「跨引擎可攜性」和「Electron 肥大」兩大問題。

---

## 參考資料

- [Tiled Map Editor](https://www.mapeditor.org/) — [v1.12 Release Notes (2026-03)](http://www.mapeditor.org/2026/03/13/tiled-1-12-released.html)
- [LDtk 官網](https://ldtk.io/) — [GitHub Releases](https://github.com/deepnight/ldtk/releases)
- [Ogmo Editor 3 CE](https://ogmo-editor-3.github.io/) — [GitHub](https://github.com/Ogmo-Editor-3/OgmoEditor3-CE)
- [Why are 2D tilemaps still a pain in 2025? (itch.io)](https://itch.io/t/4996279/-why-are-2d-tilemaps-still-a-pain-in-2025)
- [Slant: Best 2D Tilemap Editors 2026](https://www.slant.co/topics/1469/~best-2d-tilemap-editors)
