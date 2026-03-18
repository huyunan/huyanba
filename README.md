# 护眼吧 (huyanba)

桌面护眼小软件：防蓝光过滤 + 定时休息锁屏。

## 功能概览
- 过滤蓝光：强度 + 色调调节，预设模式（智能/办公/影视/游戏）
- 定时休息：默认每 30 分钟休息 1 分钟
- 全屏休息锁屏：多显示器覆盖、倒计时显示
- 托盘控制：显示/隐藏/立即休息/退出

## 界面截图
首页（护眼状态概览 + 下一次休息）
![首页](docs/images/首页.png)

功能（过滤蓝光强度/色调 + 休息节奏）
![功能](docs/images/功能.png)

锁屏（全屏覆盖 + 倒计时）
![锁屏](docs/images/锁屏.png)

故宫壁纸功能（在线获取 + 缓存轮播）
![故宫壁纸功能](docs/images/故宫壁纸功能.png)

## 版本
- 当前打包版本：`2.0.0`

## 安装包位置（本机）
```
D:\Ai\huyanba\huzamba\aseet
```

当前已整理的打包产物：
- `aseet\huyanba_2.0.0_x64-setup.exe`
- `aseet\huyanba_2.0.0_x64_en-US.msi`

## 下载
- [点击下载便携版 EXE（NSIS 安装包）](https://github.com/guoruya/huyanba/releases/download/v2.0.0/huyanba_2.0.0_x64-setup.exe)
- [点击下载 MSI 安装包](https://github.com/guoruya/huyanba/releases/download/v2.0.0/huyanba_2.0.0_x64_en-US.msi)
- [查看 v2.0.0 Release 页面](https://github.com/guoruya/huyanba/releases/tag/v2.0.0)

## 本地开发
```
cd D:\Ai\huyanba\huzamba
npm install
npm run tauri dev
```

## 打包（Windows 安装包）
```
cd D:\Ai\huyanba\huzamba
npm run tauri build
```

打包完成后，安装包会额外整理到项目根目录下的 `aseet` 目录：
```
aseet
```

## 说明
- 过滤蓝光通过系统 gamma 曲线实现
- 锁屏使用全屏覆盖窗口（非系统锁屏）

---

# Huyanba (English)

Desktop eye-care app: blue-light filter + scheduled break lockscreen.

## Features
- Blue-light filter with strength + tone presets
- Scheduled breaks (default 30 minutes work / 1 minute rest)
- Fullscreen rest lockscreen (multi-monitor)
- Tray controls (show/hide/rest/quit)

## Screenshots
Home (status + next break)
![Home](docs/images/首页.png)

Filter & Schedule
![Features](docs/images/功能.png)

Lockscreen
![Lockscreen](docs/images/锁屏.png)

Palace Museum wallpapers (online fetch + cached rotation)
![Palace Museum Wallpapers](docs/images/故宫壁纸功能.png)

## Version
- Current packaged version: `2.0.0`

## Installer (local path)
```
D:\Ai\huyanba\huzamba\aseet
```

Packaged artifacts currently available:
- `aseet\huyanba_2.0.0_x64-setup.exe`
- `aseet\huyanba_2.0.0_x64_en-US.msi`

## Download
- [Download NSIS installer (.exe)](https://github.com/guoruya/huyanba/releases/download/v2.0.0/huyanba_2.0.0_x64-setup.exe)
- [Download MSI installer (.msi)](https://github.com/guoruya/huyanba/releases/download/v2.0.0/huyanba_2.0.0_x64_en-US.msi)
- [Open the v2.0.0 release page](https://github.com/guoruya/huyanba/releases/tag/v2.0.0)

## Development
```
cd D:\Ai\huyanba\huzamba
npm install
npm run tauri dev
```

## Build (Windows)
```
cd D:\Ai\huyanba\huzamba
npm run tauri build
```

After the build, the packaged outputs are also copied to:
```
aseet
```
