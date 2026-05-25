import { ref, onMounted, onUnmounted } from "vue";

export function getOffsetMinutes(timezone: string, date: Date): number {
  const tzDate = new Date(date.toLocaleString("en-US", { timeZone: timezone }));
  const utcDate = new Date(date.toLocaleString("en-US", { timeZone: "UTC" }));
  return (tzDate.getTime() - utcDate.getTime()) / 60000;
}

export function getDateInTimezone(timezone: string): string {
  return new Intl.DateTimeFormat("zh-CN", {
    timeZone: timezone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(new Date());
}

export function convertTimeBetweenZones(
  timeStr: string,
  fromTz: string,
  toTz: string
): { time: string; date: string; dayDiff: number } {
  try {
    const [hours, minutes] = timeStr.split(":").map(Number);

    const now = new Date();
    const dateParts = new Intl.DateTimeFormat("en-CA", {
      timeZone: fromTz,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    }).format(now);

    const localDateStr = `${dateParts}T${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:00`;
    const localDate = new Date(localDateStr);
    const localOffset = -localDate.getTimezoneOffset();
    const fromOffset = getOffsetMinutes(fromTz, localDate);
    const utcMs = localDate.getTime() + (localOffset - fromOffset) * 60000;
    const targetDate = new Date(utcMs);

    const time = new Intl.DateTimeFormat("zh-CN", {
      timeZone: toTz,
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    }).format(targetDate);

    const date = new Intl.DateTimeFormat("zh-CN", {
      timeZone: toTz,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    }).format(targetDate);

    const srcDate = new Intl.DateTimeFormat("zh-CN", {
      timeZone: fromTz,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    }).format(now);

    const dayDiff = Math.round(
      (new Date(date).getTime() - new Date(srcDate).getTime()) / 86400000
    );

    return { time, date, dayDiff };
  } catch {
    return { time: "--:--", date: "----/--/--", dayDiff: 0 };
  }
}

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
