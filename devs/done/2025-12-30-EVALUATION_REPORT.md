# Wenget 功能增強開發計畫 - 執行評估報告

**評估日期**: 2025-12-30  
**評估者**: Antigravity AI  
**專案版本**: v0.6.3

---

## 執行摘要

本次評估針對 `2025-12-29-DEV_PLAN.md` 中列出的四項主要功能需求進行驗證。經過代碼審查、編譯測試和功能驗證，發現部分功能已實作，但仍有關鍵功能缺失。

### 整體完成度: **50%** (2/4 項完成)

---

## 詳細評估結果

### ✅ 項目 2: wenget list --all 顯示 SOURCE 欄位
**狀態**: ✅ **已完成並驗證**

**驗證結果**:
- ✅ 代碼實作位置: `src/commands/list.rs` (第 38-76 行)
- ✅ 已在 `list_installed_packages()` 函數中加入 SOURCE 欄位顯示
- ✅ 支援三種來源類型:
  - Bucket 來源: 顯示 bucket 名稱
  - DirectRepo: 顯示 "url"
  - Script: 顯示腳本類型 (powershell/bash)
- ✅ 輸出格式正確，包含顏色標記 (cyan)

**代碼證據**:
```rust
// 第 37-44 行
println!(
    "{:<20} {:<15} {:<10} {:<12} {}",
    "NAME".bold(),
    "COMMAND".bold(),
    "VERSION".bold(),
    "SOURCE".bold(),  // ← SOURCE 欄位已加入
    "DESCRIPTION".bold()
);

// 第 54-60 行
let source_display = match &pkg.source {
    PackageSource::Bucket { name } => name.clone(),
    PackageSource::DirectRepo { .. } => "url".to_string(),
    PackageSource::Script { script_type, .. } => {
        script_type.display_name().to_lowercase().to_string()
    }
};
```

---

### ✅ 項目 4: 子命令別名優化
**狀態**: ⚠️ **部分完成** (1/2 完成)

**驗證結果**:

#### ❌ info 新增別名 i
**狀態**: ❌ **未實作**

**問題**: 
- `src/cli.rs` 第 53 行的 `Info` 命令沒有 `visible_alias` 屬性
- 執行 `wenget info --help` 沒有顯示別名

**預期代碼**:
```rust
/// Show package information from cache or GitHub URL
#[command(visible_alias = "i")]  // ← 缺少此行
Info {
    names: Vec<String>,
},
```

#### ❌ del 新增別名 rm, uninstall
**狀態**: ⚠️ **僅部分實作**

**問題**:
- `src/cli.rs` 第 77 行的 `Del` 命令只有 `visible_alias = "remove"`
- 缺少 `rm` 和 `uninstall` 別名

**當前代碼**:
```rust
/// Delete (remove) installed packages
#[command(visible_alias = "remove")]  // ← 只有 remove，缺少 rm 和 uninstall
Del {
    ...
}
```

**預期代碼**:
```rust
/// Delete (remove) installed packages
#[command(visible_alias = "remove")]
#[command(visible_alias = "rm")]
#[command(visible_alias = "uninstall")]
Del {
    ...
}
```

---

### ❌ 項目 3: add 增加 --platform/-p 參數
**狀態**: ❌ **未實作**

**驗證結果**:
- ❌ `src/cli.rs` 的 `Add` 命令結構中沒有 `platform` 參數
- ❌ 執行 `wenget add --help` 沒有顯示 `--platform` 或 `-p` 選項
- ❌ `src/commands/add.rs` 中沒有處理平台選擇的邏輯

**當前 Add 命令參數**:
```rust
Add {
    names: Vec<String>,
    #[arg(short = 'y', long)]
    yes: bool,
    #[arg(short = 'n', long = "name")]
    script_name: Option<String>,
    // ← 缺少 platform 參數
}
```

**需要新增**:
```rust
Add {
    names: Vec<String>,
    #[arg(short = 'y', long)]
    yes: bool,
    #[arg(short = 'n', long = "name")]
    script_name: Option<String>,
    /// Specify target platform (e.g., windows-x64, linux-x64)
    #[arg(short = 'p', long = "platform")]
    platform: Option<String>,  // ← 需要新增
}
```

**影響範圍**:
1. CLI 參數解析 (`src/cli.rs`)
2. Add 命令處理 (`src/commands/add.rs` - `run()` 函數簽名)
3. Package 安裝邏輯 (`install_package()` 函數需要接受並使用指定平台)

---

### ❌ 項目 1: add 支援 binary/zip 檔案路徑或 direct url
**狀態**: ⚠️ **部分支援**

**驗證結果**:

#### ✅ 已支援功能:
- ✅ Script 檔案 (PowerShell/Bash) 的本地路徑和 URL
- ✅ GitHub Repository URL
- ✅ 從 cache/bucket 安裝 package

