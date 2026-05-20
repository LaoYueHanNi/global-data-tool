<script setup lang="ts">
interface Tab {
  id: string;
  icon: string;
  label: string;
}

defineProps<{
  tabs: Tab[];
  activeTab: string;
}>();

const emit = defineEmits<{
  switch: [id: string];
}>();
</script>

<template>
  <div class="side-tab">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      class="tab-item"
      :class="{ active: tab.id === activeTab }"
      @click="emit('switch', tab.id)"
    >
      <span class="tab-icon">{{ tab.icon }}</span>
      <span class="tab-label">{{ tab.label }}</span>
    </div>
  </div>
</template>

<style scoped>
.side-tab {
  width: 64px;
  background: #1e1e1e;
  border-right: 1px solid #333;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 12px;
  flex-shrink: 0;
}

.tab-item {
  width: 56px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 8px 4px;
  border-radius: 6px;
  cursor: pointer;
  margin-bottom: 4px;
  transition: background 0.15s;
}

.tab-item:hover {
  background: #2a2a2a;
}

.tab-item.active {
  background: #333;
}

.tab-icon {
  font-size: 1.2rem;
  line-height: 1;
}

.tab-label {
  font-size: 0.6rem;
  color: #999;
  white-space: nowrap;
}

.tab-item.active .tab-label {
  color: #667eea;
}
</style>
