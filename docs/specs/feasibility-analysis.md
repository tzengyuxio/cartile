# Tilemap 跨平台工具：市場可行性分析與產品設計

> 日期：2026-03-21

## 願景

在 tilemap 領域做出一個相當於 Spine 在骨架動畫領域的工具——跨平台編輯器 + 統一資料格式 + 多語言 runtime binding，成為 2D tilemap 的事實標準。

## 產品定位

- **類型**：開源 tilemap 編輯器 + 跨引擎 runtime 生態
- **商業模式**：完全開源，先建立生態影響力，成為事實標準後再探索商業化路徑。潛在的可持續性來源包括：GitHub Sponsors / Open Collective 贊助、付費教學 / 諮詢、雙授權（MIT 開源 + 商業友好授權）、或未來的 premium 功能（雲端協作、企業支援等）。前期不追求營利，但需在 Phase 2 結束前驗證是否有可持續的贊助基礎
- **目標用戶**：獨立遊戲開發者（programmer）+ 關卡設計師 / 美術（designer）
- **涵蓋範圍**：經典 2D 格狀 tilemap → level design（entity/trigger）→ 廣義 2D 世界編輯（程式化生成、動畫場景）
- **投入規模**：認真的個人專案，大量時間投入

---

## 一、市場規模與機會

### 市場數據

| 指標 | 數據 |
|------|------|
| 2D 遊戲市場規模 | 2024 年 USD 1,799 億，預計 2033 年達 USD 3,344 億（CAGR 7.5%） |
| 獨立遊戲市場 | 2025 年 USD 48.5-111.4 億，獨立工作室佔 52.7% |
| 遊戲引擎市場 | 持續成長，2D 需求穩定 |

### 可觸及市場（TAM → SAM → SOM）

- **TAM**：所有 2D 遊戲開發者（數十萬人）
- **SAM**：使用 tilemap 的開發者（RPG、platformer、策略遊戲等），估計佔 2D 開發者的 40-60%
- **SOM**：願意嘗試新工具的開發者。參考指標：
  - Tiled：GitHub ~11k stars，200+ 付費贊助者，月收 $3,600
  - LDtk：GitHub 3.8k stars，免費
  - Spine（可對標）：278 家公司使用，$299/專業版

### 關鍵洞察

市場不大但穩定，且現有工具的維護者都是個人或小團隊——進入門檻低，但天花板相對有限。這不是一個能養活大公司的市場，但對認真的個人專案來說，空間足夠。

---

## 二、競爭態勢分析

### 直接競爭者

| 工具 | 強項 | 弱項 | 威脅程度 |
|------|------|------|----------|
| **Tiled** | 生態最大、格式通用、活躍維護（2026-03 仍在更新） | UI 老舊、C++/Qt 技術棧笨重、auto-tiling 弱 | ★★★★☆ |
| **LDtk** | UX 最佳、JSON 乾淨、名人效應（Dead Cells） | Haxe 生態小、個人維護、更新放緩 | ★★★☆☆ |
| **引擎內建**（Godot/Unity） | 零整合成本 | 綁定引擎、跨平台無法共用 | ★★★☆☆ |

### 間接競爭者

| 工具 | 說明 |
|------|------|
| **Aseprite** | 像素畫編輯器，部分用戶直接在裡面畫 tileset + 簡易 tilemap |
| **程式化生成** | Wave Function Collapse 等演算法，部分場景不需要手動編輯 tilemap |

### 競爭空白（機會）

1. **沒有人同時做好「編輯器 + 跨引擎 runtime」**— Tiled 的 runtime 全靠社群各自實作，品質參差不齊；LDtk 的 Haxe-first 讓非 Haxe 用戶成為二等公民
2. **沒有人用現代技術棧**— Tiled 是 C++/Qt，LDtk/Ogmo 是 Haxe/Electron，沒有人用 Rust/WASM 這種兼顧效能和跨平台的方案
3. **沒有人認真對待格式標準化**— TMX 是 XML 時代的產物，LDtk JSON 是編輯器內部格式外洩，都不是為「跨工具互通」設計的

---

## 三、Spine 成功模式的可複製性

