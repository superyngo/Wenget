# WenPM 開發計畫

## 📋 專案概述

**專案名稱**: WenPM (Wen Package Manager)
**專案定位**: 跨平台 GitHub Binary 包管理器
**開發語言**: Rust
**目標平台**: Windows, Linux, macOS
**GitHub**: https://github.com/superyngo/WenPM

### 核心特點
- 無版本管理（永遠安裝最新版）
- 完全本地 metadata 維護
- 自動解析 GitHub Release
- 支援多來源（GitHub/GitLab/self-hosted）
- 多執行緒下載與分析

---

## 🏗️ 技術架構

### 專案結構
```
wenpm/
├── Cargo.toml
├── README.md
├── DEVELOPMENT_PLAN.md
├── src/
│   ├── main.rs              # CLI 入口
│   ├── cli.rs               # 命令列參數解析
│   ├── commands/            # 命令實作
│   │   ├── mod.rs
│   │   ├── add.rs           # add 命令
│   │   ├── list.rs          # list 命令
│   │   ├── search.rs        # search 命令
│   │   ├── info.rs          # info 命令
│   │   ├── update.rs        # update 命令
│   │   ├── install.rs       # install 命令
│   │   ├── upgrade.rs       # upgrade 命令
│   │   ├── delete.rs        # del 命令
│   │   └── setup_path.rs    # setup-path 命令
│   ├── core/                # 核心功能
│   │   ├── mod.rs
│   │   ├── config.rs        # 配置管理
│   │   ├── manifest.rs      # Manifest 資料結構
│   │   ├── platform.rs      # 平台偵測與匹配
│   │   └── paths.rs         # 路徑管理
│   ├── providers/           # 來源提供者
│   │   ├── mod.rs
│   │   ├── base.rs          # Provider trait
│   │   └── github.rs        # GitHub 實作
│   ├── downloader/          # 下載模組
│   │   ├── mod.rs
│   │   └── multi_thread.rs  # 多執行緒下載
│   ├── installer/           # 安裝模組
│   │   ├── mod.rs
│   │   ├── extractor.rs     # 壓縮檔解壓
│   │   ├── symlink.rs       # 符號連結 (Unix)
│   │   └── shim.rs          # Shim 建立 (Windows)
│   └── utils/               # 工具函數
│       ├── mod.rs
│       ├── http.rs          # HTTP 客戶端
│       ├── json.rs          # JSON 處理
│       └── progress.rs      # 進度顯示
├── tests/                   # 整合測試
│   ├── integration_test.rs
│   └── fixtures/
└── scripts/                 # 安裝腳本
    ├── install.sh           # Unix 安裝
    └── install.ps1          # Windows 安裝
```

### 資料結構設計

#### sources.json (來源庫)
```json
{
  "packages": [
    {
      "name": "wenpm",
      "description": "Wen Package Manager",
      "repo": "https://github.com/superyngo/WenPM",
      "homepage": "https://github.com/superyngo/WenPM",
      "license": "MIT",
      "latest": "0.1.0",
      "updated_at": "2025-01-19T10:00:00Z",
      "platforms": {
        "windows-x86_64": {
          "url": "https://github.com/superyngo/WenPM/releases/download/v0.1.0/wenpm-windows-x86_64.zip",
          "size": 2500000
        },
        "linux-x86_64-musl": {
          "url": "https://github.com/superyngo/WenPM/releases/download/v0.1.0/wenpm-linux-x86_64-musl.tar.gz",
          "size": 2300000
        },
        "macos-aarch64": {
          "url": "https://github.com/superyngo/WenPM/releases/download/v0.1.0/wenpm-macos-aarch64.tar.gz",
          "size": 2400000
        }
      }
    }
  ],
  "last_updated": "2025-01-19T10:00:00Z"
}
```

#### installed.json (已安裝資訊)
```json
{
  "packages": {
    "wenpm": {
      "version": "0.1.0",
      "platform": "windows-x86_64",
      "installed_at": "2025-01-19T12:00:00Z",
      "install_path": "C:\\Users\\user\\.wenpm\\apps\\wenpm",
      "files": [
        "bin/wenpm.exe"
      ]
    }
  }
}
```

