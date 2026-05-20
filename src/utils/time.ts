import { ref, onMounted, onUnmounted } from "vue";

// 获取指定时区的当前时间
export function getTimeInTimezone(timezone: string): { time: string; date: string; offset: string } {
  try {
    const date = new Date();
    const options: Intl.DateTimeFormatOptions = {
      timeZone: timezone,
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false,
    };
    const dateOptions: Intl.DateTimeFormatOptions = {
      timeZone: timezone,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    };

    const time = date.toLocaleTimeString("zh-CN", options);
    const dateStr = date.toLocaleDateString("zh-CN", dateOptions);

    // 计算UTC偏移
    const tzDate = new Date(date.toLocaleString("en-US", { timeZone: timezone }));
    const utcDate = new Date(date.toLocaleString("en-US", { timeZone: "UTC" }));
    const diff = (tzDate.getTime() - utcDate.getTime()) / (1000 * 60 * 60);
    const sign = diff >= 0 ? "+" : "-";
    const hours = Math.floor(Math.abs(diff));
    const minutes = Math.round((Math.abs(diff) - hours) * 60);
    const offset = `UTC${sign}${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}`;

    return { time, date: dateStr, offset };
  } catch {
    return { time: "--:--:--", date: "----/--/--", offset: "UTC+??:??" };
  }
}

// composable: 在组件中使用实时时间
export function useRealtimeTime(timezone: () => string) {
  const time = ref("");
  const date = ref("");
  const offset = ref("");

  function update() {
    const tz = timezone();
    if (tz) {
      const result = getTimeInTimezone(tz);
      time.value = result.time;
      date.value = result.date;
      offset.value = result.offset;
    }
  }

  let timer: ReturnType<typeof setInterval> | null = null;

  onMounted(() => {
    update();
    timer = setInterval(update, 1000);
  });

  onUnmounted(() => {
    if (timer) {
      clearInterval(timer);
    }
  });

  return { time, date, offset };
}
