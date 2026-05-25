<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  convertTimeBetweenZones,
  getTimeInTimezone,
} from "../utils/time";
import type { CityResult, SearchResponse } from "../types";

const props = defineProps<{
  cityA: CityResult;
}>();

const emit = defineEmits<{
  close: [];
}>();

const cityA = ref<CityResult>({ ...props.cityA });
const cityB = ref<CityResult | null>(null);
const timeA = ref("");
const timeB = ref("");
const dateA = ref("");
const dateB = ref("");
const dayDiffB = ref(0);
const lastEdited = ref<"A" | "B">("A");

// City B search state
const searchQuery = ref("");
const searchOpen = ref(false);
const searchResults = ref<CityResult[]>([]);
const searchInputRef = ref<HTMLInputElement | null>(null);

const pickerA = ref(false);
const pickerB = ref(false);
const pickerAHours = ref(0);
const pickerAMinutes = ref(0);
const pickerBHours = ref(0);
const pickerBMinutes = ref(0);

function parseTime(t: string): [number, number] {
  const [h, m] = t.split(":").map(Number);
  return [isNaN(h) ? 0 : h, isNaN(m) ? 0 : m];
}

function openPickerA() {
  pickerB.value = false;
  [pickerAHours.value, pickerAMinutes.value] = parseTime(timeA.value);
  pickerA.value = true;
}

function openPickerB() {
  pickerA.value = false;
  [pickerBHours.value, pickerBMinutes.value] = parseTime(timeB.value);
  pickerB.value = true;
}

function selectTimeA() {
  timeA.value = `${String(pickerAHours.value).padStart(2, "0")}:${String(pickerAMinutes.value).padStart(2, "0")}`;
  pickerA.value = false;
  lastEdited.value = "A";
  recalc();
}

function selectTimeB() {
  timeB.value = `${String(pickerBHours.value).padStart(2, "0")}:${String(pickerBMinutes.value).padStart(2, "0")}`;
  pickerB.value = false;
  lastEdited.value = "B";
  recalc();
}

const hourList = Array.from({ length: 24 }, (_, i) => String(i).padStart(2, "0"));
const minList = Array.from({ length: 60 }, (_, i) => String(i).padStart(2, "0"));

function calcDayDiff(d1: string, d2: string): number {
  return Math.round(
    (new Date(d1).getTime() - new Date(d2).getTime()) / 86400000
  );
}

function recalc() {
  if (!cityB.value) return;

  const fromTz = lastEdited.value === "A" ? cityA.value.timezone : cityB.value!.timezone;
  const toTz = lastEdited.value === "A" ? cityB.value!.timezone : cityA.value.timezone;
  const timeStr = lastEdited.value === "A" ? timeA.value : timeB.value;

  if (!timeStr || timeStr.length < 4) return;

  const result = convertTimeBetweenZones(timeStr, fromTz, toTz);

  if (lastEdited.value === "A") {
    timeB.value = result.time.substring(0, 5);
    dateB.value = result.date;
    dayDiffB.value = calcDayDiff(result.date, dateA.value);
  } else {
    timeA.value = result.time.substring(0, 5);
    dateA.value = result.date;
  }
}

function swap() {
  if (!cityB.value) return;
  const tmpCity = { ...cityA.value };
  cityA.value = { ...cityB.value };
  cityB.value = tmpCity;
  const tmpTime = timeA.value;
  timeA.value = timeB.value;
  timeB.value = tmpTime;
  const tmpDate = dateA.value;
  dateA.value = dateB.value;
  dateB.value = tmpDate;
  lastEdited.value = "A";
  recalc();
}

function initTimeA() {
  const { time } = getTimeInTimezone(cityA.value.timezone);
  timeA.value = time.substring(0, 5);
  dateA.value = getTimeInTimezone(cityA.value.timezone).date;
}

// City B search
async function openSearch() {
  searchOpen.value = true;
  searchQuery.value = "";
  searchResults.value = [];
  await nextTick();
  searchInputRef.value?.focus();
}

function closeSearch() {
  searchOpen.value = false;
  searchQuery.value = "";
  searchResults.value = [];
}

let searchTimer: ReturnType<typeof setTimeout> | null = null;

watch(searchQuery, (q) => {
  if (searchTimer) clearTimeout(searchTimer);
  if (!q.trim()) {
    searchResults.value = [];
    return;
  }
  searchTimer = setTimeout(async () => {
    try {
      const resp = await invoke<SearchResponse>("search_cities", { query: q });
      searchResults.value = resp.cities.slice(0, 20);
    } catch {
      searchResults.value = [];
    }
  }, 300);
});

function selectCityB(city: CityResult) {
  cityB.value = city;
  closeSearch();
  dateB.value = getTimeInTimezone(city.timezone).date;
  recalc();
}

function onClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (!target.closest(".city-search")) {
    closeSearch();
  }
  if (!target.closest(".time-picker-wrap")) {
    pickerA.value = false;
    pickerB.value = false;
  }
}

