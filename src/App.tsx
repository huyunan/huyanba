import {
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function pad2(value: number) {
  return value.toString().padStart(2, "0");
}

function formatDuration(totalSeconds: number) {
  const clamped = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(clamped / 3600);
  const minutes = Math.floor((clamped % 3600) / 60);
  const seconds = clamped % 60;
  return `${pad2(hours)}:${pad2(minutes)}:${pad2(seconds)}`;
}

function formatDuration2(totalSeconds: number) {
  const clamped = Math.max(0, Math.floor(totalSeconds));
  const minutes = Math.floor((clamped % 3600) / 60);
  const seconds = clamped % 60;
  return `${pad2(minutes)}:${pad2(seconds)}`;
}

function App() {
  const isLockWindow =
    new URLSearchParams(window.location.search).get("lockscreen") === "1";
  const [now, setNow] = useState(new Date());
  // 过滤蓝光开关
  const [filterEnabled, setFilterEnabled] = useState(true);
  // 开机自启
  const [startupEnabled, setStartupEnabled] = useState(false);
  // 强度
  const [filterStrength, setFilterStrength] = useState(30);
  // 色调
  const [colorTemp, setColorTemp] = useState(4700);
  // 休息节奏开关
  const [restEnabled, setRestEnabled] = useState(true);
  // 休息间隔
  const [restMinutes, setRestMinutes] = useState(60);
  // 休息时间
  const [restDuration, setRestDuration] = useState(3);
  // 显示锁屏弹框
  const [showLockScreen, setShowLockScreen] = useState(false);
  const [activePreset, setActivePreset] = useState("智能");
  const [nextRestAt, setNextRestAt] = useState<Date | null>(null);
  // 休息结束时间（未弹出锁屏窗口前）
  const [restEndAt, setRestEndAt] = useState<Date | null>(null);
  // 锁屏数据
  const [lockPayload, setLockPayload] = useState({
    timeText: "--:--",
    dateText: "",
    restCountdown: "00:00",
  });
  // 休息结束时间（已弹出锁屏窗口）
  const [lockEndAtMs, setLockEndAtMs] = useState<number | null>(null);

  const presets = useMemo(
    () => ({
      智能: {
        day: { temp: 4700, strength: 30 },
        night: { temp: 3400, strength: 30 },
      },
      办公: {
        day: { temp: 5200, strength: 50 },
        night: { temp: 4700, strength: 60 },
      },
      影视: {
        day: { temp: 5600, strength: 45 },
        night: { temp: 5200, strength: 55 },
      },
      游戏: {
        day: { temp: 6000, strength: 35 },
        night: { temp: 5600, strength: 45 },
      },
    }),
    [],
  );

  const isDaytime = now.getHours() >= 6 && now.getHours() < 18;
  const resolvePreset = useCallback(
    (preset: keyof typeof presets) => {
      const config = presets[preset];
      if (!config) {
        return { temp: 4700, strength: 30 };
      }
      if (preset === "智能") {
        return isDaytime ? config.day : config.night;
      }
      return config.day;
    },
    [isDaytime, presets],
  );

  useEffect(() => {
    if (activePreset !== "智能") return;
    const next = resolvePreset("智能");
    setFilterStrength(next.strength);
    setColorTemp(next.temp);
  }, [activePreset, resolvePreset]);

  const restDuraAt = () => {
    return new Date(Date.now() + restDuration * 60 * 1000);
  };
  
  const restMsAt = () => {
    return new Date(Date.now() + restMinutes * 60 * 1000);
  };
  
  const handleStartRest = useCallback(() => {
    const endAt = restDuraAt();
    setRestEndAt(endAt);
    setShowLockScreen(true);
    showLockWindows();
  }, [restDuration]);
  
  const showLockWindows = () => {
    const endAt = restEndAt ?? restDuraAt();
    invoke("show_lock_windows", {
      endAtMs: endAt.getTime(),
    }).catch((error) => console.error("锁屏窗口创建失败", error));
  }
  
  const hideLockWindows = () => {
    invoke("log_app", { message: "前端请求关闭锁屏" }).catch(() => undefined);
    invoke("hide_lock_windows").catch((error) =>
      console.error("锁屏窗口关闭失败", error),
    );
  }  
  
  useEffect(() => {
    if (!showLockScreen) return;
    setRestEndAt(restDuraAt());
  }, [restDuration, showLockScreen]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const appWebview = getCurrentWebviewWindow();
    appWebview
      .listen<string>("lockscreen-action", (event) => {
        if (event.payload === "exit") {
          handleExitRest();
        }
      })
      .then((fn) => {
        unlisten = fn;
      })
      .catch((error) => console.error("监听锁屏动作失败", error));

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);
  
  const handleExitRest = useCallback(() => {
    invoke("log_app", { message: "前端退出休息: start" }).catch(() => undefined);
    setShowLockScreen(false);
    hideLockWindows();
    setRestEndAt(null);
    if (restEnabled) {
      setNextRestAt(restMsAt());
    } else {
      setNextRestAt(null);
    }
    if (!isLockWindow) {
      invoke("set_gamma", {
        filterEnabled,
        strength: filterStrength,
        colorTemp,
      }).catch(() => undefined);
    }
    invoke("log_app", { message: "前端退出休息: end" }).catch(() => undefined);
  }, [
    restEnabled,
    restMinutes,
    isLockWindow,
    filterEnabled,
    filterStrength,
    colorTemp,
  ]);

  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  useEffect(() => {
    if (isLockWindow) return;
    const reset = () => {
      invoke("reset_gamma").catch(() => undefined);
    };
    window.addEventListener("beforeunload", reset);
    return () => {
      window.removeEventListener("beforeunload", reset);
      reset();
    };
  }, [isLockWindow]);

  useEffect(() => {
    if (isLockWindow) return;
    let active = true;
    const handle = setTimeout(() => {
      invoke("set_gamma", {
        filterEnabled,
        strength: filterStrength,
        colorTemp,
      }).catch((error) => {
        if (active) {
          console.error("过滤蓝光设置失败", error);
        }
      });
    }, 80);
    return () => {
      active = false;
      clearTimeout(handle);
    };
  }, [isLockWindow, filterEnabled, filterStrength, colorTemp]);

  useEffect(() => {
    if (!isLockWindow) return;
    const params = new URLSearchParams(window.location.search);
    const end = Number(params.get("end") || 0);
    setLockEndAtMs(end > 0 ? end : null);
  }, [isLockWindow]);
  
  useEffect(() => {
    if (!isLockWindow) return;
    const timer = setInterval(() => {
      const nowValue = new Date();
      const timeValue = nowValue.toLocaleTimeString("zh-CN", {
        hour: "2-digit",
        minute: "2-digit",
      });
      const dateValue = nowValue.toLocaleDateString("zh-CN", {
        month: "long",
        day: "numeric",
        weekday: "short",
      });

      let countdown = "00:00";
      if (lockEndAtMs) {
        countdown = formatDuration2((lockEndAtMs - nowValue.getTime()) / 1000);
      }

      setLockPayload((prev) => ({
        ...prev,
        timeText: timeValue,
        dateText: dateValue,
        restCountdown: countdown,
      }));
    }, 500);
    return () => clearInterval(timer);
  }, [isLockWindow, lockEndAtMs]);
  
  useEffect(() => {
    if (showLockScreen) return;
    if (!restEnabled) {
      setNextRestAt(null);
      return;
    }
    setNextRestAt(restMsAt());
  }, [showLockScreen, restEnabled, restMinutes]);

  useEffect(() => {
    if (!restEnabled || showLockScreen) return;
    if (!nextRestAt) return;
    if (now.getTime() >= nextRestAt.getTime()) {
      const endAt = restDuraAt();
      setRestEndAt(endAt);
      setShowLockScreen(true);
      showLockWindows();
    }
  }, [now, restEnabled, nextRestAt, restDuration, showLockScreen]);

  useEffect(() => {
    const fetchData = async () => {
      try {
        if (startupEnabled) {
          await enable();
          console.log(`registered for autostart? ${await isEnabled()}`);
        } else {
          disable();
        }
      } catch (error) {
        console.error('开机启动配置失败:', error);
      }
    };

    fetchData();
  }, [startupEnabled]);
  
  useEffect(() => {
    if (!showLockScreen || !restEndAt) return;
    if (now.getTime() >= restEndAt.getTime()) {
      handleExitRest();
    }
  }, [handleExitRest, now, restEndAt, showLockScreen]);

  useEffect(() => {
    if (showLockScreen) return;
    if (!restEnabled || !nextRestAt) return;
    if (now.getTime() < nextRestAt.getTime()) return;
    setNextRestAt(restMsAt());
  }, [now, showLockScreen, restEnabled, nextRestAt, restMinutes]);

  const nextRestCountdown = restEnabled && nextRestAt
    ? formatDuration((nextRestAt.getTime() - now.getTime()) / 1000)
    : "已暂停";

  const timeText = now.toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
  });
  const dateText = now.toLocaleDateString("zh-CN", {
    month: "long",
    day: "numeric",
    weekday: "short",
  });

  return (
    <div className="app">
      {!isLockWindow && (
        <>
          <div className="ambient ambient--one" />
          <div className="ambient ambient--two" />
          <div className="ambient ambient--grid" />

          <header className="topbar">
            <div className="brand">
              <img className="brand__icon" src="/huyanba-mark.svg" alt="" />
              <div>
                <p className="brand__name">护眼吧</p>
                <p className="brand__tag">清醒护眼 · 专注节奏</p>
              </div>
            </div>
            <div className="topbar__right">
              <div className="time-pill">
                <span>{timeText}</span>
                <span className="time-pill__date">{dateText}</span>
              </div>
            </div>
          </header>

          <>
            <section className="hero">
              <div className="hero__text">
                <p className="hero__kicker">今日护眼状态</p>
                <h1>保持专注，但别忘了松一口气。</h1>
                <div className="hero__stats">
                  <div>
                    <p className="stat__label">今日休息次数</p>
                    <p className="stat__value">4 次</p>
                  </div>
                  <div>
                    <p className="stat__label">下一次休息</p>
                    <p className="stat__value">{nextRestCountdown}</p>
                  </div>
                </div>
              </div>
            </section>

            <section className="main-grid">
            {filterEnabled && (
              <div className="card">
                <div className="card__header">
                  <div>
                    <p className="card__eyebrow">护眼滤镜</p>
                    <h2>过滤蓝光</h2>
                  </div>
                  <label className="toggle">
                    <input
                      type="checkbox"
                      checked={filterEnabled}
                      onChange={() => setFilterEnabled((prev) => !prev)}
                    />
                    <span className="toggle__track" />
                  </label>
                </div>

                <div className="slider-group">
                  <div className="slider-row">
                    <span>强度</span>
                    <span>{filterStrength}%</span>
                  </div>
                  <input
                    type="range"
                    min={0}
                    max={100}
                    value={filterStrength}
                    onChange={(event) =>
                      setFilterStrength(Number(event.target.value))
                    }
                  />
                </div>

                <div className="chips">
                  {(Object.keys(presets) as Array<keyof typeof presets>).map(
                    (preset) => (
                      <button
                        key={preset}
                        type="button"
                        className={`chip ${
                          activePreset === preset ? "chip--active" : ""
                        }`}
                        onClick={() => {
                          setActivePreset(preset);
                          const next = resolvePreset(preset);
                          setFilterStrength(next.strength);
                          setColorTemp(next.temp);
                          setFilterEnabled(true);
                        }}
                      >
                        {preset}
                      </button>
                    ),
                  )}
                </div>

                <div className="slider-group">
                  <div className="slider-row">
                    <span>色调</span>
                    <span>{colorTemp}K</span>
                  </div>
                  <input
                    type="range"
                    min={2000}
                    max={6500}
                    step={100}
                    value={colorTemp}
                    onChange={(event) => setColorTemp(Number(event.target.value))}
                  />
                </div>
              </div>
            )}
              <div className="card">
                <div className="card__header">
                  <div>
                    <p className="card__eyebrow">定时休息</p>
                    <h2>休息节奏</h2>
                  </div>
                  <label className="toggle">
                    <input
                      type="checkbox"
                      checked={restEnabled}
                      onChange={() => setRestEnabled((prev) => !prev)}
                    />
                    <span className="toggle__track" />
                  </label>
                </div>

                <div className="pill-row">
                  <div className="pill">
                    <p className="pill__label">每隔</p>
                    <input
                      className="pill__input"
                      type="number"
                      min={15}
                      max={120}
                      value={restMinutes}
                      onChange={(event) =>
                        setRestMinutes(Number(event.target.value))
                      }
                    />
                    <span>分钟</span>
                  </div>
                  <div className="pill">
                    <p className="pill__label">休息</p>
                    <input
                      className="pill__input"
                      type="number"
                      min={1}
                      max={30}
                      value={restDuration}
                      onChange={(event) =>
                        setRestDuration(Number(event.target.value))
                      }
                    />
                    <span>分钟</span>
                  </div>
                </div>

                <div className="rest-countdown">
                  <p>距离下次休息还有</p>
                  <h3>{nextRestCountdown}</h3>
                </div>

                <button
                  className="btn btn--ghost"
                  type="button"
                  onClick={handleStartRest}
                >
                  立即进入休息
                </button>
              </div>

              <div className="card">
                <div className="card__header">
                  <div>
                    <p className="card__eyebrow">系统设置</p>
                  </div>
                </div>

                <div className="settings">
                  <label className="setting-row">
                    <span>开启护眼</span>
                    <label className="toggle">
                      <input
                        type="checkbox"
                        checked={filterEnabled}
                        onChange={() => setFilterEnabled((prev) => !prev)}
                      />
                      <span className="toggle__track" />
                    </label>
                  </label>

                  <label className="setting-row">
                    <span>开机自启</span>
                    <label className="toggle">
                      <input
                        type="checkbox"
                        checked={startupEnabled}
                        onChange={() => setStartupEnabled((prev) => !prev)}
                      />
                      <span className="toggle__track" />
                    </label>
                  </label>
                </div>
              </div>
            </section>
          </>
        </>
      )}
      
      {isLockWindow && (
        <div
          className="lockscreen"
        >
          <div className="lockscreen__scrim" />
          <div className="lockscreen__content">
            <div className="lockscreen__top">
              <div>
                <p className="lockscreen__time">{lockPayload.timeText}</p>
                <p className="lockscreen__date">{lockPayload.dateText}</p>
              </div>
              <div />
            </div>
            <div className="lockscreen__center">
              <p>休息一下，放松眼睛</p>
              <div className="lockscreen__timer">
                <div className="lockscreen__timer-value">
                  {lockPayload.restCountdown.replaceAll(":", " : ")}
                </div>
              </div>
              <button
                className="view-tab"
                type="button"
                onClick={() => {
                  invoke("lockscreen_action", {action: "exit"}).catch((error) =>
                    console.error("锁屏窗口关闭失败", error),
                  )}
                }
              >
                跳过休息
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