| Spine 成功因素 | 在 tilemap 領域的對應情況 | 可複製？ |
|---|---|---|
| **解決了真實痛點**：逐幀動畫浪費記憶體、修改成本高 | tilemap 的痛點存在但不如骨架動畫劇烈——現有工具「能用」，只是不夠好 | ⚠️ 部分 |
| **編輯器體驗不可取代**：動畫師用了就回不去 | 目前沒有任何 tilemap 編輯器達到「不可取代」的程度，機會存在 | ✅ 可以 |
| **官方 runtime 品質統一**：每個引擎的 runtime 都由 Spine 官方維護 | 這正是 Tiled 最大的弱點——runtime 全靠社群，品質參差。最值得複製的策略 | ✅ 關鍵 |
| **付費門檻篩選專業用戶** | 開源路線，用戶獲取更容易但營利更難。前期是優勢（降低採用門檻） | ⚠️ 取捨 |
| **先發優勢 + 網路效應** | tilemap 領域還沒有人佔據這個位置，窗口是開的 | ✅ 機會 |

### 核心結論

Spine 模式在 tilemap 領域可以部分複製，但有一個根本差異——

> **骨架動畫沒有 Spine 幾乎做不了；tilemap 沒有專用編輯器還是做得了。**

這代表工具不是「從 0 到 1」的革命，而是「從 60 分到 90 分」的體驗躍進。要成功，編輯器必須在某些工作流上做到「用過就回不去」的程度。最可能的突破口：

1. **智慧 auto-tiling**（把 47-tile 設定從痛苦變成一鍵完成）
2. **官方統一 runtime**（一次設定，所有引擎行為一致）
3. **動畫 tile + 即時預覽**（現有工具幾乎都做得很差）

---

## 四、技術可行性與策略優勢

### 為什麼選擇 Rust + WASM

本專案的核心需求是「一份程式碼，多種部署形態」——同一個 core library 要能作為：
1. **Native library**（透過 C ABI / FFI 給各語言呼叫）
2. **WASM module**（在瀏覽器中驅動編輯器，也可嵌入引擎的 Web 工具鏈）
3. **CLI 工具**（格式轉換、驗證）
4. **桌面應用**（未來透過 wgpu 實現 native 編輯器）

能同時滿足這四種部署目標的語言選擇非常有限：

| 候選語言 | Native 效能 | WASM 支援 | C ABI / FFI | 記憶體安全 | 生態活躍度 | 適合度 |
|----------|-------------|-----------|-------------|------------|------------|--------|
| **Rust** | ✅ 接近 C++ | ✅ 一等公民（wasm-bindgen, wasm-pack） | ✅ 原生支援 | ✅ 編譯期保證 | ✅ 遊戲開發生態快速成長 | ★★★★★ |
| **C++** | ✅ 最快 | ⚠️ 可行但工具鏈複雜（Emscripten） | ✅ 原生 | ❌ 手動管理 | ⚠️ 成熟但貢獻門檻高 | ★★★☆☆ |
| **C** | ✅ 最快 | ⚠️ 可行但更繁瑣 | ✅ 最通用 | ❌ 手動管理 | ⚠️ 缺乏現代抽象能力 | ★★☆☆☆ |
| **Go** | ⚠️ GC pause | ⚠️ WASM binary 過大（~10MB+） | ❌ CGO 效能差 | ✅ GC 管理 | ⚠️ 遊戲生態弱 | ★★☆☆☆ |
| **Zig** | ✅ 接近 C | ✅ 良好 | ✅ 原生 C ABI | ⚠️ 手動但有安全機制 | ❌ 生態尚未成熟 | ★★★☆☆ |

**選擇 Rust 的關鍵理由**：

1. **WASM 是一等公民** — Rust 的 WASM 工具鏈（wasm-bindgen、wasm-pack）是所有語言中最成熟的。編輯器以 Web 為首要平台，這一點至關重要
2. **「寫一次，部署到處」真正可行** — 同一份 Rust 程式碼可以編譯為 native binary（CLI/桌面）、WASM（Web 編輯器）、和 C-compatible library（各語言 binding），不需要維護多份實作
3. **型別系統保證格式正確性** — Rust 的 `serde` + 強型別 enum 可以在編譯期確保格式 schema 和程式碼一致，避免 runtime 的格式解析錯誤
4. **記憶體安全 = 更可靠的 runtime** — 遊戲開發者嵌入第三方 library 最怕的是記憶體洩漏和 crash，Rust 在這方面有結構性優勢
5. **社群契合度高** — Rust 遊戲開發社群（Bevy、wgpu、macroquad）活躍且品質意識強，容易吸引高品質貢獻者
6. **避免重蹈 Haxe 覆轍** — LDtk 和 Ogmo 都選了 Haxe，結果社群太小導致生態難以擴展。Rust 的社群規模和成長趨勢遠優於 Haxe

