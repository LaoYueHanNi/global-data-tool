<script setup lang="ts">
import type { CityResult } from "../types";
import { useRealtimeTime } from "../utils/time";

const props = defineProps<{
  city: CityResult;
}>();

const emit = defineEmits<{
  select: [city: CityResult];
  contextmenu: [city: CityResult, e: MouseEvent];
}>();

const { time, offset } = useRealtimeTime(() => props.city.timezone);

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
  <div class="city-row" @click="emit('select', city)" @contextmenu.prevent="emit('contextmenu', city, $event)">
    <div class="left">
      <span class="capital" :title="city.is_capital ? '首都' : ''">{{ city.is_capital ? '★' : '' }}</span>
      <span class="name" :title="city.name_cn || city.name">{{ city.name_cn || city.name }}</span>
      <span class="name-en" :title="city.name">{{ city.name_cn ? city.name : '' }}</span>
      <span class="feature" :title="getFeatureLabel(city.feature_code)">{{ getFeatureLabel(city.feature_code) }}</span>
      <span class="country" :title="city.country_name_cn || city.country_name">{{ city.country_name_cn || city.country_name }}</span>
    </div>
    <div class="right">
      <span class="time">{{ time }}</span>
      <span class="offset">{{ offset }}</span>
    </div>
  </div>
</template>

<style scoped>
.city-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.35rem 0.6rem;
  background: #252525;
  border-bottom: 1px solid #333;
  font-size: 0.85rem;
  cursor: pointer;
}

.city-row:hover {
  background: #2a2a2a;
}

.city-row:last-child {
  border-bottom: none;
}

.left {
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  flex: 1;
}

.right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  margin-left: 12px;
}

.capital {
  width: 18px;
  color: #f5a623;
  font-size: 0.8rem;
  text-align: center;
  flex-shrink: 0;
}

.name {
  font-weight: 500;
  color: #e0e0e0;
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
}

.name-en {
  color: #888;
  font-size: 0.75rem;
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
}

.feature {
  color: #aaa;
  font-size: 0.7rem;
  max-width: 50px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
}

.country {
  color: #888;
  font-size: 0.8rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.time {
  font-family: "SF Mono", "Monaco", monospace;
  font-weight: 600;
  color: #667eea;
}

.offset {
  color: #888;
  font-size: 0.75rem;
}
</style>
