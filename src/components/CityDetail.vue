<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import type { CityResult } from "../types";
import { getTimeInTimezone } from "../utils/time";

const props = defineProps<{
  city: CityResult;
}>();

const emit = defineEmits<{
  close: [];
}>();

const now = ref(Date.now());
let timer: ReturnType<typeof setInterval>;
onMounted(() => {
  timer = setInterval(() => { now.value = Date.now(); }, 1000);
});
onUnmounted(() => {
  if (timer) clearInterval(timer);
});

function getRealTime() {
  void now.value;
  return getTimeInTimezone(props.city.timezone);
}

function formatPopulation(pop: number): string {
  if (pop >= 1000000) {
    return (pop / 1000000).toFixed(1) + "M";
  } else if (pop >= 1000) {
    return (pop / 1000).toFixed(0) + "K";
  }
  return pop.toString();
}

function formatCoord(lat: number, lng: number): string {
  const latDir = lat >= 0 ? "N" : "S";
  const lngDir = lng >= 0 ? "E" : "W";
  return `${Math.abs(lat).toFixed(4)}°${latDir}, ${Math.abs(lng).toFixed(4)}°${lngDir}`;
}

function getFeatureLabel(code: string | null): string {
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
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="detail-card">
      <button class="close-btn" @click="emit('close')">✕</button>

      <div class="header">
        <div class="badges">
          <span class="capital-badge" v-if="city.is_capital">★ 首都</span>
          <span class="feature-badge" v-if="getFeatureLabel(city.feature_code)">{{ getFeatureLabel(city.feature_code) }}</span>
        </div>
        <h2 class="city-name">{{ city.name_cn || city.name }}</h2>
        <div class="city-name-en" v-if="city.name_cn">{{ city.name }}</div>
      </div>

      <div class="info-grid">
        <div class="info-item">
          <div class="label">国家</div>
          <div class="value">{{ city.country_name_cn || city.country_name }}</div>
        </div>
        <div class="info-item">
          <div class="label">当地时间</div>
          <div class="value time">{{ getRealTime().time }}</div>
        </div>
        <div class="info-item">
          <div class="label">当地日期</div>
          <div class="value">{{ getRealTime().date }}</div>
        </div>
        <div class="info-item">
          <div class="label">时区</div>
          <div class="value">{{ city.timezone }}</div>
        </div>
        <div class="info-item">
          <div class="label">UTC偏移</div>
          <div class="value">{{ getRealTime().offset }}</div>
        </div>
        <div class="info-item">
          <div class="label">人口</div>
          <div class="value">{{ formatPopulation(city.population) }}</div>
        </div>
        <div class="info-item full">
          <div class="label">坐标</div>
          <div class="value">{{ formatCoord(city.latitude, city.longitude) }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  padding: 1rem;
}

.detail-card {
  background: #252525;
  border-radius: 12px;
  padding: 1.5rem;
  width: 100%;
  max-width: 360px;
  position: relative;
}

.close-btn {
  position: absolute;
  top: 0.8rem;
  right: 0.8rem;
  background: none;
  border: none;
  color: #888;
  font-size: 1.2rem;
  cursor: pointer;
  padding: 0.2rem;
}

.close-btn:hover {
  color: #e0e0e0;
}

.header {
  margin-bottom: 1.2rem;
  text-align: center;
}

.badges {
  display: flex;
  justify-content: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.capital-badge {
  display: inline-block;
  background: #f5a623;
  color: #000;
  font-size: 0.75rem;
  padding: 0.15rem 0.5rem;
  border-radius: 4px;
}

.feature-badge {
  display: inline-block;
  background: #444;
  color: #aaa;
  font-size: 0.75rem;
  padding: 0.15rem 0.5rem;
  border-radius: 4px;
}

.city-name {
  font-size: 1.4rem;
  font-weight: 600;
  color: #e0e0e0;
  margin: 0;
}

.city-name-en {
  font-size: 0.9rem;
  color: #888;
  margin-top: 0.2rem;
}

.info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.8rem;
}

.info-item {
  background: #333;
  padding: 0.6rem;
  border-radius: 6px;
}

.info-item.full {
  grid-column: 1 / -1;
}

.label {
  font-size: 0.7rem;
  color: #888;
  margin-bottom: 0.2rem;
}

.value {
  font-size: 0.9rem;
  color: #e0e0e0;
}

.value.time {
  font-family: monospace;
  font-weight: 600;
  color: #667eea;
  font-size: 1.1rem;
}
</style>