#### 目錄結構 (~/.wenpm/)
```
~/.wenpm/
├── sources.json           # 來源庫
├── installed.json         # 已安裝資訊
├── bin/                   # Symlink/Shim 目錄（加入 PATH）
│   ├── wenpm -> ../apps/wenpm/bin/wenpm  (Unix)
│   └── wenpm.cmd                         (Windows)
├── apps/                  # 應用程式安裝目錄
│   ├── wenpm/
│   │   └── bin/
│   │       └── wenpm.exe
│   └── wedi/
│       ├── bin/
│       │   └── wedi.exe
│       └── data/
└── cache/                 # 下載快取
    └── downloads/
```

---

## 🎯 開發階段

### Phase 1: 基礎架構 (Week 1-2)

#### 1.1 專案初始化
- [x] 決策制定
- [ ] 建立 Cargo 專案
- [ ] 設定專案結構
- [ ] 配置 CI/CD (GitHub Actions)
- [ ] 撰寫 README.md

#### 1.2 核心模組實作
- [ ] **config.rs**: 配置管理
  - 載入/儲存 sources.json
  - 載入/儲存 installed.json
  - 目錄初始化 (~/.wenpm/)

- [ ] **manifest.rs**: 資料結構定義
  - `Package` struct
  - `Platform` struct
  - `SourceManifest` struct
  - `InstalledManifest` struct
  - JSON 序列化/反序列化

- [ ] **platform.rs**: 平台偵測
  - 偵測當前系統 (OS + Architecture)
  - 實作 `current_platform()` 函數
  - 平台字串標準化

- [ ] **paths.rs**: 路徑管理
  - 取得 home 目錄
  - 建立標準路徑常數
  - 路徑驗證與創建

#### 1.3 CLI 框架
- [ ] **cli.rs**: 使用 clap 定義命令
  ```rust
  wenpm add <url>...
  wenpm list
  wenpm search <name>...
  wenpm info <name>...
  wenpm update
  wenpm install <name>...
  wenpm upgrade [all|<name>...]
  wenpm del <name>...
  wenpm setup-path
  ```

---

### Phase 2: GitHub Provider (Week 2-3)

#### 2.1 HTTP 客戶端
- [ ] **http.rs**: 使用 reqwest
  - GET 請求封裝
  - User-Agent 設定
  - 錯誤處理
  - Rate limit 檢查

#### 2.2 GitHub API 整合
- [ ] **github.rs**: Provider 實作
  - 實作 `SourceProvider` trait
  - API 端點定義
  - 解析 latest release JSON
  - 解析 assets 列表
  - 提取版本號、描述、授權

#### 2.3 Binary 選擇邏輯
- [ ] **platform.rs**: Binary 匹配演算法
  - 關鍵字權重計算
  - 平台識別規則
    - Windows: `["windows", "win64", "pc-windows"]`
    - Linux: `["linux", "unknown-linux"]` + musl 優先
    - macOS: `["darwin", "macos", "apple"]`
  - 架構識別: `["x86_64", "aarch64", "arm64"]`
  - 排除規則: `["source", ".deb", ".rpm", ".sha256"]`
  - 檔案格式優先級: `tar.gz > zip > 7z`

---

### Phase 3: add 命令實作 (Week 3-4)

#### 3.1 單一 URL 處理
- [ ] **add.rs**: 基本功能
  - 解析 GitHub URL
  - 呼叫 GitHub Provider
  - 取得 release metadata
  - 選擇各平台 binary
  - 更新 sources.json

#### 3.2 批次處理
- [ ] 支援多個 URL (空格分隔)
- [ ] 使用 Tokio 多執行緒並行處理
- [ ] 進度顯示 (使用 indicatif)

#### 3.3 檔案來源支援
- [ ] `--source <file>` 參數
- [ ] 讀取本地文字檔
- [ ] 讀取線上 URL 文字檔
- [ ] 每行一個 GitHub URL

#### 3.4 錯誤處理
- [ ] 無效 URL → 顯示錯誤
- [ ] 沒有 binary → 顯示警告並跳過
- [ ] API 失敗 → 顯示錯誤
- [ ] 彙總結果報告

---

### Phase 4: list/search/info 命令 (Week 4)

