# Wenget 重構需求評估報告

## 評估日期
2026-01-02

## 概述
本文檔評估四個主要重構需求的可行性、複雜度和實施建議。

---

## 需求 1: Root/Administrator 執行時的系統級安裝

### 需求描述
- 偵測 wenget 是否以 root/Administrator 身分執行
- Linux: 安裝至 `/opt/wenget/app`，連結至 `/usr/local/bin`
- Windows: 安裝至 `%ProgramW6432%\wenget\app`，將 `%ProgramW6432%\wenget\bin` 加入系統 PATH

### 可行性評估: ✅ 可行

### 技術分析

#### 權限偵測
**Linux/Unix:**
```rust
fn is_running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}
```

**Windows:**
```rust
// 需要 windows-sys crate
fn is_running_as_admin() -> bool {
    // 檢查當前 token 是否有 admin 權限
    // 可使用 is_elevated crate 或自行實現
}
```

#### 路徑管理重構
需要修改 `src/core/paths.rs` 的 `WenPaths` 結構：

**目前實作:**
- 固定使用 `~/.wenget` (用戶主目錄)

**建議實作:**
```rust
pub struct WenPaths {
    root: PathBuf,
    is_system_install: bool,
}

impl WenPaths {
    pub fn new() -> Result<Self> {
        let (root, is_system_install) = if is_elevated() {
            // 系統級安裝
            #[cfg(unix)]
            let root = PathBuf::from("/opt/wenget");
            
            #[cfg(windows)]
            let root = {
                let program_files = std::env::var("ProgramW6432")
                    .unwrap_or_else(|_| "C:\\Program Files".to_string());
                PathBuf::from(program_files).join("wenget")
            };
            
            (root, true)
        } else {
            // 用戶級安裝（現有邏輯）
            let home = dirs::home_dir().context("...")?;
            (home.join(".wenget"), false)
        };
        
        Ok(Self { root, is_system_install })
    }
    
    pub fn bin_dir(&self) -> PathBuf {
        if self.is_system_install {
            #[cfg(unix)]
            return PathBuf::from("/usr/local/bin");
            
            #[cfg(windows)]
            return self.root.join("bin");
        }
        self.root.join("bin")
    }
}
```

#### Windows PATH 登錄
```rust
// 需要 winreg crate
fn add_to_system_path(path: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let env = hklm.open_subkey_with_flags(
            "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
            KEY_READ | KEY_WRITE,
        )?;
        
        let current_path: String = env.get_value("Path")?;
        let new_path = format!("{};{}", current_path, path.display());
        env.set_value("Path", &new_path)?;
    }
    Ok(())
}
```

### 複雜度評估: 🟡 中等

**預估工時:** 2-3 天

**影響範圍:**
- `src/core/paths.rs` - 核心路徑邏輯
- `src/commands/init.rs` - 初始化邏輯
- `src/installer/*.rs` - 所有安裝器模組
- 需要新增 crate 依賴:
  - `libc` (Unix 權限)
  - `is_elevated` 或 `windows-sys` (Windows 權限)
  - `winreg` (Windows 註冊表)

### 風險與注意事項

1. **向後兼容性**
   - 現有用戶級安裝不受影響
   - 需要在文檔中明確說明兩種安裝模式的差異

2. **權限問題**
   - 系統級安裝需要管理員權限
   - 解除安裝時也需要相同權限
   - 建議在 CLI 顯示清楚的提示

3. **Linux 符號連結**
   - `/usr/local/bin` 可能已有同名檔案
   - 需要衝突檢測和處理機制

4. **Windows PATH 長度限制**
   - 舊版 Windows PATH 有 2048 字元限制
   - 需要檢查和警告

### 建議實施步驟

1. 新增權限偵測功能
2. 重構 `WenPaths` 支援雙模式
3. 修改安裝邏輯
4. 實作 Windows PATH 註冊
5. 實作 Linux 符號連結到 `/usr/local/bin`
6. 新增測試（需要管理員權限的整合測試）
7. 更新文檔

---

## 需求 2: Windows PowerShell 腳本優先使用 pwsh

### 需求描述
自動偵測系統是否有 `pwsh`，若有則優先使用 `pwsh` 而非 `powershell` 執行腳本

### 可行性評估: ✅ 可行且簡單

### 技術分析

目前實作（`src/installer/script.rs` line 225-229）:
```rust
ScriptType::PowerShell => {
    format!(
        "@echo off\r\npowershell -NoProfile -ExecutionPolicy Bypass -File \"%~dp0{}\" %*\r\n",
        relative_path_str
    )
}
```

