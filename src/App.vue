<template>
  <main v-if="isSettingsWindow" class="settings-shell">
    <template v-if="settingsMask">
      <header class="settings-header">
        <div>
          <span class="eyebrow">Mask appearance</span>
          <h1>{{ settingsMask.name }}</h1>
        </div>
        <button type="button" class="settings-close" aria-label="Close settings" title="Close settings" @click="hideSettings">
          ×
        </button>
      </header>

      <section class="settings-section">
        <label for="mask-color">Color</label>
        <div class="color-row">
          <input id="mask-color" v-model="settingsColor" type="color" @input="scheduleAppearanceUpdate" />
          <code>{{ settingsColor.toUpperCase() }}</code>
        </div>
        <div class="presets" aria-label="Color presets">
          <button
            v-for="preset in colorPresets"
            :key="preset"
            type="button"
            class="preset"
            :style="{ backgroundColor: preset }"
            :aria-label="`Use color ${preset}`"
            :title="preset"
            @click="selectPreset(preset)"
          />
        </div>
      </section>

      <section class="settings-section">
        <label for="mask-opacity">Opacity: {{ settingsOpacity }}%</label>
        <input
          id="mask-opacity"
          v-model.number="settingsOpacity"
          type="range"
          min="10"
          max="100"
          step="10"
          @input="scheduleAppearanceUpdate"
        />
      </section>

      <p v-if="errorMessage" class="settings-error" role="alert">{{ errorMessage }}</p>
      <p class="settings-help">Changes are applied immediately. Locked masks remain controllable from the system tray.</p>
    </template>
    <div v-else class="empty-settings">
      <h1>No mask selected</h1>
      <p>Choose Appearance from a mask menu or from the system tray.</p>
    </div>
  </main>

  <div
    v-else
    class="mask-container"
    :class="{ 'is-locked': maskState?.locked }"
    :style="{ backgroundColor: maskBackground }"
    @mousedown.self="startMove"
    @dblclick.prevent.stop
    @contextmenu.prevent="showMaskMenu"
  >
    <template v-if="maskState && !maskState.locked">
      <div
        v-for="handle in resizeHandles"
        :key="handle.className"
        class="resize-handle"
        :class="handle.className"
        @mousedown.stop.prevent="startResize($event, handle.direction)"
      />

      <div class="mask-toolbar" @mousedown.stop @contextmenu.stop.prevent="showMaskMenu">
        <button type="button" class="icon-button" aria-label="Mask settings" title="Mask settings" @click.stop="showMaskMenu">
          ⋯
        </button>
        <button
          type="button"
          class="icon-button"
          aria-label="Lock and enable click-through"
          title="Lock and enable click-through"
          :disabled="busy || !maskState.trayReady"
          @click.stop="lockMask"
        >
          🔒
        </button>
        <button
          type="button"
          class="icon-button delete-button"
          aria-label="Delete mask"
          title="Delete mask"
          :disabled="busy"
          @click.stop="deleteMask"
        >
          ×
        </button>
      </div>

      <div v-if="errorMessage" class="mask-error" role="alert">{{ errorMessage }}</div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

type ResizeDirection =
  | 'East'
  | 'North'
  | 'NorthEast'
  | 'NorthWest'
  | 'South'
  | 'SouthEast'
  | 'SouthWest'
  | 'West';

interface MaskViewState {
  id: string;
  name: string;
  color: string;
  opacity: number;
  locked: boolean;
  visible: boolean;
  trayReady: boolean;
}

const appWindow = getCurrentWindow();
const isSettingsWindow = new URLSearchParams(window.location.search).get('view') === 'settings';
const maskState = ref<MaskViewState | null>(null);
const settingsMask = ref<MaskViewState | null>(null);
const settingsColor = ref('#000000');
const settingsOpacity = ref(90);
const busy = ref(false);
const errorMessage = ref('');
const colorPresets = ['#000000', '#FFFFFF', '#555555', '#1D4ED8', '#B91C1C'];