### 技術棧比較

| 面向 | Tiled (C++/Qt) | LDtk (Haxe/Electron) | 本方案 (Rust/WASM) |
|------|---|---|---|
| **跨平台** | 需編譯各平台 binary | Electron 肥大（~200MB） | Native + Web 雙模式，WASM 零安裝 |
| **效能** | 最快 | 中等 | 接近 C++ 效能，遠超 Electron |
| **Runtime binding** | 社群各自實作 | Haxe-first，其他語言二等公民 | Rust core + C ABI，天然支援所有語言 FFI |
| **格式安全性** | XML parsing 老問題 | 弱型別 JSON | 可用 Rust 的型別系統保證格式正確性 |
| **可嵌入性** | 不可能 | 不可能 | WASM 可嵌入任何引擎的編輯器 |
| **社群貢獻門檻** | C++ 門檻高 | Haxe 冷門 | Rust 社群活躍且品質意識高 |

### Runtime 架構：Core + Binding 分層

WASM 和 C ABI 是兩條互補的部署路線，而非二擇一。核心邏輯只用 Rust 寫一次，各引擎透過薄膠水層（binding layer）整合：

```
                    ┌─────────────────────────────────────┐
                    │         Rust Core Library            │
                    │  (格式解析、auto-tiling、tileset管理)  │
                    └──────┬──────────────┬────────────────┘
                           │              │
              ┌────────────▼───┐    ┌─────▼──────────┐
              │   C ABI (.so)  │    │  WASM (.wasm)  │
              │   Native 路線   │    │   Web 路線      │
              └──┬────┬────┬───┘    └──┬─────────┬───┘
                 │    │    │           │         │
                 ▼    ▼    ▼           ▼         ▼
              Godot Unity Bevy    Web 編輯器  PixiJS 遊戲
```

各引擎的整合方式：

| 引擎 | 整合機制 | 需要額外寫什麼 | 工作量 |
|------|----------|---------------|--------|
| **Godot** | GDExtension（支援 C ABI） | 一層 Rust → GDExtension binding（用 `gdext` crate） | 中等，有成熟工具鏈 |
| **Unity** | Native Plugin（.dll/.so/.dylib） | 一層 C# P/Invoke wrapper 呼叫 C ABI | 中等，主要是 marshalling |
| **Bevy** | 直接用 Rust crate | 不需要任何 binding，直接 `use` | 零 |
| **PixiJS / Web** | WASM module | 一層 JS/TS wrapper（用 `wasm-bindgen` 自動生成） | 低，大部分自動化 |
| **其他 C/C++ 引擎** | 直接連結 C ABI | 標準 C header | 低 |

**與 Tiled 現狀的根本差異**：Tiled 的每個引擎 runtime 都是社群各自從零實作完整的 TMX parser + renderer，導致行為不一致、bug 各自不同。本方案的 binding layer 只負責介面轉譯（型別轉換、生命週期管理），不含業務邏輯，因此**所有引擎上的核心行為 100% 一致**。

**這正是 Spine 採用的架構**：spine-core 是核心動畫邏輯（一份實作），spine-unity / spine-godot / spine-ts 各是一層薄膠水。每個 runtime 都是「核心引擎 + 薄膠水」，而非各自重寫。本方案複製這個經過驗證的模式。

### 分階段推進策略

```
Phase 1（建立基礎）— 預估 4-6 個月
├── 定義格式 spec v0.1（JSON Schema）
│   └── Scope：基本 tilemap（orthogonal/isometric/hex）、tileset 定義、
│       圖層（tile layer + object layer）、自訂屬性。
│       不含 entity 系統和程式化生成（留給 Phase 3）
├── Rust core library
│   └── Scope：格式讀寫與驗證、基礎 auto-tiling（bitmask-based，
│       非智慧推斷）、tileset 管理
├── CLI 工具（格式轉換：TMX/LDtk JSON → 新格式，雙向）
└── 1-2 個 runtime binding（Godot GDExtension + JS/WASM）

Phase 2（編輯器 MVP）— 預估 6-9 個月
├── Web-based 編輯器（Rust + WASM + WebGPU，降級到 Canvas 2D）
├── 核心功能：tilemap 繪製、圖層管理、基礎 auto-tiling
├── 匯入既有 Tiled/LDtk 專案
└── 更多 runtime binding（Bevy、Unity）

Phase 3（差異化功能）— 持續迭代
├── 智慧 auto-tiling（一鍵規則推斷）
├── 動畫 tile 編輯 + 即時預覽
├── Entity / trigger / collision 編輯
├── 程式化生成規則整合
└── Native 桌面版（同一 Rust codebase，透過 wgpu）
```