onMounted(() => {
  initTimeA();
  document.addEventListener("click", onClickOutside);
});
onUnmounted(() => {
  document.removeEventListener("click", onClickOutside);
  if (searchTimer) clearTimeout(searchTimer);
});

watch(() => props.cityA, (city) => {
  cityA.value = { ...city };
  initTimeA();
  recalc();
});
</script>

<template>
  <div class="tz-converter">
    <div class="converter-header">
      <span class="title">时区换算</span>
      <button class="close-btn" @click="emit('close')">✕</button>
    </div>

    <div class="tz-card">
      <div class="card-header">
        <span class="city-name">{{ cityA.name_cn || cityA.name }}</span>
        <span class="badge-base">基准</span>
        <span class="city-tz">{{ cityA.timezone }}</span>
      </div>
      <div class="card-date">{{ dateA }}</div>
      <div class="time-picker-wrap" @click.stop>
        <div class="time-display" @click="openPickerA">
          <span class="time-value">{{ timeA }}</span>
        </div>
        <div v-if="pickerA" class="time-selector">
          <div class="ts-body">
            <div class="ts-col">
              <div class="ts-label">时</div>
              <div class="ts-scroll">
                <div
                  v-for="h in hourList"
                  :key="h"
                  class="ts-item"
                  :class="{ active: parseInt(h) === pickerAHours }"
                  @click="pickerAHours = parseInt(h)"
                >{{ h }}</div>
              </div>
            </div>
            <div class="ts-divider"></div>
            <div class="ts-col">
              <div class="ts-label">分</div>
              <div class="ts-scroll">
                <div
                  v-for="m in minList"
                  :key="m"
                  class="ts-item"
                  :class="{ active: parseInt(m) === pickerAMinutes }"
                  @click="pickerAMinutes = parseInt(m)"
                >{{ m }}</div>
              </div>
            </div>
          </div>
          <div class="ts-actions">
            <button class="ts-btn" @click="selectTimeA">确定</button>
          </div>
        </div>
      </div>
    </div>

    <div class="swap-btn" @click="swap">
      <span>&#8693;</span>
    </div>

    <div class="tz-card">
      <div v-if="cityB" class="card-header">
        <span class="city-name">{{ cityB.name_cn || cityB.name }}</span>
        <span class="city-tz">{{ cityB.timezone }}</span>
      </div>
      <div v-else class="city-search" @click.stop>
        <div class="search-trigger" @click="openSearch">
          <input
            v-if="searchOpen"
            ref="searchInputRef"
            v-model="searchQuery"
            class="search-input"
            placeholder="搜索城市..."
          />
          <span v-else class="search-placeholder">选择城市</span>
          <span class="select-arrow">&#9662;</span>
        </div>
        <div v-if="searchOpen && searchResults.length > 0" class="search-dropdown">
          <div
            v-for="c in searchResults"
            :key="c.geonameid"
            class="search-option"
            @click="selectCityB(c)"
          >
            <span class="opt-name">{{ c.name_cn || c.name }}</span>
            <span class="opt-country">{{ c.country_name_cn || c.country_name }}</span>
          </div>
        </div>
        <div v-if="searchOpen && searchQuery && searchResults.length === 0" class="search-empty">
          无匹配
        </div>
      </div>
      <template v-if="cityB">
        <div class="card-date">
          {{ dateB }}
          <span v-if="dayDiffB > 0" class="day-badge">+{{ dayDiffB }}日</span>
          <span v-if="dayDiffB < 0" class="day-badge">{{ dayDiffB }}日</span>
        </div>
        <div class="time-picker-wrap" @click.stop>
          <div class="time-display" @click="openPickerB">
            <span class="time-value">{{ timeB }}</span>
          </div>
          <div v-if="pickerB" class="time-selector">
            <div class="ts-body">
              <div class="ts-col">
                <div class="ts-label">时</div>
                <div class="ts-scroll">
                  <div
                    v-for="h in hourList"
                    :key="h"
                    class="ts-item"
                    :class="{ active: parseInt(h) === pickerBHours }"
                    @click="pickerBHours = parseInt(h)"
                  >{{ h }}</div>
                </div>
              </div>
              <div class="ts-divider"></div>
              <div class="ts-col">
                <div class="ts-label">分</div>
                <div class="ts-scroll">
                  <div
                    v-for="m in minList"
                    :key="m"
                    class="ts-item"
                    :class="{ active: parseInt(m) === pickerBMinutes }"
                    @click="pickerBMinutes = parseInt(m)"
                  >{{ m }}</div>
                </div>
              </div>
            </div>
            <div class="ts-actions">
              <button class="ts-btn" @click="selectTimeB">确定</button>
            </div>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.tz-converter {
  margin-top: 0.5rem;
  background: #1e1e1e;
  border: 1px solid #333;
  border-radius: 12px;
  padding: 0.8rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.converter-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.2rem;
}

.converter-header .title {
  font-size: 0.85rem;
  font-weight: 600;
  color: #888;
}

.close-btn {
  background: none;
  border: none;
  color: #888;
  font-size: 1rem;
  cursor: pointer;
  padding: 0.1rem 0.3rem;
}

