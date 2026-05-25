<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import CityCard from "./CityCard.vue";
import { getTimeInTimezone } from "../utils/time";
import type { SearchResponse, CityResult, CitySummary } from "../types";

defineProps<{
  result: SearchResponse;
}>();

const emit = defineEmits<{
  select: [city: CityResult];
  searchCountryCity: [city: CitySummary];
  contextmenu: [city: CityResult | CitySummary, e: MouseEvent];
}>();

const now = ref(Date.now());
let timer: ReturnType<typeof setInterval>;
onMounted(() => {
  timer = setInterval(() => { now.value = Date.now(); }, 1000);
});
onUnmounted(() => {
  if (timer) clearInterval(timer);
});

function getRealTime(timezone: string) {
  void now.value;
  return getTimeInTimezone(timezone);
}
</script>

<template>
  <div class="result-list">
    <div v-for="country in result.countries" :key="country.country_iso" class="country-section">
      <div class="country-title">
        {{ country.country_name_cn || country.country_name }}
        <span v-if="country.continent" class="continent">{{ country.continent }}</span>
      </div>
      <div
        v-for="city in country.cities"
        :key="city.geonameid"
        class="country-city"
        @click="emit('searchCountryCity', city)"
        @contextmenu.prevent="emit('contextmenu', city, $event)"
      >
        <span class="capital-mark" v-if="city.is_capital">★</span>
        <span class="city-name">{{ city.name_cn || city.name }}</span>
        <span class="city-time">{{ getRealTime(city.timezone).time }}</span>
      </div>
    </div>

    <div class="cities-container">
      <CityCard
        v-for="city in result.cities"
        :key="city.geonameid"
        :city="city"
        @select="emit('select', city)"
        @contextmenu="(city: CityResult, e: MouseEvent) => emit('contextmenu', city, e)"
      />
    </div>
    <div v-if="result.cities.length === 0" class="no-result">
      未找到匹配的城市
    </div>
  </div>
</template>

<style scoped>
.result-list {
  margin-top: 0.5rem;
}

.country-section {
  background: #252525;
  border-radius: 8px;
  padding: 0.5rem 0.6rem;
  margin-bottom: 0.5rem;
}

.country-title {
  font-size: 0.8rem;
  color: #888;
  margin-bottom: 0.3rem;
  font-weight: 500;
}

.continent {
  font-size: 0.7rem;
  color: #667eea;
  margin-left: 0.4rem;
}

.country-city {
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  background: #333;
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  margin: 0.15rem;
  cursor: pointer;
}

.country-city:hover {
  background: #444;
}

.country-city .capital-mark {
  color: #f5a623;
  font-size: 0.7rem;
}

.country-city .city-name {
  color: #e0e0e0;
}

.country-city .city-time {
  color: #667eea;
  font-family: monospace;
  margin-left: 0.3rem;
}

.cities-container {
  background: #252525;
  border-radius: 8px;
  overflow: hidden;
}

.no-result {
  text-align: center;
  padding: 2rem;
  color: #888;
}
</style>