**Phase 1 的 runtime 策略**：runtime library 同時支援直接讀取 TMX 格式和自有格式。這讓用戶可以零成本開始使用（繼續用 Tiled 編輯，用新 runtime 載入），同時體驗到統一 runtime 的好處（跨引擎行為一致、效能優化）。當用戶準備好遷移時，CLI 工具可以將 TMX 轉為自有格式以獲得完整功能。

**Phase 1 的用戶限制**：此階段只有 library 和 CLI，實際上只服務 programmer 用戶。Designer 用戶需要等到 Phase 2 的編輯器才能加入。這是刻意的取捨——先用技術用戶驗證格式設計的合理性。

### 格式設計核心原則

1. **Human-readable** — JSON 格式，開發者可以直接閱讀和手動編輯
2. **Schema-validated** — 完整的 JSON Schema，支援編輯器和 CI 驗證
3. **Version-control friendly** — 結構穩定、diff 友好，避免不必要的欄位重排
4. **Forward-compatible** — 版本化 schema，新版本可讀取舊格式，未知欄位保留不丟棄
5. **Streaming-parseable** — 大地圖可以分區塊載入，不需要一次讀取整個檔案
6. **Engine-agnostic** — 不假設特定引擎的座標系統、渲染管線或物理引擎
7. **Composable** — tileset 定義與 map 定義分離，同一 tileset 可被多個 map 引用

---

## 五、現存痛點分析

