<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ExchangeRates } from "../types";
import { CURRENCY_NAMES } from "../utils/currencies";

interface CurrencyMeta {
  code: string;
  name: string;
  nameCn: string;
}

const topCurrency = ref("CNY");
const bottomCurrency = ref("USD");
const rates = ref<Record<string, number>>({});
const updatedAt = ref("");

const currencyList = computed<CurrencyMeta[]>(() => {
  return Object.keys(rates.value)
    .filter((code) => code.length === 3)
    .sort()
    .map((code) => {
      const meta = CURRENCY_NAMES[code];
      return {
        code,
        name: meta?.name || code,
        nameCn: meta?.nameCn || "",
      };
    });
});

// Search state
const topQuery = ref("");
const topOpen = ref(false);
const topInputRef = ref<HTMLInputElement | null>(null);
const bottomQuery = ref("");
const bottomOpen = ref(false);
const bottomInputRef = ref<HTMLInputElement | null>(null);

function filterCurrencies(q: string) {
  const s = q.toLowerCase().trim();
  if (!s) return currencyList.value;
  return currencyList.value.filter(
    (c) =>
      c.code.toLowerCase().includes(s) ||
      c.nameCn.includes(s) ||
      c.name.toLowerCase().includes(s)
  );
}

const topFiltered = computed(() => filterCurrencies(topQuery.value));
const bottomFiltered = computed(() => filterCurrencies(bottomQuery.value));

function openTop() {
  topOpen.value = true;
  topQuery.value = "";
  bottomOpen.value = false;
  nextTick(() => topInputRef.value?.focus());
}

function openBottom() {
  bottomOpen.value = true;
  bottomQuery.value = "";
  topOpen.value = false;
  nextTick(() => bottomInputRef.value?.focus());
}

function selectTop(code: string) {
  topCurrency.value = code;
  topOpen.value = false;
}

function selectBottom(code: string) {
  bottomCurrency.value = code;
  bottomOpen.value = false;
}

function onClickOutside(e: MouseEvent) {
  if (!(e.target as HTMLElement).closest(".search-select")) {
    topOpen.value = false;
    bottomOpen.value = false;
  }
}

onMounted(() => document.addEventListener("click", onClickOutside));
onUnmounted(() => document.removeEventListener("click", onClickOutside));

// Rate logic — rates are "1 CNY = X foreign"
function valueInCny(code: string): number {
  const r = rates.value[code];
  if (!r || r === 0) return 1;
  return 1 / r;
}

const ratio = computed(() => {
  const top = valueInCny(topCurrency.value);
  const bottom = valueInCny(bottomCurrency.value);
  if (bottom === 0) return 1;
  return top / bottom;
});

// Amount input: topAmount / bottomAmount, bidirectional
const topAmount = ref("1");
const bottomAmount = ref("");
const lastEdited = ref<"top" | "bottom">("top");

function addCommas(s: string): string {
  const [int, dec] = s.split(".");
  const formatted = int.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  return dec !== undefined ? `${formatted}.${dec}` : formatted;
}

function removeCommas(s: string): string {
  return s.replace(/,/g, "");
}

function fmtCalc(n: number): string {
  let s: string;
  if (Number.isInteger(n)) s = n.toString();
  else if (n >= 1000) s = n.toFixed(2).replace(/\.?0+$/, "");
  else if (n >= 1) s = n.toFixed(4).replace(/\.?0+$/, "");
  else s = n.toFixed(6).replace(/\.?0+$/, "");
  return addCommas(s);
}

function recalc() {
  if (lastEdited.value === "top") {
    const val = parseFloat(removeCommas(topAmount.value));
    if (isNaN(val)) { bottomAmount.value = ""; return; }
    bottomAmount.value = fmtCalc(val * ratio.value);
  } else {
    const val = parseFloat(removeCommas(bottomAmount.value));
    if (isNaN(val)) { topAmount.value = ""; return; }
    topAmount.value = fmtCalc(val / ratio.value);
  }
}