#### 4.1 list 命令
- [ ] **list.rs**: 實作
  - 讀取 sources.json
  - 過濾當前平台支援的 packages
  - 格式化輸出 (表格)
    ```
    NAME    VERSION  SIZE     DESCRIPTION
    wenpm   0.1.0    2.5 MB   Wen Package Manager
    wedi    0.1.12   1.2 MB   A minimalist text editor
    ```

#### 4.2 search 命令
- [ ] **search.rs**: 實作
  - 支援多個關鍵字
  - 支援萬用字元 `*`
  - 名稱模糊匹配
  - 同樣輸出格式

#### 4.3 info 命令
- [ ] **info.rs**: 實作
  - 讀取 sources.json + installed.json
  - 比對已安裝版本 vs 最新版本
  - 輸出詳細資訊
    ```
    Name: wedi
    Description: A minimalist text editor
    Repository: https://github.com/superyngo/wedi
    Latest Version: 0.1.12
    Installed Version: 0.1.10 (upgrade available)
    Platform: windows-x86_64
    Size: 1.2 MB
    ```

---

### Phase 5: 下載與安裝 (Week 5-6)

#### 5.1 下載模組
- [ ] **downloader/multi_thread.rs**
  - 使用 reqwest 非同步下載
  - 進度條顯示 (indicatif)
  - 斷點續傳支援 (可選)
  - 下載到 `~/.wenpm/cache/downloads/`

#### 5.2 解壓模組
- [ ] **installer/extractor.rs**
  - 支援 `.zip` (使用 `zip` crate)
  - 支援 `.tar.gz` (使用 `tar` + `flate2`)
  - 支援 `.tar.xz` (使用 `tar` + `xz2`)
  - 偵測壓縮檔結構
  - 處理三種情況:
    1. 單一執行檔 → 移至 `bin/`
    2. 標準目錄結構 → 保持結構
    3. 扁平結構 → 建立 `bin/` 並移動執行檔

#### 5.3 執行檔處理
- [ ] 偵測執行檔
  - Windows: `*.exe`
  - Unix: 檢查 executable bit 或常見名稱
- [ ] 設定執行權限 (Unix)
  ```rust
  #[cfg(unix)]
  fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
  ```

#### 5.4 Symlink/Shim 建立
- [ ] **installer/symlink.rs** (Unix)
  ```rust
  symlink("../apps/wedi/bin/wedi", "~/.wenpm/bin/wedi")?;
  ```

- [ ] **installer/shim.rs** (Windows)
  ```cmd
  @echo off
  "%~dp0..\apps\wedi\bin\wedi.exe" %*
  ```

#### 5.5 install 命令
- [ ] **install.rs**: 整合流程
  1. 檢查 sources.json 是否存在 package
  2. 檢查當前平台是否支援
  3. 下載 binary
  4. 解壓到 `~/.wenpm/apps/<name>/`
  5. 建立 symlink/shim
  6. 更新 installed.json
  7. 清理下載快取

- [ ] 支援批次安裝
- [ ] 支援萬用字元 `*`

---

### Phase 6: update/upgrade 命令 (Week 6)

#### 6.1 update 命令
- [ ] **update.rs**: 更新 sources.json
  - 讀取所有已記錄的 repos
  - 使用多執行緒並行更新
  - 重新取得 latest release
  - 更新 metadata
  - 顯示更新統計

#### 6.2 upgrade 命令
- [ ] **upgrade.rs**: 更新已安裝 packages
  - 比對 installed.json vs sources.json
  - 找出可更新的 packages
  - `upgrade` → 列出可更新項目
  - `upgrade all` → 更新全部
  - `upgrade <name>...` → 更新指定項目
  - 重複 install 流程（覆蓋舊版）

#### 6.3 upgrade self
- [ ] 特殊處理 wenpm 自身更新
  - **Unix**: 直接覆蓋執行檔
    ```rust
    fs::copy("wenpm.new", "~/.wenpm/bin/wenpm")?;
    ```
  - **Windows**: 重命名 + 清理腳本
    ```rust
    fs::rename("wenpm.exe", "wenpm.old.exe")?;
    fs::rename("wenpm.new.exe", "wenpm.exe")?;
    create_cleanup_script()?;
    exit(0);
    ```

---

### Phase 7: delete 命令 (Week 7)

