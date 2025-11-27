我想仿造andoid obtanium/windows scoop用rust開發一個跨平台通用的github binary跨平台包管理器Wenget(Wen Package Manager)
https://github.com/superyngo/Wenget
命令wenget
基本思路是
**GitHub binary metadata aggregator + local portable package manager**


特點

* **無版本管理（永遠安裝最新版）**
* **metadata 完全本地、來源清單自己維護**
* **自動解析 GitHub Release → 產生本地 JSON manifest**
* **可同時支援 GitHub / GitLab / self-hosted**
* **多執行緒下載/分析**

這個方向非常合適、明確，不會爆炸複雜度。

包管理部分：
0.主要針對portable binary，不提供版本管理(只安裝最新版)
1.add命令：
  - 加入github專案網址
  - 用html分析github專案release binary網址、平台、short description、最新版本(stable)、容量等
  - 將符合規範的分析資料以json儲存作為來源庫(專案網址、short description、最新版號、各平台binary網址...等等)
  - 無binary等不符規範的就跳過
  - 可add多筆用空格隔開
  - 可用--source傳入文字檔或線上網址
  - 用多線程進行分析動作
  - 預設第一筆資料是包管理器本身的github資訊
2.來源庫基本上只在本地維護更新
3.list命令：秀出來源庫中支援本機平台安裝的app並顯示最新版號、容量和short desctiption
4.search命令：搜尋來源庫中支援本機平台安裝的app並顯示最新版號、容量和short desctiption 可接多個名稱篩選app(支援*)
4.info嗅出本機平台安裝的app並顯示最新版號和short desctiption及本機安裝的版號 可接多個名稱篩選app(支援*)
5.update命令更新來源庫(多線程)
6.install命令安裝app(使用內建壓縮庫自動處理)，並用另一個josn紀錄本機安裝資訊 可接多個名稱篩選app(支援*)
7.upgrade命令秀出可更新的app 接all或多個名稱篩選並更新app(支援*)
8.del刪除app 可接多個名稱篩選app(支援*)
9.所有app安裝在使用者home目錄內(包含app和json等，要處理安裝 layout 與 PATH 結構)
10.也可支援gitlab等不同開源專案源


包管理器本身部分
1.包管理器本身提供線上安裝腳本，處理PATH並先add包管理器本身的github repo
2.用upgrade self更新自己
3.github repo內可維護一份可add的url list文字檔，並開放issue提交url建議，當有新建議時觸發workflow判斷是否並執行加入list


問題難點
1.GitHub metadata解析
直接抓 HTML以避免API限制，或latest release用api其他用HTML


2. 🧭 binary 的選擇規則

不同 repo Release 通常會有：

| 文件檔案名稱                               | 平台      |
| ------------------------------------ | ------- |
| app-x86_64-unknown-linux-musl.tar.gz | Linux   |
| app-windows-x86_64.zip               | Windows |
| app-macos-aarch64.tar.gz             | Mac     |
| source.tar.gz                        | x       |

* 如何識別哪個 `.zip` 是 平台
* Linux glibc vs musl 是否區分？優先使用musl?
* Mac Intel vs ARM 是否自動 try？


3 🗂️ 安裝 layout 與 PATH 結構

「裝在使用者 home 內」，建議如下架構：

```
~/.wenget/
   sources.json
   installed.json
   apps/
      app1/
         bin/
         data/
      app2/
```

固定 binary 入口點：

```
~/.wenget/apps/<name>/bin/<name>
```

然後 PATH 只加：

```
$HOME/.wenget/bin
```

再在此做 symlink：

```
~/.wenget/bin/app1 → ../apps/app1/bin/app1
```

Windows 則用 .cmd shim。

---

4. ⚙️ Upgrade self 是否可能？

**可以，但要特別處理 Windows 鎖檔問題**

你可用：

### 方法 A：新檔案 → 舊檔案替換

Linux/macOS OK。
Windows 要用：

```
rename running exe → .old
download new exe → original name
exit self
```

或寫 bootstrap mini-updater。

---

4. 📦 multi-source merge（你說的 GitHub/GitLab）

建議支援：

```
add https://github.com/foo/bar
add https://gitlab.com/user/project
add https://gitea.io/foo
```

但你需：

* 用 URL 判斷平台
* 寫多個 parser（或 trait）

---

5. ⚠️ 錯誤處理：Release 可能沒有 binary

你要處理：

* release 只有原始碼 → ignore
* binary 檔名格式不標準 → ignore
* nightly/incomplete release → 只抓最新 stable tag



6. manifest 完整結構

建議 JSON 至少包含：

```json
{
  "name": "wedi",
  "description": "A minimalist text editor",
  "repo": "https://github.com/superyngo/wedi",
  "latest": "0.1.12",
  "platforms": {
    "windows-x86_64": {
      "url": "https://...",
      "size": 1200000,
    },
    "linux-x86_64": {...}
  }
}


# 先做 MVP（最小可用版本）

**第一版就實作以下即可：**

* `add`（解析 GitHub latest 釋出 binary）
* `list`
* `install`
* `upgrade self`
* 本地 JSON manifest
* PATH 處理