const resizeHandles: Array<{ className: string; direction: ResizeDirection }> = [
  { className: 'north', direction: 'North' },
  { className: 'south', direction: 'South' },
  { className: 'west', direction: 'West' },
  { className: 'east', direction: 'East' },
  { className: 'north-west', direction: 'NorthWest' },
  { className: 'north-east', direction: 'NorthEast' },
  { className: 'south-west', direction: 'SouthWest' },
  { className: 'south-east', direction: 'SouthEast' },
];

const maskBackground = computed(() => {
  const state = maskState.value;
  if (!state) return 'rgba(0, 0, 0, 0.9)';
  const hex = state.color.replace('#', '');
  const red = Number.parseInt(hex.slice(0, 2), 16);
  const green = Number.parseInt(hex.slice(2, 4), 16);
  const blue = Number.parseInt(hex.slice(4, 6), 16);
  return `rgba(${red}, ${green}, ${blue}, ${state.opacity / 100})`;
});

let unlistenMask: UnlistenFn | undefined;
let unlistenSettings: UnlistenFn | undefined;
let appearanceTimer: ReturnType<typeof setTimeout> | undefined;

function reportError(error: unknown) {
  errorMessage.value = error instanceof Error ? error.message : String(error);
  window.setTimeout(() => {
    errorMessage.value = '';
  }, 5000);
}

async function startMove(event: MouseEvent) {
  if (event.button !== 0 || event.detail !== 1 || maskState.value?.locked) return;
  try {
    await appWindow.startDragging();
  } catch (error) {
    reportError(error);
  }
}

async function startResize(event: MouseEvent, direction: ResizeDirection) {
  if (event.button !== 0 || maskState.value?.locked) return;
  try {
    await appWindow.startResizeDragging(direction);
  } catch (error) {
    reportError(error);
  }
}

async function showMaskMenu() {
  if (maskState.value?.locked) return;
  try {
    await invoke('show_current_mask_menu');
  } catch (error) {
    reportError(error);
  }
}

async function lockMask() {
  if (busy.value || !maskState.value?.trayReady) return;
  busy.value = true;
  try {
    await invoke('lock_current_mask');
  } catch (error) {
    reportError(error);
  } finally {
    busy.value = false;
  }
}

async function deleteMask() {
  if (busy.value) return;
  busy.value = true;
  try {
    await invoke('delete_current_mask');
  } catch (error) {
    busy.value = false;
    reportError(error);
  }
}

function applySettingsTarget(state: MaskViewState | null) {
  settingsMask.value = state;
  if (state) {
    settingsColor.value = state.color;
    settingsOpacity.value = state.opacity;
  }
}

function scheduleAppearanceUpdate() {
  if (!settingsMask.value) return;
  if (appearanceTimer) window.clearTimeout(appearanceTimer);
  appearanceTimer = window.setTimeout(updateAppearance, 80);
}

async function updateAppearance() {
  const target = settingsMask.value;
  if (!target) return;
  try {
    const updated = await invoke<MaskViewState>('update_mask_appearance', {
      id: target.id,
      color: settingsColor.value,
      opacity: settingsOpacity.value,
    });
    applySettingsTarget(updated);
  } catch (error) {
    reportError(error);
  }
}

function selectPreset(color: string) {
  settingsColor.value = color;
  scheduleAppearanceUpdate();
}

async function hideSettings() {
  try {
    await invoke('hide_settings_window');
  } catch (error) {
    reportError(error);
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && isSettingsWindow) hideSettings();
}

onMounted(async () => {
  window.addEventListener('keydown', handleKeydown);
  try {
    if (isSettingsWindow) {
      applySettingsTarget(await invoke<MaskViewState | null>('get_settings_target'));
      unlistenSettings = await listen<MaskViewState>('settings-target-changed', (event) => {
        applySettingsTarget(event.payload);
      });
    } else {
      maskState.value = await invoke<MaskViewState>('get_current_mask');
      unlistenMask = await listen<MaskViewState>('mask-state-changed', (event) => {
        maskState.value = event.payload;
      });
    }
  } catch (error) {
    reportError(error);
  }
  await nextTick();
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
  unlistenMask?.();
  unlistenSettings?.();
  if (appearanceTimer) window.clearTimeout(appearanceTimer);
});
</script>