根據社群討論（[itch.io: Why are 2D tilemaps still a pain in 2025?](https://itch.io/t/4996279/-why-are-2d-tilemaps-still-a-pain-in-2025)），目前 tilemap 工具仍有以下未被良好解決的問題：

1. **Auto-tiling 規則設定繁瑣** — 47-tile blob tileset 的規則設定極其痛苦，terrain transition 組合爆炸
2. **Y-sorting 與層級管理** — Top-down 遊戲中物件遮擋關係（z-order）至今沒有一致且直覺的方案
3. **Tileset 缺乏標準規範** — 每個美術產出的 tileset 格式各異，匯入流程碎片化
4. **動畫 Tile 工作流破碎** — 多數編輯器對 animated tile 的支援停留在基礎階段
5. **Editor 與 Runtime 之間的斷裂** — 編輯器產出的資料格式與引擎需要的資料結構之間總是需要轉換層
6. **跨引擎可攜性差** — Tilemap 資料與工作流高度綁定特定引擎或編輯器

---

## 六、風險評估與緩解策略

| 風險 | 嚴重度 | 可能性 | 緩解策略 |
|------|--------|--------|----------|
| **Tiled 大幅現代化** | 高 | 低 — 15+ 年 C++/Qt 包袱，重寫機率極低 | 專注在 Tiled 結構上做不到的事（WASM 嵌入、官方 runtime） |
| **個人精力不足** | 高 | 中 | Phase 1 先只做 library，驗證市場需求後再投入編輯器；開源吸引貢獻者 |
| **格式戰爭 / TMX 生態慣性** | 高 | 高 — 技術優越性從來不是格式勝出的主因，生態和慣性才是 | 不強推取代 TMX，而是同時支援 TMX 讀取；讓編輯器體驗帶動格式自然採納 |
| **2D 遊戲市場萎縮** | 中 | 低 — 數據顯示持續成長 | 格式設計預留 2.5D / isometric 支援 |
| **做出來沒人用** | 高 | 中 | Phase 1 的 TMX 轉換器讓用戶零成本試用；在 Reddit/itch.io/Discord 建立社群 |
| **AI 生成地圖取代手動 tilemap** | 中 | 長期可能 — 引擎可能直接內建 AI tilemap 生成 | 工具本身整合 AI 輔助，把威脅變功能；即使 AI 生成地圖，仍需要格式和 runtime 來承載結果 |

---

## 七、成功指標（里程碑）

| 階段 | 時間框架 | 成功指標 |
|------|----------|----------|
| **Phase 1 完成後 6 個月** | 上線後半年 | GitHub 300+ stars、crates.io/npm 月下載量 500+、Discord 社群 100+ 人、至少 2 個引擎有可用的 binding |
| **Phase 2 MVP 後 6 個月** | 編輯器上線後半年 | GitHub 1,000+ stars、月活躍用戶 200+、至少 3 篇社群自發的使用教學或分享 |
| **格式開始被採納** | Phase 2 穩定後 | 有第三方工具主動支援格式匯出、至少 1 款已上架遊戲使用此格式 |
| **長期目標** | 2-3 年 | 在 Reddit/itch.io 的 tilemap 工具推薦討論中被提及為主流選項之一 |

---

## 八、總結判斷

| 面向 | 評估 |
|------|------|
| **市場需求** | ✅ 存在，但是「從 60 分到 90 分」的改善，不是從 0 到 1 |
| **競爭窗口** | ✅ 開放——沒有人同時佔據「好編輯器 + 統一 runtime + 現代技術棧」 |
| **技術可行性** | ✅ Rust + WASM 是正確選擇，結構性優於現有競爭者 |
| **個人可執行性** | ⚠️ 可行但需要嚴格分階段，Phase 1 以 library 為核心降低初期工作量 |
| **成為事實標準的可能性** | ⚠️ 有路徑但需要時間——靠編輯器體驗帶動格式採納，而非直接推標準 |
| **最大風險** | 精力分散 + 沒人用。用 Phase 1 的低成本方式先驗證需求 |

**結論：值得做，但要有正確的期望管理。** 這不會是一夜爆紅的產品，而是一個需要 2-3 年持續投入的生態建設。最關鍵的一步是 Phase 1——如果 Rust library + TMX 轉換器能吸引到第一批技術用戶，後續的路就會清晰很多。

---

## 產品策略

### 方案比較

在設計階段評估了三種切入策略：

| 方案 | 策略 | 優點 | 缺點 |
|------|------|------|------|
| **A：格式驅動**（tilemap 界的 glTF） | 先定義開放格式規範，再圍繞它建構生態 | 護城河最深 | 沒有殺手級編輯器的格式極難推動採納；個人推動標準幾乎不可能 |
| **B：編輯器驅動**（tilemap 界的 Spine） | 先做體驗遠超現有工具的編輯器，格式隨編輯器定義 | 最接近 Spine 成功路線；用戶用了就離不開 | 編輯器開發工作量大 |
| **C：Runtime Library 驅動**（tilemap 界的 SDL） | 先做跨語言高效能 runtime core | 開發量最小、最快能推出可用的東西 | 沒有視覺化工具難以吸引非程式開發者；天花板較低 |

### 選定策略

**方案 B 為主，借用方案 C 的分階段策略**：

- Phase 1 先推出 runtime library + 格式 spec（方案 C 的低成本驗證），讓早期技術用戶先用起來
- Phase 2 推出編輯器 MVP（方案 B 的核心），此時格式已穩定、有 runtime 驗證過
- Phase 3 差異化功能，建立「用過就回不去」的體驗

### 退出策略

如果 Phase 1 完成後 6 個月內未達成基礎指標（GitHub 300+ stars、crates.io 月下載量 200+），應重新評估方向：考慮轉為純 library 專案（放棄編輯器），或併入現有工具生態（如為 Tiled 貢獻 Rust runtime）。

---

## 參考資料

- [Tiled Map Editor](https://www.mapeditor.org/) — [v1.12 Release Notes (2026-03)](http://www.mapeditor.org/2026/03/13/tiled-1-12-released.html)
- [LDtk 官網](https://ldtk.io/) — [GitHub Releases](https://github.com/deepnight/ldtk/releases)
- [Ogmo Editor 3 CE](https://ogmo-editor-3.github.io/) — [GitHub](https://github.com/Ogmo-Editor-3/OgmoEditor3-CE)
- [Spine 購買頁面](https://esotericsoftware.com/spine-purchase)
- [glTF - Khronos Group](https://www.khronos.org/gltf/)
- [Why are 2D tilemaps still a pain in 2025? (itch.io)](https://itch.io/t/4996279/-why-are-2d-tilemaps-still-a-pain-in-2025)
- [Indie Game Market Size (Mordor Intelligence)](https://www.mordorintelligence.com/industry-reports/indie-game-market)
- [2D Game Market Size (Verified Market Reports)](https://www.verifiedmarketreports.com/product/2d-game-market/)
- [Slant: Best 2D Tilemap Editors 2026](https://www.slant.co/topics/1469/~best-2d-tilemap-editors)
