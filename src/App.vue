<script setup lang="ts">
import { ref } from "vue";
import SideTab from "./components/SideTab.vue";
import CityTimePage from "./components/CityTimePage.vue";
import CurrencyPage from "./components/CurrencyPage.vue";

const activeTab = ref("city-time");

const tabs = [
  { id: "city-time", icon: "🕐", label: "City Time" },
  { id: "currency", icon: "💱", label: "汇率" },
];
</script>

<template>
  <div class="app-shell">
    <SideTab :tabs="tabs" :active-tab="activeTab" @switch="activeTab = $event" />
    <div class="content-area">
      <CityTimePage v-show="activeTab === 'city-time'" />
      <CurrencyPage v-show="activeTab === 'currency'" />
    </div>
  </div>
</template>

<style>
@import "./styles/main.css";

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  overflow: hidden;
}

.app-shell {
  display: flex;
  height: 100vh;
  background: #1a1a1a;
}

.content-area {
  flex: 1;
  overflow-y: auto;
  min-width: 0;
}
</style>
