<script setup lang="ts">
import { reactive, provide, inject, type InjectionKey } from "vue";

export interface MenuItem {
  label: string;
  icon?: string;
  action: () => void;
}

export interface MenuState {
  visible: boolean;
  x: number;
  y: number;
  items: MenuItem[];
}

const menu = reactive<MenuState>({
  visible: false,
  x: 0,
  y: 0,
  items: [],
});

function showMenu(x: number, y: number, items: MenuItem[]) {
  menu.x = x;
  menu.y = y;
  menu.items = items;
  menu.visible = true;
}

function hideMenu() {
  menu.visible = false;
}

function onItemClick(item: MenuItem) {
  item.action();
  hideMenu();
}

function onClickOutside() {
  if (menu.visible) hideMenu();
}

provide(contextMenuKey, { showMenu, hideMenu });
</script>

<template>
  <slot />
  <Teleport to="body">
    <div
      v-if="menu.visible"
      class="context-menu-overlay"
      @click="onClickOutside"
      @contextmenu.prevent="onClickOutside"
    >
      <div
        class="context-menu"
        :style="{ left: menu.x + 'px', top: menu.y + 'px' }"
      >
        <div
          v-for="(item, i) in menu.items"
          :key="i"
          class="menu-item"
          @click.stop="onItemClick(item)"
        >
          <span v-if="item.icon" class="menu-icon">{{ item.icon }}</span>
          <span class="menu-label">{{ item.label }}</span>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script lang="ts">
export const contextMenuKey: InjectionKey<{
  showMenu: (x: number, y: number, items: MenuItem[]) => void;
  hideMenu: () => void;
}> = Symbol("context-menu");

export function useContextMenu() {
  const ctx = inject(contextMenuKey);
  if (!ctx) throw new Error("useContextMenu must be used inside ContextMenuProvider");
  return ctx;
}
</script>

<style scoped>
.context-menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 9999;
}

.context-menu {
  position: absolute;
  background: #2a2a2a;
  border: 1px solid #444;
  border-radius: 8px;
  padding: 4px 0;
  min-width: 160px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  cursor: pointer;
  font-size: 0.8rem;
  color: #e0e0e0;
  transition: background 0.1s;
}

.menu-item:hover {
  background: #667eea;
  color: #fff;
}

.menu-icon {
  font-size: 0.85rem;
  width: 16px;
  text-align: center;
}

.menu-label {
  white-space: nowrap;
}
</style>