#### 7.1 delete 命令實作
- [ ] **delete.rs**
  - 檢查 installed.json
  - 刪除 `~/.wenpm/apps/<name>/`
  - 刪除 symlink/shim
  - 更新 installed.json
  - 支援批次刪除
  - 支援萬用字元 `*`

#### 7.2 安全檢查
- [ ] 防止刪除 wenpm 自身（除非使用 `--force`）
- [ ] 確認提示（除非使用 `--yes`）

---

### Phase 8: 進階功能 (Week 7-8)

#### 8.1 setup-path 命令
- [ ] **setup_path.rs**
  - **Unix**: 偵測 shell (bash/zsh/fish)
  - 附加到配置檔
    ```bash
    echo 'export PATH="$HOME/.wenpm/bin:$PATH"' >> ~/.bashrc
    ```
  - **Windows**: 使用 `setx` 修改環境變數
    ```powershell
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    ```

#### 8.2 進度與 UI 優化
- [ ] **progress.rs**: 使用 indicatif
  - 下載進度條
  - 多任務進度顯示
  - Spinner 動畫

- [ ] 彩色輸出 (使用 colored)
  - ✓ 成功訊息 (綠色)
  - ⚠ 警告訊息 (黃色)
  - ✗ 錯誤訊息 (紅色)

#### 8.3 日誌系統
- [ ] 使用 `env_logger` 或 `tracing`
- [ ] 支援 `-v` / `--verbose` 參數
- [ ] 錯誤詳細追蹤

---

### Phase 9: 測試 (Week 8-9)

#### 9.1 單元測試
- [ ] 每個模組編寫單元測試
- [ ] Platform 匹配邏輯測試
- [ ] JSON 序列化測試
- [ ] Binary 選擇演算法測試

#### 9.2 整合測試
- [ ] `tests/integration_test.rs`
- [ ] 模擬完整工作流程
  ```rust
  #[test]
  fn test_full_workflow() {
      // add → list → install → info → upgrade → del
  }
  ```

#### 9.3 手動測試矩陣
| 平台 | 測試項目 |
|------|---------|
| Windows 10/11 | 所有命令 + PATH 設定 |
| Ubuntu 22.04 | 所有命令 + symlink |
| macOS (Intel) | 所有命令 + symlink |
| macOS (ARM) | 所有命令 + symlink |

---

### Phase 10: 部署與發布 (Week 9-10)

#### 10.1 安裝腳本
- [ ] **scripts/install.sh** (Unix)
  ```bash
  #!/bin/bash
  # 偵測平台、下載 binary、解壓、設定 PATH、add 自身
  ```

- [ ] **scripts/install.ps1** (Windows)
  ```powershell
  # 下載、解壓、設定 PATH、add 自身
  ```

#### 10.2 GitHub Actions CI/CD
- [ ] `.github/workflows/ci.yml`
  - 多平台編譯 (Windows, Linux, macOS)
  - 執行測試
  - 產生 Release artifacts

- [ ] `.github/workflows/release.yml`
  - Tag push 時自動 Release
  - 上傳編譯好的 binaries
  - 自動產生 Release Notes

#### 10.3 文件撰寫
- [ ] **README.md**
  - 專案介紹
  - 快速開始
  - 命令參考
  - 範例

- [ ] **CONTRIBUTING.md**
  - 開發指南
  - 提交規範

- [ ] **docs/COMMANDS.md**
  - 詳細命令文檔

#### 10.4 社群管理機制
- [ ] 建立 `urls.txt` 在 repo
  - 維護可 add 的 URL 清單

- [ ] GitHub Issue Template
  - 建議新 package 的模板

- [ ] **未來**: GitHub Actions 自動處理 issue
  - 觸發 workflow 驗證 URL
  - 自動加入 `urls.txt`
  - 自動關閉 issue

---

## 🔧 技術選型

### 核心依賴

