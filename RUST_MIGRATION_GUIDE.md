# Rust 环境迁移指南

## 当前状态

Rust 环境目前安装在 C 盘：
- Rustup 目录：`C:\Users\odayo\.rustup`
- Cargo 目录：`C:\Users\odayo\.cargo`

## 迁移步骤

### 方法一：重新安装（推荐）

1. 卸载现有的 Rust
   ```powershell
   rustup self uninstall
   ```

2. 创建 D 盘目录
   ```powershell
   mkdir D:\Rust\.cargo
   mkdir D:\Rust\.rustup
   ```

3. 设置环境变量（临时，或者永久添加到系统环境变量）
   ```powershell
   $env:CARGO_HOME = "D:\Rust\.cargo"
   $env:RUSTUP_HOME = "D:\Rust\.rustup"
   ```

4. 重新安装 Rust
   访问 https://rustup.rs 下载并按照安装程序

5. 验证安装
   ```powershell
   rustc --version
   cargo --version
   rustup show home
   ```

### 方法二：移动现有目录

1. 停止所有正在使用的 Rust 程序

2. 移动目录
   ```powershell
   Move-Item C:\Users\odayo\.cargo D:\Rust\.cargo
   Move-Item C:\Users\odayo\.rustup D:\Rust\.rustup
   ```

3. 设置环境变量
   - 打开 "系统属性" -> "环境变量"
   - 添加/修改以下变量：
     - `CARGO_HOME` = `D:\Rust\.cargo`
     - `RUSTUP_HOME` = `D:\Rust\.rustup`
   - 更新 `PATH` 变量，把 `D:\Rust\.cargo\bin` 添加进去

4. 验证
   ```powershell
   rustc --version
   cargo --version
   ```

## 验证迁移成功

运行以下命令确认目录已迁移到 D 盘：

```powershell
rustup show home
echo $env:CARGO_HOME
```

## 注意事项

1. 需要重新编译项目
2. 已下载的依赖需要重新下载
3. 如果使用 WSL，需要单独设置 WSL 中的环境变量