.close-btn:hover {
  color: #e0e0e0;
}

.tz-card {
  width: 100%;
  background: #252525;
  border-radius: 12px;
  padding: 0.8rem 1rem;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 6px;
}

.badge-base {
  font-size: 0.6rem;
  background: #667eea;
  color: #fff;
  padding: 1px 5px;
  border-radius: 3px;
  font-weight: 500;
  flex-shrink: 0;
}

.city-name {
  font-size: 0.95rem;
  font-weight: 600;
  color: #e0e0e0;
}

.city-tz {
  font-size: 0.7rem;
  color: #888;
  margin-left: auto;
}

.card-date {
  font-size: 0.75rem;
  color: #888;
  margin-top: 0.2rem;
  margin-bottom: 0.4rem;
  display: flex;
  align-items: center;
  gap: 6px;
}

.day-badge {
  font-size: 0.7rem;
  background: #667eea;
  color: #fff;
  padding: 1px 6px;
  border-radius: 4px;
  font-weight: 500;
}

.time-picker-wrap {
  position: relative;
}

.time-display {
  cursor: pointer;
  user-select: none;
  text-align: right;
  width: 100%;
  padding: 2px 0;
  border-radius: 6px;
  transition: background 0.1s;
}

.time-display:hover {
  background: #2a2a2a;
}

.time-value {
  font-family: "SF Mono", "Monaco", "Consolas", monospace;
  font-weight: 600;
  font-size: 1.8rem;
  color: #667eea;
}

.time-selector {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  background: #252525;
  border: 1px solid #444;
  border-radius: 10px;
  z-index: 60;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.5);
  overflow: hidden;
  min-width: 180px;
}

.ts-body {
  display: flex;
  align-items: stretch;
}

.ts-col {
  flex: 1;
  text-align: center;
}

.ts-label {
  font-size: 0.65rem;
  color: #888;
  padding: 6px 0 4px;
  font-weight: 500;
  border-bottom: 1px solid #333;
}

.ts-scroll {
  height: 180px;
  overflow-y: auto;
  padding: 4px 0;
}

.ts-scroll::-webkit-scrollbar {
  width: 4px;
}

.ts-scroll::-webkit-scrollbar-track {
  background: transparent;
}

.ts-scroll::-webkit-scrollbar-thumb {
  background: #444;
  border-radius: 2px;
}

.ts-item {
  padding: 5px 0;
  font-size: 0.9rem;
  font-family: "SF Mono", "Monaco", "Consolas", monospace;
  color: #ccc;
  cursor: pointer;
  transition: background 0.1s;
}

.ts-item:hover {
  background: #333;
  color: #e0e0e0;
}

.ts-item.active {
  color: #667eea;
  font-weight: 700;
  background: rgba(102, 126, 234, 0.15);
}

.ts-divider {
  width: 1px;
  background: #333;
  align-self: stretch;
}

.ts-actions {
  border-top: 1px solid #333;
}

.ts-btn {
  width: 100%;
  padding: 7px 0;
  border: none;
  background: #667eea;
  color: #fff;
  font-size: 0.8rem;
  cursor: pointer;
  font-weight: 500;
  transition: background 0.1s;
}

.ts-btn:hover {
  background: #5a6fd6;
}

.swap-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #333;
  border-radius: 50%;
  cursor: pointer;
  transition: background 0.15s;
  font-size: 1.1rem;
  color: #667eea;
  user-select: none;
  align-self: center;
}

.swap-btn:hover {
  background: #444;
}

.city-search {
  position: relative;
}

.search-trigger {
  display: flex;
  align-items: center;
  gap: 4px;
  background: #333;
  border: 1px solid #444;
  border-radius: 6px;
  padding: 0.3rem 0.6rem;
  cursor: pointer;
  width: 100%;
  max-width: 220px;
}

.city-search:focus-within .search-trigger {
  border-color: #667eea;
}

.search-placeholder {
  font-size: 0.95rem;
  color: #666;
  flex: 1;
}

.search-input {
  background: none;
  border: none;
  color: #e0e0e0;
  font-size: 0.95rem;
  font-weight: 600;
  outline: none;
  flex: 1;
  min-width: 0;
}

.search-input::placeholder {
  color: #666;
}

.select-arrow {
  color: #888;
  font-size: 0.7rem;
  flex-shrink: 0;
}

.search-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  min-width: 100%;
  background: #2a2a2a;
  border: 1px solid #444;
  border-radius: 8px;
  max-height: 200px;
  overflow-y: auto;
  z-index: 50;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

.search-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0.4rem 0.7rem;
  cursor: pointer;
  font-size: 0.85rem;
  color: #ccc;
}

.search-option:hover {
  background: #333;
}

.opt-name {
  font-weight: 600;
  color: #e0e0e0;
}

.opt-country {
  color: #888;
  font-size: 0.8rem;
}

.search-empty {
  padding: 0.6rem 0.7rem;
  color: #666;
  font-size: 0.8rem;
  background: #2a2a2a;
  border: 1px solid #444;
  border-radius: 8px;
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  min-width: 100%;
}
</style>