```toml
[dependencies]
# CLI 框架
clap = { version = "4.5", features = ["derive"] }

# HTTP 客戶端
reqwest = { version = "0.12", features = ["json", "blocking"] }

# 非同步運行時
tokio = { version = "1.35", features = ["full"] }

# JSON 處理
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 壓縮檔處理
zip = "0.6"
tar = "0.4"
flate2 = "1.0"
xz2 = "0.1"

# 進度條
indicatif = "0.17"

# 彩色輸出
colored = "2.1"

# 錯誤處理
anyhow = "1.0"
thiserror = "1.0"

# 日誌
env_logger = "0.11"
log = "0.4"

# 其他工具
chrono = "0.4"         # 時間處理
dirs = "5.0"           # 跨平台目錄
glob = "0.3"           # 萬用字元匹配

[dev-dependencies]
tempfile = "3.8"       # 測試用臨時檔案
mockito = "1.2"        # HTTP mock
```

### 編譯優化

```toml
[profile.release]
opt-level = "z"        # 最小化 binary 大小
lto = true             # Link-time optimization
codegen-units = 1      # 單一 codegen unit
strip = true           # 移除符號表
panic = "abort"        # Panic 時直接 abort
```

---

## 📊 開發時程表

| 階段 | 時間 | 重點任務 | 交付產出 |
|------|------|---------|---------|
| Phase 1 | Week 1-2 | 基礎架構 | 專案骨架、核心模組 |
| Phase 2 | Week 2-3 | GitHub Provider | API 整合、Binary 選擇 |
| Phase 3 | Week 3-4 | add 命令 | 可解析 GitHub repo |
| Phase 4 | Week 4 | 查詢命令 | list/search/info |
| Phase 5 | Week 5-6 | 安裝功能 | install 命令完整流程 |
| Phase 6 | Week 6 | 更新功能 | update/upgrade 命令 |
| Phase 7 | Week 7 | 刪除功能 | delete 命令 |
| Phase 8 | Week 7-8 | 進階功能 | setup-path、UI 優化 |
| Phase 9 | Week 8-9 | 測試 | 單元測試、整合測試 |
| Phase 10 | Week 9-10 | 部署發布 | v0.1.0 Release |

**總計**: 10 週 (約 2.5 個月)

---

## ✅ 驗收標準

### MVP (v0.1.0) 必須達成:
- [x] 所有決策已制定
- [ ] 可 `add` GitHub repo 並解析 binary
- [ ] 可 `list` 可用 packages
- [ ] 可 `install` package 到本機
- [ ] 可 `upgrade self` 自我更新
- [ ] 支援 Windows/Linux/macOS
- [ ] 支援 .zip, .tar.gz 格式
- [ ] PATH 設定機制運作正常
- [ ] 通過所有平台測試
- [ ] 提供安裝腳本
- [ ] 基本文檔完整

### v0.2.0 計畫功能:
- [ ] Checksum (SHA256) 驗證
- [ ] GitLab 支援
- [ ] Gitea/self-hosted 支援
- [ ] 更智慧的 binary 偵測
- [ ] 自動化社群 URL 提交機制

---

## 🚀 快速開始開發

```bash
# 1. 建立專案
cargo new wenpm
cd wenpm

# 2. 複製此開發計畫
cp ../DEVELOPMENT_PLAN.md .

# 3. 建立目錄結構
mkdir -p src/{commands,core,providers,downloader,installer,utils}
mkdir -p tests/fixtures
mkdir -p scripts

# 4. 開始 Phase 1 開發
# 編輯 Cargo.toml 加入依賴
# 實作 core/config.rs

# 5. 執行測試
cargo test

# 6. 本地執行
cargo run -- add https://github.com/superyngo/wedi
```

---

## 📝 備註

### 開發原則
1. **測試驅動開發 (TDD)**: 先寫測試，再寫實作
2. **錯誤處理優先**: 使用 `Result<T, E>` 明確處理錯誤
3. **文檔註解**: 所有 public API 必須有 rustdoc 註解
4. **程式碼審查**: 每個 PR 需要經過審查
5. **語義化版本**: 遵循 SemVer 2.0

### Git 工作流程
```bash
main (穩定版本)
  ├── develop (開發分支)
  │   ├── feature/add-command
  │   ├── feature/install-command
  │   └── feature/upgrade-self
  └── release/v0.1.0
```

### Commit 規範
```
feat: 新增 add 命令基本功能
fix: 修正 Windows 路徑處理 bug
docs: 更新 README 安裝說明
test: 加入 platform 模組單元測試
refactor: 重構 binary 選擇邏輯
```

---

**Last Updated**: 2025-01-19
**Version**: 1.0
**Author**: WenPM Development Team
