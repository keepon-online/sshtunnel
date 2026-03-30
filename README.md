# SSH Tunnel Manager

轻量桌面版 SSH 本地端口转发管理工具，目标平台为 Linux 和 Windows。

## 当前实现状态

- 已创建 Rust workspace
- 已实现 `sshtunnel-core` 核心库
- 已实现基础的 Tauri 外壳、托盘菜单和单窗口前端
- 已接入 JSON 配置存储
- 已接入系统凭据库适配：Windows Credential Manager / Linux Secret Service
- 当前连接流程优先调用系统 `ssh`
- 密码认证已接入 PTY 交互执行链路，会在检测到密码提示符后向系统 `ssh` 写入凭据库中的密码
- 原生 `ssh` 连接已采集实时 `stderr`，并写入运行日志面板
- Windows 发布版启动隧道时不再弹出 `cmd` 黑框
- 已暴露开机自启状态与切换开关
- 已支持托盘动态展示最近 3 条隧道，并直接执行连接/断开
- 已为前端 view-model 增加快照摘要、状态文案和连接动作映射测试
- 已为前端 tunnel 列表增加文案映射和选中态测试
- 已将主窗口重构为左侧隧道列表、右侧工作区和抽屉式编辑器
- 已强化工作区状态概览卡片，直接显示状态、转发、认证和错误摘要
- 已将日志面板升级为分组诊断视图，区分状态事件与 SSH 输出
- 已为后端运行时增加日志截断、退出状态和断开清理测试
- 已为后端 save/delete 配置流增加状态变更与持久化测试
- 已为 connect/disconnect/autostart 命令流增加 helper 级自动化测试
- 已为 tray `disconnect_all` 分支增加批量断开 helper 测试
- 已修复桌面前端的 Tauri bridge 容错，保存失败会在抽屉内显示，并支持私钥文件选择
- GitHub Actions 已支持 Ubuntu 校验、Linux `.deb` 构建和 Windows `.exe` 安装包构建

## 目录结构

- `crates/core`: 配置校验和 `ssh -L` 参数拼装
- `src-tauri`: Tauri 桌面壳和后端命令
- `web`: 无框架静态前端
- `docs/superpowers`: 设计文档和实现计划

## 本地开发

当前执行环境里 `cargo` 不在默认 PATH 中，建议先补上：

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

运行核心测试：

```bash
cargo test -p sshtunnel-core
```

尝试检查桌面壳是否可编译：

```bash
cargo check -p sshtunnel-app
```

## 配置与安全

- 普通隧道配置写入系统配置目录下的 `sshtunnel-manager/config.json`
- 密码不会写入配置文件
- 密码通过 `keyring` crate 写入系统凭据库
- 密钥认证直接生成 `ssh -L` 参数
- 密码认证通过交互式 PTY 会话驱动系统 `ssh`

## 已知限制

- Debug/dev 模式下如果仍保留控制台窗口，属于开发期行为；当前修复针对 Windows 发布版安装包

## 当前任务

- 视需要为 tray 刷新和真实平台集成补更深一层测试