<style>
:root {
  color: #f8fafc;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  font-synthesis: none;
}

* {
  box-sizing: border-box;
}

html,
body,
#app {
  width: 100%;
  height: 100%;
  margin: 0;
  overflow: hidden;
  background: transparent;
}

button,
input {
  font: inherit;
}

.mask-container {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  border: 1px dashed rgba(255, 255, 255, 0.62);
  border-radius: 10px;
  box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.2);
  user-select: none;
}

.mask-container.is-locked {
  border: 0;
  box-shadow: none;
}

.mask-toolbar {
  position: absolute;
  z-index: 30;
  top: 4px;
  right: 4px;
  display: flex;
  gap: 3px;
}

.icon-button {
  display: grid;
  width: 24px;
  height: 24px;
  padding: 0;
  place-items: center;
  border: 1px solid rgba(255, 255, 255, 0.42);
  border-radius: 5px;
  color: #fff;
  background: rgba(15, 23, 42, 0.68);
  cursor: pointer;
  line-height: 1;
}

.icon-button:hover:not(:disabled),
.icon-button:focus-visible {
  background: rgba(51, 65, 85, 0.94);
}

.icon-button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.delete-button:hover:not(:disabled),
.delete-button:focus-visible {
  background: rgba(185, 28, 28, 0.94);
}

.resize-handle {
  position: absolute;
  z-index: 20;
}

.north { top: 0; left: 14px; right: 14px; height: 7px; cursor: n-resize; }
.south { bottom: 0; left: 14px; right: 14px; height: 7px; cursor: s-resize; }
.west { left: 0; top: 14px; bottom: 14px; width: 7px; cursor: w-resize; }
.east { right: 0; top: 14px; bottom: 14px; width: 7px; cursor: e-resize; }
.north-west { top: 0; left: 0; width: 14px; height: 14px; cursor: nw-resize; }
.north-east { top: 0; right: 0; width: 14px; height: 14px; cursor: ne-resize; }
.south-west { bottom: 0; left: 0; width: 14px; height: 14px; cursor: sw-resize; }
.south-east { right: 0; bottom: 0; width: 14px; height: 14px; cursor: se-resize; }

.mask-error {
  position: absolute;
  right: 4px;
  bottom: 4px;
  left: 4px;
  overflow: hidden;
  padding: 3px 6px;
  border-radius: 4px;
  color: #fee2e2;
  background: rgba(127, 29, 29, 0.92);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.settings-shell {
  min-height: 100%;
  padding: 20px;
  color: #e2e8f0;
  background: linear-gradient(150deg, #0f172a, #111827);
}

.settings-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 18px;
}

.eyebrow {
  color: #94a3b8;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.11em;
  text-transform: uppercase;
}

.settings-header h1,
.empty-settings h1 {
  margin: 3px 0 0;
  font-size: 21px;
}

.settings-close {
  width: 30px;
  height: 30px;
  border: 0;
  border-radius: 7px;
  color: #cbd5e1;
  background: rgba(148, 163, 184, 0.12);
  cursor: pointer;
}

.settings-section {
  display: grid;
  gap: 9px;
  margin-top: 15px;
}

.settings-section label {
  font-size: 13px;
  font-weight: 650;
}

.color-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.color-row input {
  width: 52px;
  height: 34px;
  padding: 2px;
  border: 1px solid #475569;
  border-radius: 6px;
  background: #1e293b;
}

.color-row code {
  color: #cbd5e1;
}

.presets {
  display: flex;
  gap: 8px;
}

.preset {
  width: 26px;
  height: 26px;
  border: 2px solid rgba(255, 255, 255, 0.5);
  border-radius: 50%;
  cursor: pointer;
}

.settings-error {
  margin: 12px 0 0;
  color: #fca5a5;
  font-size: 12px;
}

.settings-help,
.empty-settings p {
  color: #94a3b8;
  font-size: 12px;
  line-height: 1.45;
}

.empty-settings {
  display: grid;
  min-height: 180px;
  place-content: center;
  text-align: center;
}
</style>
