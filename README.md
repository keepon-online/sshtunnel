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

- 当前托盘菜单是固定菜单，尚未动态列出最近三条隧道
- 自动启动插件已注册，但前端还未暴露开关
- 运行日志当前以生命周期日志为主，尚未采集 `ssh` 实时 stderr
- Windows 下的密码认证执行链路尚未做实机验证，当前实现基于跨平台 PTY 抽象
