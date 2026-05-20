<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import SearchBox from "./SearchBox.vue";
import ResultList from "./ResultList.vue";
import CityDetail from "./CityDetail.vue";
import { getTimeInTimezone } from "../utils/time";
import type { SearchResponse, CityResult, CitySummary } from "../types";

const searchResult = ref<SearchResponse | null>(null);
const recentCities = ref<CityResult[]>([]);
const triggerQuery = ref("");
const selectedCity = ref<CityResult | null>(null);
const now = ref(Date.now());

let timer: ReturnType<typeof setInterval>;
onMounted(() => {
  timer = setInterval(() => {
    now.value = Date.now();
  }, 1000);
});
onUnmounted(() => {
  if (timer) clearInterval(timer);
});

function getRealTime(timezone: string) {
  void now.value;
  return getTimeInTimezone(timezone);
}

function getFeatureLabel(code: string | null | undefined): string {
  if (!code) return '';
  const labels: Record<string, string> = {
    'PPLC': '首都',
    'PPLA': '省会',
    'PPLA2': '地级市',
    'PPLA3': '县级市',
    'PPLA4': '县',
    'PPL': '',
  };
  return labels[code] || '';
}

onMounted(async () => {
  try {
    recentCities.value = await invoke<CityResult[]>("get_recent_cities");
  } catch (e) {
    console.error("Failed to load recent searches:", e);
  }
});

function onSearch(result: SearchResponse) {
  searchResult.value = result;
}

function onClear() {
  searchResult.value = null;
}

function onSelectCity(city: CityResult) {
  selectedCity.value = city;
  addRecent(city);
}

function onCloseDetail() {
  selectedCity.value = null;
}

async function addRecent(city: CityResult) {
  try {
    await invoke("add_recent_city", { city });
    recentCities.value = await invoke<CityResult[]>("get_recent_cities");
  } catch (e) {
    console.error("Failed to save recent:", e);
  }
}

function onClickRecent(q: string) {
  triggerQuery.value = q;
}

async function onSearchCountryCity(citySummary: CitySummary) {
  const query = citySummary.name_cn || citySummary.name;
  try {
    const result = await invoke<SearchResponse>("search_cities", { query });
    const found = result.cities.find(c => c.geonameid === citySummary.geonameid);
    if (found) {
      selectedCity.value = found;
      addRecent(found);
    }
  } catch (err) {
    console.error("Search failed:", err);
  }
}
</script>

<template>
  <div class="city-time-page">
    <SearchBox
      @search="onSearch"
      @clear="onClear"
      @recent="addRecent"
      :trigger-query="triggerQuery"
    />
    <ResultList
      v-if="searchResult"
      :result="searchResult"
      @select="onSelectCity"
      @searchCountryCity="onSearchCountryCity"
    />
    <div v-else-if="recentCities.length > 0" class="recent-list">
      <div class="recent-title">最近搜索</div>
      <div
        v-for="city in recentCities"
        :key="city.geonameid"
        class="recent-item"
        @click="onClickRecent(city.name_cn || city.name)"
      >
        <div class="left">
          <span class="capital" :title="city.is_capital ? '首都' : ''">{{ city.is_capital ? '★' : '' }}</span>
          <span class="name" :title="city.name_cn || city.name">{{ city.name_cn || city.name }}</span>
          <span class="name-en" :title="city.name">{{ city.name_cn ? city.name : '' }}</span>
          <span class="feature" :title="getFeatureLabel(city.feature_code)">{{ getFeatureLabel(city.feature_code) }}</span>
          <span class="country" :title="city.country_name_cn || city.country_name">{{ city.country_name_cn || city.country_name }}</span>
        </div>
        <div class="right">
          <span class="time">{{ getRealTime(city.timezone).time }}</span>
          <span class="offset">{{ getRealTime(city.timezone).offset }}</span>
        </div>
      </div>
    </div>

    <CityDetail v-if="selectedCity" :city="selectedCity" @close="onCloseDetail" />
  </div>
</template>

<style scoped>
.city-time-page {
  padding: 1rem;
  min-height: 100vh;
}

.recent-list {
  margin-top: 0.5rem;
  background: #252525;
  border-radius: 8px;
  overflow: hidden;
}

.recent-title {
  font-size: 0.8rem;
  color: #888;
  padding: 0.5rem 0.6rem;
  border-bottom: 1px solid #333;
}

.recent-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.35rem 0.6rem;
  border-bottom: 1px solid #333;
  font-size: 0.85rem;
  color: #e0e0e0;
  cursor: pointer;
}

.recent-item:hover {
  background: #2a2a2a;
}

.recent-item:last-child {
  border-bottom: none;
}

.recent-item .left {
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  flex: 1;
}

.recent-item .right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  margin-left: 12px;
}

.recent-item .capital {
  width: 18px;
  color: #f5a623;
  font-size: 0.8rem;
  text-align: center;
  flex-shrink: 0;
}

.recent-item .name {
  font-weight: 500;
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
}

.recent-item .name-en {
  color: #888;
  font-size: 0.75rem;
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
}

.recent-item .feature {
  color: #aaa;
  font-size: 0.7rem;
  max-width: 50px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
}

.recent-item .country {
  color: #888;
  font-size: 0.8rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.recent-item .time {
  font-family: monospace;
  color: #667eea;
}

.recent-item .offset {
  color: #888;
  font-size: 0.75rem;
}
</style>