建議修改:
```rust
fn detect_powershell_command() -> &'static str {
    // 檢查 pwsh 是否可用
    if std::process::Command::new("pwsh")
        .arg("--version")
        .output()
        .is_ok()
    {
        "pwsh"
    } else {
        "powershell"
    }
}

// 在 create_script_shim_windows 中使用
ScriptType::PowerShell => {
    let ps_cmd = detect_powershell_command();
    format!(
        "@echo off\r\n{} -NoProfile -ExecutionPolicy Bypass -File \"%~dp0{}\" %*\r\n",
        ps_cmd, relative_path_str
    )
}
```

### 優化建議

**快取偵測結果:**
```rust
use std::sync::OnceLock;

static POWERSHELL_CMD: OnceLock<&'static str> = OnceLock::new();

fn get_powershell_command() -> &'static str {
    POWERSHELL_CMD.get_or_init(|| {
        if std::process::Command::new("pwsh")
            .arg("--version")
            .output()
            .is_ok()
        {
            "pwsh"
        } else {
            "powershell"
        }
    })
}
```

### 複雜度評估: 🟢 簡單

**預估工時:** 2-4 小時

**影響範圍:**
- `src/installer/script.rs` - 僅需修改 shim 生成邏輯
- 可選: Unix 平台的 PowerShell shim 也可應用相同邏輯（line 279-283）

### 建議實施步驟

1. 實作 `get_powershell_command()` 函數
2. 修改 Windows shim 生成邏輯
3. （可選）修改 Unix wrapper 生成邏輯
4. 新增單元測試
5. 更新 CHANGELOG

---

## 需求 3: Manifest 支援多平台同名腳本

### 需求描述
修改 `manifest-cache.json` 中 scripts 的格式，支援同源同名但不同平台的腳本。

**目前格式 (`ScriptItem`):**
```json
{
  "name": "rclonemm",
  "description": "...",
  "url": "https://gist.githubusercontent.com/.../rclonemm.sh",
  "script_type": "bash",
  "repo": "..."
}
```

**建議新格式:**
```json
{
  "name": "rclonemm",
  "description": "Manage rclone mount through ssh config.",
  "repo": "https://gist.github.com/superyngo/f74f4749882df654dfdf286b7f718a9e",
  "platforms": {
    "bash": {
      "url": "https://gist.githubusercontent.com/.../rclonemm.sh"
    },
    "powershell": {
      "url": "https://gist.githubusercontent.com/.../rclonemm.ps1"
    }
  },
  "source": {
    "type": "bucket",
    "name": "wenget"
  }
}
```

### 可行性評估: ✅ 可行

**注意:** 根據需求確認，此功能**不需要向後兼容性處理**，現有 manifest 將直接採用新格式。

### 技術分析

#### 1. 數據結構修改

**目前定義** (`src/core/manifest.rs` line 162-190):
```rust
pub struct ScriptItem {
    pub name: String,
    pub description: String,
    pub url: String,
    pub script_type: ScriptType,
    pub repo: String,
    // ...
}
```

