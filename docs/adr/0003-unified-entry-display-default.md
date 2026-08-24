# ADR-0003：统一入口、角色选择与默认电视显示模式

- 状态：已接受（设计阶段）
- 日期：2026-08-24

## 背景

Gateway 同时承担两种 Web 使用方式：

1. Control：在手机或 Windows 上选择内容、登录网站、控制播放和执行 handoff；
2. Display：在电视、显示器或其他浏览器上长期等待并播放 Gateway 分配的媒体。

如果根 URL 永远固定为 Control，电视端需要记忆额外路径；如果根 URL 永远固定为 Display，手机和 Windows 首次访问又不够直观。仅依赖屏幕分辨率自动判断角色也不可靠：4K 屏幕可能是桌面显示器，电视浏览器也可能报告缩放后的较小 viewport，而且分辨率绝不能成为控制权限依据。

项目需要同时满足：

- 电视只记住一个简单地址；
- 手机/Windows 可以快速进入控制模式；
- 无遥控器操作时电视能自动进入待投屏状态；
- 自动化测试可以绕过倒计时，获得确定性页面角色；
- 页面角色、显示布局和安全授权互不混淆。

## 决策

### 1. `/` 是统一智能入口

访问 Gateway 根 URL 时，显示极简角色选择：

```text
Gateway
├── 显示模式
└── 控制模式
```

MVP 默认倒计时为 5 秒。如果用户在倒计时内没有操作，入口自动进入：

```text
PageRole = display
DisplayProfile = tv
```

超时时间属于 UI/配置层，可在后续调整；它不是 Playback Coordinator 的领域状态。

### 2. 保留两个确定性深链接

```text
/display
/control
```

- `/display`：直接进入专用 Web Display，不经过根入口倒计时。
- `/control`：直接进入 Control PWA，不经过根入口倒计时。

这两个入口用于电视书签、kiosk、二维码、开发调试和自动化测试。

### 3. PageRole 与 DisplayProfile 分离

页面角色回答“这个页面现在做什么”：

```text
PageRole = control | display
```

显示 profile 回答“Display 应该怎样呈现”：

```text
DisplayProfile = tv | desktop | mobile | auto
```

路径、用户选择或已保存偏好决定 PageRole；viewport、输入方式、媒体能力和用户显式选择可以影响 DisplayProfile。

屏幕尺寸、User-Agent 或 profile 不得被用来推断认证身份、控制权限或设备所有权。

### 4. 专用 Display 默认面向电视/大屏

`/display` 是常驻投屏页面，首版默认采用 TV-oriented immersive layout：

- 占满 viewport；
- 黑色背景；
- 视频 `object-fit: contain`；
- 大字号、自适应字幕；
- 遥控器/方向键友好焦点；
- 播放后自动隐藏控制层；
- 空闲时显示 display label、在线状态和 `/control` 地址/二维码；
- 支持时申请 Screen Wake Lock。

浏览器真正 Fullscreen 可能要求用户手势，因此 `requestFullscreen()` 成功不得成为注册或播放前置条件。页面首先保证 viewport 级沉浸播放，首次点击、触摸或遥控器按键可用于申请真正 Fullscreen。

### 5. Control 不自动成为 Display

访问 `/control` 只创建控制上下文，不自动注册新的 `DisplayInstance`。

如果用户希望当前手机/电脑本机播放，必须执行“在本机播放”或等价显式操作，此时才通过 `WebDisplayAdapter` 注册当前浏览器显示端。

这样可以避免每个临时控制页都污染 Display 列表。

### 6. 可以记住 preferred_role，但它只是 UX 偏好

浏览器可以通过 localStorage 或等价机制保存：

```text
preferred_role = control | display
```

实现可以利用该偏好减少重复选择，但必须提供切换/清除方式。该值属于不可信客户端状态，不能替代服务端认证、Display 配对或媒体授权。

### 7. 路由角色不是安全边界

直接访问 `/control` 不获得控制权限；直接访问 `/display` 也不自动获得任意媒体任务。

所有敏感操作仍由 Gateway API 独立校验：

- 用户/管理会话；
- Display 注册或配对；
- PlaybackSession 归属；
- handoff 权限；
- 临时媒体 Token。

根入口自动跳转只能指向 Gateway 自身固定路径，不接受任意外部 redirect。

### 8. 测试使用确定性入口

绝大多数 Playwright/E2E 不通过 `/` 进入角色，而直接使用：

```text
Browser A → /control
Browser B → /display?profile=tv
Browser C → /display?profile=desktop
```

`/` 单独保留以下入口路由测试：

- 点击 Control 后取消/覆盖默认 Display 跳转；
- 点击 Display 立即进入显示模式；
- 无操作 5 秒后进入 TV Display；
- 自动跳转失败时仍可手动选择。

这使 Web Display 既是正式产品能力，也是 Display Adapter 的参考实现和自动化测试基准。

## 结果

优点：

- 电视只需要记住 Gateway 根地址。
- 无操作即可进入待投屏状态，降低遥控器操作成本。
- 手机/Windows 仍能从同一地址明确进入 Control。
- `/display` 与 `/control` 给自动化和书签提供稳定入口。
- PageRole、DisplayProfile、DisplayInstance 与安全身份边界清晰。
- Control 页面不会制造大量无意义 Web Display。
- TV Web Display 可以成为无需 Jellyfin 的一等投屏路径。

代价：

- 根入口增加一个小型路由状态和倒计时行为。
- 需要处理 preferred_role、显式切换和自动跳转失败等 UX 状态。
- Fullscreen、Wake Lock 和电视浏览器能力存在平台差异，需要 capability 检测和真实设备补充测试。
- `/`、`/display`、`/control` 三条路径都需要纳入兼容性测试。

## 被拒绝方案

### `/` 永远是 Control

电视需要记住 `/display`，不符合“打开一个简单 URL 就等待投屏”的目标。

### `/` 永远是 Display

手机和 Windows 首次访问时不够直观，且控制入口可发现性下降。

### 仅通过分辨率自动判断角色

设备类型推断不可靠，并且会把 UI 自适应和权限/角色混在一起。

### Control 页面总是自动注册为 Display

会让每个手机/桌面控制会话都出现在显示端列表，增加误投、状态竞争和测试噪声。

### 所有测试都从 `/` 等待倒计时

会增加测试时间和不稳定性。确定性深链接更适合作为自动化主入口。