**代碼證據** (`src/commands/add.rs`):
```rust
// 第 49-51 行: 已區分 script 和 package 輸入
let script_inputs: Vec<&String> = names.iter().filter(|n| is_script_input(n)).collect();
let package_inputs: Vec<&String> = names.iter().filter(|n| !is_script_input(n)).collect();

// 第 94-115 行: 已支援 script URL 和本地路徑
let is_url = input.starts_with("http://") || input.starts_with("https://");
let content = if is_url {
    download_script(input)?
} else {
    let path = Path::new(input);
    read_local_script(path)?
};
```

#### ❌ 缺少功能:
- ❌ **直接安裝本地 binary 檔案** (例如: `wenget add ./ripgrep.exe`)
- ❌ **直接安裝本地 zip/tar.gz 檔案** (例如: `wenget add ./app.zip`)
- ❌ **直接從 URL 下載並安裝 binary** (例如: `wenget add https://example.com/app.exe`)
- ❌ **直接從 URL 下載並安裝 archive** (例如: `wenget add https://example.com/app.zip`)

**當前限制**:
- 只能安裝 GitHub repository (需要解析 release assets)
- 只能安裝 script 檔案 (.ps1, .sh)
- 不支援直接指定 binary 或 archive 檔案

**需要實作**:
1. 檢測輸入是否為本地檔案路徑或直接下載 URL
2. 判斷檔案類型 (binary vs archive)
3. 對於 binary: 直接複製到 app 目錄並創建 shim
4. 對於 archive: 解壓縮並安裝
5. 處理自訂命令名稱 (使用 `--name` 參數)

---

## 編譯測試結果

### ✅ 編譯狀態
```
Compiling wenget v0.6.3
Finished `release` profile [optimized] target(s) in 1m 54s
Exit code: 0
```
**結論**: ✅ 代碼編譯成功，無語法錯誤

---

## 功能測試結果

### 測試 1: 檢查命令別名

#### Test 1.1: info 別名
```bash
$ wenget info --help
# 輸出中沒有顯示別名 [aliases: i]
```
**結果**: ❌ 失敗 - 缺少 `i` 別名

#### Test 1.2: del 別名
```bash
$ wenget del --help
# 輸出中沒有顯示別名 [aliases: rm, uninstall]
```
**結果**: ❌ 失敗 - 只有 `remove` 別名，缺少 `rm` 和 `uninstall`

### 測試 2: 檢查 --platform 參數
```bash
$ wenget add --help
# 輸出中沒有 --platform/-p 選項
```
**結果**: ❌ 失敗 - 參數未實作

---

## 建議修正措施

### 優先級 1 (高): 完成子命令別名
**預估工時**: 5 分鐘

**修改檔案**: `src/cli.rs`

```rust
// 第 52-56 行
/// Show package information from cache or GitHub URL
#[command(visible_alias = "i")]  // ← 新增
Info {
    names: Vec<String>,
},

// 第 76-89 行
/// Delete (remove) installed packages
#[command(visible_alias = "remove")]
#[command(visible_alias = "rm")]        // ← 新增
#[command(visible_alias = "uninstall")] // ← 新增
Del {
    names: Vec<String>,
    #[arg(short = 'y', long)]
    yes: bool,
    #[arg(short, long)]
    force: bool,
},
```

### 優先級 2 (高): 實作 --platform 參數
**預估工時**: 30 分鐘

**步驟**:
1. 修改 `src/cli.rs` - 新增 `platform` 參數到 `Add` 命令
2. 修改 `src/commands/add.rs` - 更新 `run()` 函數簽名
3. 修改 `install_package()` - 使用指定平台而非自動檢測
4. 新增驗證邏輯 - 檢查指定平台是否存在於 package platforms 中

### 優先級 3 (中): 支援本地 binary/archive 安裝
**預估工時**: 2-3 小時

**步驟**:
1. 新增檔案類型檢測函數 (binary vs archive vs script)
2. 實作 `install_local_binary()` 函數
3. 實作 `install_local_archive()` 函數
4. 實作 `install_from_direct_url()` 函數
5. 整合到 `add` 命令的主流程
6. 新增測試案例

---

## 總結

### 已完成項目 ✅
1. ✅ **wenget list --all 顯示 SOURCE 欄位** - 完全實作且運作正常

### 部分完成項目 ⚠️
2. ⚠️ **子命令別名優化** - 僅有 `del` 的 `remove` 別名，缺少其他別名
3. ⚠️ **add 支援檔案/URL** - 僅支援 script，不支援 binary/archive

### 未完成項目 ❌
4. ❌ **add --platform 參數** - 完全未實作

### 下一步行動建議

1. **立即修正**: 完成子命令別名 (5 分鐘工作)
2. **短期目標**: 實作 --platform 參數 (30 分鐘工作)
3. **中期目標**: 完整支援本地 binary/archive 安裝 (2-3 小時工作)

### 風險評估

- **低風險**: 別名修正 - 僅需修改 CLI 定義
- **中風險**: --platform 參數 - 需要修改多個函數簽名
- **高風險**: binary/archive 支援 - 需要新增大量邏輯，可能影響現有功能

---

**報告結束**
