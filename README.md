# wclean

🧹 一个用于清理 Wwise 工程中 `Originals` 目录下未被引用的 `.wav` 音频文件的小工具。

## ✨ 功能简介

- 自动扫描 `.wwu` 文件中引用的音频路径；
- 比对 `Originals/SFX` 和 `Originals/Voices` 下的 `.wav` 文件；
- 找出未被使用的 `.wav`；
- 支持将未引用列表导出为文本；
- 支持直接删除未引用的 `.wav` 文件。

## 📦 下载

[下载地址](https://github.com/xmimu/wclean/releases)


## 📌 示例用法
### 参数说明

| 参数                      | 说明                                         |
| ----------------------- | ------------------------------------------ |
| `<wwise_project_path>`  | 必填。Wwise 工程目录（包含 `.wproj` 文件）              |
| `-o`, `--output <file>` | 可选。将未引用的 `.wav` 路径导出到指定文件                  |
| `-d`, `--delete <file>` | 可选。删除指定文件中列出的 `.wav`，或在未指定 `-o` 时直接删除未引用文件 |

### 1️⃣ **默认模式**：扫描并展示数量

```bash
wclean D:\WwiseProject
```

* 扫描 `.wwu` 文件引用的 `.wav`，并展示已引用与未引用的数量
* **不导出、不删除**

---

### 2️⃣ **导出未引用列表到文件**

```bash
wclean D:\WwiseProject -o unused.txt
```

* 自动扫描 `.wwu` 引用
* 导出未被引用的 `.wav` 文件路径列表到 `unused.txt`
* **不删除**

---

### 3️⃣ **扫描并删除未引用的 `.wav` 文件**

```bash
wclean D:\WwiseProject -d
```

* 自动扫描 `.wwu` 引用
* 删除未被引用的 `.wav` 文件
* **不导出**

---

### 4️⃣ **扫描、导出并删除未引用的 `.wav` 文件**

```bash
wclean D:\WwiseProject -o unused.txt -d
```

* 自动扫描 `.wwu` 引用
* 导出未引用列表
* 删除未被引用的 `.wav`

---

### 5️⃣ **从文件中删除列表指定的 `.wav`**

```bash
wclean D:\WwiseProject -d unused.txt
```

* 不重新扫描
* 从指定列表文件中读取 `.wav` 路径并删除
* 通常用于上一次导出的 `unused.txt`

---

#### 先导出，后手动审核再删除：

```bash
# 第一步：导出未引用列表
wclean E:/WWise/MyProject -o unused.txt

# 第二步：编辑 unused.txt 后再删除
wclean E:/WWise/MyProject -d unused.txt
```

## 🛡️ 安全提示

* 删除操作不可恢复，执行前建议先加 `-o` 输出列表进行人工确认；
* 删除前请确保已备份相关工程文件。或者已添加到版本控制工具。

## 🧩 技术栈

* Rust + [clap](https://crates.io/crates/clap) 命令行解析
* [glob](https://crates.io/crates/glob) 文件匹配
* [rayon](https://crates.io/crates/rayon) 并行处理
* [roxmltree](https://crates.io/crates/roxmltree) XML 解析

## 📄 License

MIT License

---