**新定義:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptPlatform {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptItem {
    pub name: String,
    pub description: String,
    pub repo: String,
    
    // 支援多平台
    pub platforms: HashMap<ScriptType, ScriptPlatform>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

impl ScriptItem {
    /// 獲取適合當前平台的腳本類型和 URL
    pub fn get_compatible_script(&self) -> Option<(ScriptType, &ScriptPlatform)> {
        // 優先順序: 當前 OS 原生 -> 跨平台
        let preference_order = if cfg!(windows) {
            vec![ScriptType::PowerShell, ScriptType::Batch, ScriptType::Python, ScriptType::Bash]
        } else {
            vec![ScriptType::Bash, ScriptType::Python, ScriptType::PowerShell]
        };
        
        for script_type in preference_order {
            if script_type.is_supported_on_current_platform() {
                if let Some(platform) = self.platforms.get(&script_type) {
                    return Some((script_type, platform));
                }
            }
        }
        None
    }
    
    /// 獲取所有可用的腳本平台
    pub fn available_platforms(&self) -> Vec<ScriptType> {
        self.platforms.keys().cloned().collect()
    }
}
```

#### 2. 受影響的模組

- `src/core/manifest.rs` - 核心定義 ✏️
- `src/cache.rs` - Cache 結構中的 `CachedScript` ✏️
- `src/commands/add.rs` - 安裝腳本時的邏輯 ✏️
- `src/commands/info.rs` - 顯示腳本資訊 ✏️
- `src/commands/list.rs` - 列出腳本 ✏️
- `src/commands/search.rs` - 搜尋腳本 ✏️

### 複雜度評估: � 中等

**預估工時:** 2-3 天（不需向後兼容性大幅簡化）

**影響範圍:** 廣泛，但不需要兼容層降低複雜度

### 風險與注意事項

1. **破壞性變更**
   - ⚠️ 此為破壞性更新，舊 manifest 將無法使用
   - 需要明確版本號升級 (例如 v0.x → v1.0)
   - 需要在 CHANGELOG 和文檔中明確說明

2. **安裝時平台選擇**
   - 需要智能選擇最適合當前平台的腳本版本
   - 允許用戶手動指定平台（`--script-type` flag）

3. **用戶體驗**
   - 需要在 `info` 命令中清楚顯示所有可用平台
   - 安裝時告知用戶選擇了哪個版本
   - 當沒有適合的平台時給出清晰的錯誤訊息

### 建議實施步驟

1. 定義新的數據結構（`ScriptPlatform` 和更新的 `ScriptItem`）
2. 修改 `ManifestCache` 和 `CachedScript` 以支援新格式
3. 實作智能平台選擇邏輯 (`get_compatible_script()`)
4. 更新所有命令以支援新格式:
   - `add`: 自動選擇最佳平台
   - `info`: 顯示所有可用平台
   - `list`: 標註多平台支援
   - `search`: 搜尋結果顯示平台資訊
5. 新增完整測試覆蓋
6. 更新版本號、文檔和範例
7. 更新 bucket manifest（配合需求 4）

---

## 需求 4: 新增 `bucket create` 命令

### 需求描述
新增 `wenget bucket create` 命令來生成 manifest.json，替代目前的 Python 腳本，並整合現有的解析功能。

**參數:**
- `--reposrc <URL|PATH>`: repos 源文件
- `--scriptsrc <URL|PATH>`: scripts 源文件
- `--output <PATH>`: 輸出的 manifest.json 路徑（預設: manifest.json）

**功能:**
- 解析 `sources_repos.txt`（GitHub repo URLs）
- 解析 `sources_scripts.txt`（Gist URLs 和 raw script URLs）
- 從 GitHub API 獲取 release 資訊
- 偵測平台和架構
- 產生 `manifest.json`（支援需求 3 的多平台腳本格式）

### 可行性評估: ✅ 可行，可重用大量現有代碼

**關鍵發現：** Wenget 已經實作了多數需要的功能！

### 可重用的現有組件

#### ✅ 已實作（可直接重用）

1. **GitHub Provider** (`src/providers/github.rs`)
   - ✅ `GitHubProvider::fetch_package()` - 完整的 repo 解析
   - ✅ `parse_github_url()` - URL 解析
   - ✅ `fetch_latest_release()` - 獲取最新 release
   - ✅ `fetch_repo_info()` - 獲取 repo 資訊
   - ✅ GitHub API 結構體 (`GitHubRelease`, `GitHubAsset`, `GitHubRepo`)

2. **平台偵測** (`src/core/platform.rs`)
   - ✅ `ParsedAsset::from_filename()` - 完整的平台偵測
   - ✅ `BinarySelector::extract_platforms()` - 平台提取
   - ✅ 支援所有主要平台和架構的關鍵字
   - ✅ Compiler/toolchain 優先級邏輯

3. **輸入偵測** (`src/installer/input_detector.rs`)
   - ✅ `detect_input_type()` - 區分 URL、本地檔案、套件名稱
   - ✅ Script 偵測邏輯

4. **Script 解析** (`src/installer/script.rs`)
   - ✅ `detect_script_type()` - 腳本類型偵測
   - ✅ `download_script()` - 下載腳本
   - ✅ `extract_script_name()` - 提取腳本名稱

5. **HTTP Client** (`src/utils/http_client.rs`)
   - ✅ 已有完整的 HTTP 客戶端
   - ✅ 支援 JSON 解析

#### ❌ 需要新增

1. **Gist API 支援**
   - 需要新增 Gist API 結構體和解析邏輯
   - 需要從 Gist 提取多個腳本檔案

2. **源文件解析器**
   - 簡單的文本檔案解析（去除註釋、空行）

3. **多平台腳本分組**
   - 同源 Gist 的不同平台腳本需要合併成單一 `ScriptItem`

### 技術分析

#### 新模組結構

```
src/
  commands/
    bucket/
      create.rs       ← 新增: bucket create 命令
  providers/
    github.rs         ← 現有: 已有完整實作
    gist.rs           ← 新增: Gist API 支援
    source_parser.rs  ← 新增: 源文件解析
```

#### 1. Gist Provider（新增）

```rust
// src/providers/gist.rs
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Gist {
    pub id: String,
    pub description: Option<String>,
    pub html_url: String,
    pub files: HashMap<String, GistFile>,
}

#[derive(Debug, Deserialize)]
pub struct GistFile {
    pub filename: String,
    pub raw_url: String,
    pub size: u64,
}

pub struct GistProvider {
    http: HttpClient,
}

impl GistProvider {
    pub fn new() -> Result<Self> {
        Ok(Self { http: HttpClient::new()? })
    }
    
    /// 解析 Gist URL 獲取 ID
    pub fn parse_gist_url(&self, url: &str) -> Result<String> {
        // https://gist.github.com/username/gist_id
        let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
        if parts.len() >= 2 {
            Ok(parts.last().unwrap().to_string())
        } else {
            anyhow::bail!("Invalid Gist URL: {}", url)
        }
    }
    
    /// 獲取 Gist 資訊
    pub fn fetch_gist(&self, gist_id: &str) -> Result<Gist> {
        let url = format!("https://api.github.com/gists/{}", gist_id);
        self.http.get_json(&url)
    }
    
    /// 從 Gist 提取腳本（支援多平台格式）
    pub fn extract_scripts(&self, gist: &Gist) -> Result<Vec<ScriptItem>> {
        use crate::installer::script::detect_script_type_from_extension;
        
        // 按腳本名稱分組（同名不同副檔名）
        let mut script_groups: HashMap<String, Vec<(ScriptType, GistFile)>> = HashMap::new();
        
        for (_, file) in &gist.files {
            if let Some(script_type) = detect_script_type_from_extension(&file.filename) {
                let name = file.filename.rsplit('.').nth(1)
                    .or_else(|| file.filename.split('.').next())
                    .unwrap_or(&file.filename)
                    .to_string();
                    
                script_groups.entry(name)
                    .or_insert_with(Vec::new)
                    .push((script_type, file.clone()));
            }
        }
        
        // 產生 ScriptItem（支援多平台）
        let mut scripts = Vec::new();
        for (name, files) in script_groups {
            let mut platforms = HashMap::new();
            for (script_type, file) in files {
                platforms.insert(script_type, ScriptPlatform {
                    url: file.raw_url.clone(),
                    checksum: None,
                });
            }
            
            scripts.push(ScriptItem {
                name,
                description: gist.description.clone().unwrap_or_default(),
                repo: gist.html_url.clone(),
                platforms,
                homepage: Some(gist.html_url.clone()),
                license: None,
            });
        }
        
        Ok(scripts)
    }
}
```

#### 2. 源文件解析器（新增）

```rust
// src/providers/source_parser.rs
pub struct SourceParser;

impl SourceParser {
    /// 解析源文件（去除註釋和空行）
    pub fn parse_source_file(content: &str) -> Vec<String> {
        content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(String::from)
            .collect()
    }
    
    /// 從 URL 或路徑讀取源文件
    pub fn read_source(input: &str) -> Result<String> {
        use crate::installer::input_detector::detect_input_type;
        use crate::installer::input_detector::InputType;
        
        match detect_input_type(input) {
            InputType::DirectUrl => {
                let http = HttpClient::new()?;
                http.get_text(input)
            }
            InputType::LocalFile | InputType::PackageName => {
                std::fs::read_to_string(input)
                    .with_context(|| format!("Failed to read file: {}", input))
            }
            _ => anyhow::bail!("Invalid source input: {}", input),
        }
    }
}
```

#### 3. Manifest 生成器（新增）

```rust
// src/commands/bucket/create.rs
use crate::providers::{GitHubProvider, GistProvider, SourceProvider};
use crate::providers::source_parser::SourceParser;
use crate::core::SourceManifest;

pub fn run_create(
    repo_source: Option<String>,
    script_source: Option<String>,
    output: PathBuf,
) -> Result<()> {
    let mut manifest = SourceManifest::new();
    
    // 處理 repos
    if let Some(repo_src) = repo_source {
        println!("📦 Processing repositories...");
        let content = SourceParser::read_source(&repo_src)?;
        let repo_urls = SourceParser::parse_source_file(&content);
        
        let github = GitHubProvider::new()?;
        for (idx, url) in repo_urls.iter().enumerate() {
            print!("  [{}/{}] Fetching {}...", idx + 1, repo_urls.len(), url);
            match github.fetch_package(url) {
                Ok(package) => {
                    println!(" ✓");
                    manifest.packages.push(package);
                }
                Err(e) => {
                    println!(" ✗ Error: {}", e);
                }
            }
            
            // Rate limiting: 簡單延遲
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
    
    // 處理 scripts
    if let Some(script_src) = script_source {
        println!("\n📜 Processing scripts...");
        let content = SourceParser::read_source(&script_src)?;
        let script_urls = SourceParser::parse_source_file(&content);
        
        let gist = GistProvider::new()?;
        for (idx, url) in script_urls.iter().enumerate() {
            print!("  [{}/{}] Fetching {}...", idx + 1, script_urls.len(), url);
            
            if url.contains("gist.github.com") {
                // Gist URL
                match gist.parse_gist_url(url)
                    .and_then(|id| gist.fetch_gist(&id))
                    .and_then(|g| gist.extract_scripts(&g))
                {
                    Ok(scripts) => {
                        println!(" ✓ ({} script(s))", scripts.len());
                        manifest.scripts.extend(scripts);
                    }
                    Err(e) => println!(" ✗ Error: {}", e),
                }
            } else {
                // Raw script URL
                // 直接下載並偵測類型
                // TODO: 實作單一腳本處理
            }
            
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
    
    // 寫入 manifest
    println!("\n💾 Writing manifest to {}...", output.display());
    let json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(&output, json)?;
    
    println!("✓ Generated manifest with {} packages and {} scripts",
        manifest.packages.len(),
        manifest.scripts.len()
    );
    
    Ok(())
}
```

### 複雜度評估: � 中等

**預估工時:** 4-5 天（大幅減少，因為可重用現有代碼）

**新增依賴:**
- 無需新增！現有依賴已足夠

**影響範圍:**
- ✅ 最小化：僅新增模組，不影響現有功能
- 新增檔案：
  - `src/providers/gist.rs`
  - `src/providers/source_parser.rs`
  - `src/commands/bucket/create.rs`
- 修改檔案：
  - `src/providers/mod.rs` - 匯出新模組
  - `src/commands/bucket.rs` - 新增 Create 命令
  - `src/cli.rs` - 新增 CLI 定義

### 優勢（相比 Python 腳本）

1. **統一語言和邏輯**
   - 完全重用 Wenget 的平台偵測邏輯
   - 不需要維護兩套獨立的平台關鍵字映射

2. **無需 async**
   - 使用同步 HTTP 請求即可
   - 簡單的 `thread::sleep` 做 rate limiting
   - 避免引入 tokio 的複雜性

3. **更好的錯誤處理**
   - Rust 的類型安全和錯誤處理
   - 更清晰的錯誤訊息

4. **內建整合**
   - 可直接呼叫 `wenget bucket create` 生成 manifest
   - 不需要額外安裝 Python 依賴

### 風險與注意事項

1. **GitHub API Rate Limiting**
   - 這部分未來會透過Action線上執行故不需擔心也不用處理

2. **錯誤處理**
   - 單一 repo/script 失敗不應中斷整個流程
   - 需要收集和報告所有錯誤

3. **與需求 3 的依賴**
   - **必須先完成需求 3** 才能正確生成多平台腳本格式
   - 或者在需求 3 完成前，先生成舊格式

### 建議實施步驟

1. **Phase 1: Gist 支援**（1-1.5 天）
   - 實作 `GistProvider`
   - Gist API 結構體和解析
   - 多平台腳本分組邏輯

2. **Phase 2: 源文件解析**（0.5 天）
   - 實作 `SourceParser`
   - 支援 URL 和本地檔案

3. **Phase 3: Manifest 生成器**（1.5-2 天）
   - 實作 `bucket create` 命令
   - 整合現有的 `GitHubProvider`
   - 錯誤處理和進度顯示

4. **Phase 4: CLI 整合**（0.5 天）
   - 新增 CLI 定義
   - 命令路由

5. **Phase 5: 測試與優化**（0.5-1 天）
   - 單元測試
   - 實際生成 manifest 測試
   - Rate limiting 調整
   - 文檔

---

## 總體評估與建議

### 優先級建議

1. **需求 2 (pwsh 優先)** - 🟢 立即實施
   - 工作量小，風險低
   - 對用戶體驗有明顯改善
   - 可以快速交付

2. **需求 1 (系統級安裝)** - 🟡 中期規劃
   - 重要功能，提升專業性
   - 需要仔細設計和測試
   - 建議在穩定版本中實施

3. **需求 3 (多平台腳本)** - 🟡 中期規劃，但應優先於需求 4
   - 架構性改進，長遠有益
   - **必須在需求 4 之前完成**（需求 4 依賴此格式）
   - 不需向後兼容性，大幅降低複雜度
   - 需要充分測試和文檔

4. **需求 4 (bucket create)** - � 中期規劃（依賴需求 3）
   - 可重用大量現有代碼，工作量中等
   - 無需引入 async，簡化實作
   - 建議在需求 3 完成後立即實施
   - 統一語言和邏輯，提升維護性

### 整體時程估算

- **需求 2**: 0.5 天
- **需求 1**: 3 天
- **需求 3**: 2.5 天（移除向後兼容性後）
- **需求 4**: 4.5 天（重用現有代碼）
- **測試、文檔、整合**: 2 天

**總計: 約 12.5 天** (約 2.5 週)

**節省時間:**
- ✅ 需求 3: 節省 1.5 天（移除向後兼容性）
- ✅ 需求 4: 節省 2 天（重用現有代碼，無需 async）

### 分階段實施建議

**Sprint 1 (Week 1):**
- ✅ 實作需求 2（0.5 天）
- 開始需求 1 的設計和實作（2.5 天）
- 開始需求 3 的設計（1 天）

**Sprint 2 (Week 2):**
- ✅ 完成需求 1（0.5 天）
- 完成需求 3 的實作和測試（2 天）
- 開始需求 4 的實作（2 天）

**Sprint 3 (Week 3 - 部分):**
- ✅ 完成需求 4（2.5 天）
- 整合測試和文檔（2 天）

### 依賴關係圖

```
需求 2 (pwsh優先)
  ↓ (獨立)
需求 1 (系統級安裝)
  ↓ (獨立)
需求 3 (多平台腳本)
  ↓ (強依賴：需求4產生新格式manifest)
需求 4 (bucket create)
```

### 技術風險緩解

1. **破壞性變更管理（需求 3）**
   - 明確版本升級策略（v0.x → v1.0）
   - 在 CHANGELOG 中清楚說明
   - 提供遷移指南
   - 確保 bucket 同步更新

2. **權限相關測試（需求 1）**
   - 需要在不同 OS 和權限級別下測試
   - Windows: 測試 Administrator 和一般用戶
   - Linux: 測試 root 和一般用戶
   - 考慮使用 CI/CD 自動化測試

3. **代碼重用驗證（需求 4）**
   - 確保 `GitHubProvider` 的輸出符合 manifest 格式
   - 驗證平台偵測邏輯的準確性
   - 測試 Gist 多腳本分組邏輯

4. **GitHub API 限制**
   - 實作 rate limiting 處理
   - 支援 GITHUB_TOKEN 環境變數
   - 提供清楚的錯誤訊息

### 實作建議

#### 需求 2 & 1: 可以並行
- 兩者互不影響，可由不同開發者同時進行

#### 需求 3 & 4: 必須串行
- **強烈建議先完成需求 3**
- 需求 4 的 Gist 提取邏輯直接產生新格式
- 避免後續重構

#### 測試策略
1. **單元測試**: 每個新功能都需要單元測試
2. **整合測試**: 
   - 需求 1: 需要在不同權限下測試
   - 需求 4: 需要實際呼叫 GitHub API（可用 mock）
3. **端到端測試**: 實際安裝流程驗證

### 結論

所有需求都是可行的，基於以下發現，整體複雜度降低：

1. ✅ **需求 3 不需向後兼容** → 節省 1.5 天
2. ✅ **需求 4 可重用大量現有代碼** → 節省 2 天  
3. ✅ **無需引入 async** → 降低技術風險

**建議執行順序:** 需求 2 → 需求 1 → 需求 3 → 需求 4

需求 2 可以立即著手，作為快速勝利（quick win）。需求 1 和 3 可以在下個版本中規劃。需求 4 應該在需求 3 穩定後再實施，確保直接產生正確的多平台格式。

整體預估從 **18.5 天降至 12.5 天**，約可在 **2.5-3 週內完成所有需求**。
