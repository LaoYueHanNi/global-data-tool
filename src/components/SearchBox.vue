<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { SearchResponse, CityResult } from "../types";

const props = defineProps<{
  triggerQuery?: string;
}>();

const emit = defineEmits<{
  search: [result: SearchResponse];
  clear: [];
  recent: [city: CityResult];
}>();

const query = ref("");
const loading = ref(false);
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(() => props.triggerQuery, (newVal) => {
  if (newVal) {
    query.value = newVal;
  }
});

watch(query, (newVal) => {
  if (!newVal.trim()) {
    emit("clear");
    return;
  }
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
  debounceTimer = setTimeout(() => {
    doSearch();
  }, 300);
});

async function doSearch() {
  const q = query.value.trim();
  if (!q) return;

  loading.value = true;
  try {
    const result = await invoke<SearchResponse>("search_cities", { query: q });
    emit("search", result);
    if (result.cities.length === 1) {
      emit("recent", result.cities[0]);
    }
  } catch (err) {
    console.error("Search failed:", err);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="search-box">
    <input
      v-model="query"
      type="text"
      class="search-input"
      placeholder="搜索城市或国家..."
      autofocus
    />
    <span v-if="loading" class="loading">...</span>
  </div>
</template>

<style scoped>
.search-box {
  position: relative;
}

.search-input {
  width: 100%;
  padding: 0.6rem 0.8rem;
  font-size: 0.95rem;
  border: 1px solid #444;
  border-radius: 8px;
  background: #252525;
  color: #e0e0e0;
  outline: none;
  transition: border-color 0.2s;
}

.search-input:focus {
  border-color: #667eea;
}

.search-input::placeholder {
  color: #666;
}

.loading {
  position: absolute;
  right: 0.8rem;
  top: 50%;
  transform: translateY(-50%);
  color: #888;
  font-size: 0.85rem;
}
</style>