function onTopInput() {
  lastEdited.value = "top";
  recalc();
}

function onBottomInput() {
  lastEdited.value = "bottom";
  recalc();
}

function onBlurTop() {
  const val = parseFloat(removeCommas(topAmount.value));
  if (!isNaN(val) && val !== 0) topAmount.value = fmtCalc(val);
}

function onBlurBottom() {
  const val = parseFloat(removeCommas(bottomAmount.value));
  if (!isNaN(val) && val !== 0) bottomAmount.value = fmtCalc(val);
}

// Currency or ratio change → recalc from last edited side
watch([topCurrency, bottomCurrency, ratio], () => {
  nextTick(recalc);
});

function swap() {
  const tc = topCurrency.value;
  topCurrency.value = bottomCurrency.value;
  bottomCurrency.value = tc;
  const ta = topAmount.value;
  topAmount.value = bottomAmount.value;
  bottomAmount.value = ta;
}

// Rate label (smart flip for display)
function getCnName(code: string): string {
  return CURRENCY_NAMES[code]?.nameCn || "";
}

const rateLabel = computed(() => {
  const r = ratio.value;
  const topCn = getCnName(topCurrency.value);
  const bottomCn = getCnName(bottomCurrency.value);
  if (r >= 1) {
    return `1 ${topCurrency.value}${topCn ? " " + topCn : ""} = ${fmtCalc(r)} ${bottomCurrency.value}${bottomCn ? " " + bottomCn : ""}`;
  }
  return `1 ${bottomCurrency.value}${bottomCn ? " " + bottomCn : ""} = ${fmtCalc(1 / r)} ${topCurrency.value}${topCn ? " " + topCn : ""}`;
});

