# 护眼吧 (huyanba)

桌面护眼小软件：防蓝光过滤 + 定时休息锁屏。

## 功能概览
- 过滤蓝光：强度 + 色调调节，预设模式（智能/办公/影视/游戏）
- 定时休息：默认每 30 分钟休息 1 分钟
- 全屏休息锁屏：多显示器覆盖、倒计时显示
- 托盘控制：显示/隐藏/立即休息/退出

## 界面截图
- 首页总览（当前护眼状态 + 下一次休息）
![首页总览](docs/images/dashboard-overview.png)

## 版本
- 当前打包版本：`2.1.0`

## 安装包位置（本机）
```
D:\Ai\huyanba\huzamba\aseet
```

当前已整理的打包产物：
- `aseet\huyanba_2.1.0_x64-setup.exe`
- `aseet\huyanba_2.1.0_x64_en-US.msi`

## 下载
- [点击下载便携版 EXE（NSIS 安装包）](https://github.com/guoruya/huyanba/releases/download/v2.1.0/huyanba_2.1.0_x64-setup.exe)
- [点击下载 MSI 安装包](https://github.com/guoruya/huyanba/releases/download/v2.1.0/huyanba_2.1.0_x64_en-US.msi)
- [查看 v2.1.0 Release 页面](https://github.com/guoruya/huyanba/releases/tag/v2.1.0)

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

After the build, the packaged outputs are also copied to:
```
aseet
```