function formatTime(utcStr: string): string {
  const d = new Date(utcStr);
  if (isNaN(d.getTime())) return utcStr;
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

// Data loading
async function loadRates() {
  let needRefresh = false;
  try {
    const cached = await invoke<ExchangeRates | null>("get_exchange_rates");
    if (cached && cached.rates && Object.keys(cached.rates).length > 0) {
      rates.value = cached.rates;
      updatedAt.value = cached.updated_at;
      if (cached.next_update && new Date(cached.next_update) > new Date()) {
        return;
      }
      needRefresh = true;
    }
  } catch (e) {
    console.error("Failed to read cached rates:", e);
  }
  if (needRefresh || Object.keys(rates.value).length === 0) {
    refreshInBackground();
  }
}

async function refreshInBackground() {
  try {
    const resp = await fetch("https://open.er-api.com/v6/latest/CNY");
    if (!resp.ok) return;
    const data = await resp.json();
    if (data.result !== "success" || !data.rates) return;

    const er: ExchangeRates = {
      base: data.base_code,
      rates: data.rates,
      updated_at: data.time_last_update_utc,
      next_update: data.time_next_update_utc,
    };
    await invoke("save_exchange_rates", { rates: er });
    rates.value = er.rates;
    updatedAt.value = er.updated_at;
  } catch (e) {
    console.error("Failed to refresh rates:", e);
  }
}

onMounted(loadRates);
</script>

<template>
  <div class="currency-page">
    <div class="currency-card">
      <div class="search-select" @click.stop>
        <div class="select-trigger" @click="openTop">
          <input
            v-if="topOpen"
            ref="topInputRef"
            v-model="topQuery"
            class="search-input"
            placeholder="搜索..."
          />
          <span v-else class="select-value">{{ topCurrency }} {{ currencyList.find(c => c.code === topCurrency)?.nameCn }}</span>
          <span class="select-arrow">&#9662;</span>
        </div>
        <div v-if="topOpen" class="select-dropdown">
          <div
            v-for="c in topFiltered"
            :key="c.code"
            class="select-option"
            :class="{ active: c.code === topCurrency }"
            @click="selectTop(c.code)"
          >
            <span class="opt-code">{{ c.code }}</span>
            <span class="opt-name">{{ c.nameCn }}</span>
          </div>
          <div v-if="topFiltered.length === 0" class="select-empty">无匹配</div>
        </div>
      </div>
      <input
        class="amount-input"
        v-model="topAmount"
        @input="onTopInput"
        @blur="onBlurTop"
        type="text"
        inputmode="decimal"
        placeholder="0"
      />
    </div>

    <div class="swap-btn" @click="swap">
      <span>&#8693;</span>
    </div>

    <div class="currency-card">
      <div class="search-select" @click.stop>
        <div class="select-trigger" @click="openBottom">
          <input
            v-if="bottomOpen"
            ref="bottomInputRef"
            v-model="bottomQuery"
            class="search-input"
            placeholder="搜索..."
          />
          <span v-else class="select-value">{{ bottomCurrency }} {{ currencyList.find(c => c.code === bottomCurrency)?.nameCn }}</span>
          <span class="select-arrow">&#9662;</span>
        </div>
        <div v-if="bottomOpen" class="select-dropdown">
          <div
            v-for="c in bottomFiltered"
            :key="c.code"
            class="select-option"
            :class="{ active: c.code === bottomCurrency }"
            @click="selectBottom(c.code)"
          >
            <span class="opt-code">{{ c.code }}</span>
            <span class="opt-name">{{ c.nameCn }}</span>
          </div>
          <div v-if="bottomFiltered.length === 0" class="select-empty">无匹配</div>
        </div>
      </div>
      <input
        class="amount-input"
        v-model="bottomAmount"
        @input="onBottomInput"
        @blur="onBlurBottom"
        type="text"
        inputmode="decimal"
        placeholder="0"
      />
    </div>

    <div class="rate-info">
      <div class="rate-label">{{ rateLabel }}</div>
      <div class="rate-updated">
        <span v-if="updatedAt">更新于: {{ formatTime(updatedAt) }}</span>
      </div>
      <div class="rate-source">数据来源: open.er-api.com</div>
    </div>
  </div>
</template>

<style scoped>
.currency-page {
  padding: 1.2rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  min-height: 100vh;
}

.currency-card {
  width: 100%;
  background: #252525;
  border-radius: 12px;
  padding: 0.8rem 1rem;
  height: 100px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
}

.search-select {
  position: relative;
}

.select-trigger {
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

.search-select:focus-within .select-trigger {
  border-color: #667eea;
}

.select-value {
  font-size: 0.95rem;
  font-weight: 600;
  color: #e0e0e0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.select-arrow {
  color: #888;
  font-size: 0.7rem;
  flex-shrink: 0;
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

.select-dropdown {
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

.select-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0.4rem 0.7rem;
  cursor: pointer;
  font-size: 0.85rem;
  color: #ccc;
}

.select-option:hover {
  background: #333;
}

.select-option.active {
  color: #667eea;
}

.opt-code {
  font-weight: 600;
  color: #e0e0e0;
}

.select-option.active .opt-code {
  color: #667eea;
}

.opt-name {
  color: #888;
  font-size: 0.8rem;
}

.select-empty {
  padding: 0.6rem 0.7rem;
  color: #666;
  font-size: 0.8rem;
}

.amount-input {
  font-family: "SF Mono", "Monaco", "Consolas", monospace;
  font-weight: 600;
  font-size: 1.8rem;
  color: #667eea;
  background: none;
  border: none;
  outline: none;
  text-align: right;
  width: 100%;
}

.amount-input::placeholder {
  color: #444;
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
}

.swap-btn:hover {
  background: #444;
}

.rate-info {
  width: 100%;
  text-align: center;
  padding: 0.8rem;
  background: #252525;
  border-radius: 8px;
  margin-top: 0.3rem;
}

.rate-label {
  font-size: 0.9rem;
  color: #e0e0e0;
  font-weight: 500;
}

.rate-updated {
  font-size: 0.75rem;
  color: #666;
  margin-top: 0.3rem;
}

.rate-source {
  font-size: 0.7rem;
  color: #555;
  margin-top: 0.2rem;
}
</style>
