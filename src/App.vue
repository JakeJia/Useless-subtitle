<template>
  <!-- 当锁定且穿透时，完全去掉外边框提示 -->
  <div class="mask-container" 
       :class="{ 'is-locked': isLocked }"
       :style="{ backgroundColor: hexToRgba(maskColor, maskOpacity) }"
       data-tauri-drag-region
       @contextmenu.prevent="showContextMenu">
    
    <template v-if="!isLocked">
      <!-- 拖拽拉伸边框/四角手柄 -->
      <div class="resize-handle top" @mousedown="startResize('TOP')" />
      <div class="resize-handle bottom" @mousedown="startResize('BOTTOM')" />
      <div class="resize-handle left" @mousedown="startResize('LEFT')" />
      <div class="resize-handle right" @mousedown="startResize('RIGHT')" />
      
      <div class="resize-handle top-left" @mousedown="startResize('TOP_LEFT')" />
      <div class="resize-handle top-right" @mousedown="startResize('TOP_RIGHT')" />
      <div class="resize-handle bottom-left" @mousedown="startResize('BOTTOM_LEFT')" />
      <div class="resize-handle bottom-right" @mousedown="startResize('BOTTOM_RIGHT')" />

      <!-- 右上角悬浮控制按钮 -->
      <div class="controls">
        <button class="icon-btn lock-btn" @click.stop="lockMask" title="锁定并穿透 (Lock)">🔒</button>
        <button class="icon-btn close-btn" @click.stop="closeMask" title="关闭遮罩 (Close)">❌</button>
      </div>
    </template>

    <!-- 自定义右键菜单 -->
    <div v-if="contextMenuVisible" class="context-menu" :style="{ top: contextMenuY + 'px', left: contextMenuX + 'px' }" @click.stop>
      <div class="menu-item">
        <label>颜色:</label>
        <input type="color" v-model="maskColor" @change="saveState" />
      </div>
      <div class="menu-item">
        <label>透明度: {{ maskOpacity }}%</label>
        <input type="range" min="10" max="100" step="10" v-model="maskOpacity" @change="saveState" />
      </div>
      <hr />
      <div class="menu-item danger" @click="closeMask">关闭此遮罩 (Close)</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { Store } from '@tauri-apps/plugin-store';

const appWindow = getCurrentWindow();
const store = new (Store as any)('store.json');

// 状态
const isLocked = ref(false);
const maskColor = ref('#000000');
const maskOpacity = ref(90);

// 右键菜单状态
const contextMenuVisible = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);

// 工具函数：Hex 转 RGBA
function hexToRgba(hex: string, opacityPercent: number) {
  let r = 0, g = 0, b = 0;
  if (hex.length === 4) {
    r = parseInt(hex[1] + hex[1], 16);
    g = parseInt(hex[2] + hex[2], 16);
    b = parseInt(hex[3] + hex[3], 16);
  } else if (hex.length === 7) {
    r = parseInt(hex.substring(1, 3), 16);
    g = parseInt(hex.substring(3, 5), 16);
    b = parseInt(hex.substring(5, 7), 16);
  }
  return `rgba(${r}, ${g}, ${b}, ${opacityPercent / 100})`;
}

// 拖拽拉伸
function startResize(_direction: string) {
  if (!isLocked.value) {
    appWindow.startDragging();
  }
}

// 锁定并开启穿透
async function lockMask() {
  isLocked.value = true;
  contextMenuVisible.value = false;
  // 调用底层 API 穿透
  await appWindow.setIgnoreCursorEvents(true);
}

// 关闭遮罩
async function closeMask() {
  // 从 store 中移除自己
  await store.load();
  let maskList: any = await store.get('mask_list') || [];
  maskList = maskList.filter((m: any) => m.label !== appWindow.label);
  await store.set('mask_list', maskList);
  await store.save();
  
  await appWindow.close();
}

// 显示右键菜单
function showContextMenu(e: MouseEvent) {
  if (isLocked.value) return; // 锁定状态下已穿透，理论上触发不到
  contextMenuVisible.value = true;
  
  // 简单计算一下边界，防止菜单超出窗口
  let x = e.clientX;
  let y = e.clientY;
  if (x > window.innerWidth - 150) x = window.innerWidth - 150;
  if (y > window.innerHeight - 100) y = window.innerHeight - 100;
  
  contextMenuX.value = x;
  contextMenuY.value = y;
}

// 点击其他地方隐藏右键菜单
function hideContextMenu() {
  contextMenuVisible.value = false;
}

// 保存当前遮罩状态
async function saveState() {
  await store.load();
  let maskList: any = await store.get('mask_list') || [];
  
  // 记录坐标与大小
  const pos = await appWindow.outerPosition();
  const size = await appWindow.outerSize();
  
  const currentMask = {
    label: appWindow.label,
    color: maskColor.value,
    opacity: maskOpacity.value,
    x: pos.x,
    y: pos.y,
    width: size.width,
    height: size.height
  };
  
  const existingIdx = maskList.findIndex((m: any) => m.label === appWindow.label);
  if (existingIdx >= 0) {
    maskList[existingIdx] = currentMask;
  } else {
    maskList.push(currentMask);
  }
  
  await store.set('mask_list', maskList);
  await store.save();
}

let unlistenUnlock: () => void;
let unlistenMoved: () => void;
let unlistenResized: () => void;

onMounted(async () => {
  window.addEventListener('click', hideContextMenu);
  
  // 恢复状态 (如果是重新启动的)
  await store.load();
  const maskList: any = await store.get('mask_list') || [];
  const current = maskList.find((m: any) => m.label === appWindow.label);
  if (current) {
    maskColor.value = current.color || '#000000';
    maskOpacity.value = current.opacity || 90;
    // Window position and size will be loaded correctly via JS or Rust later, we update UI here
  }
  
  // 监听来自系统托盘的“全局解锁”事件
  unlistenUnlock = await listen('unlock_all', async () => {
    isLocked.value = false;
    // 穿透状态恢复是在 Rust 里做的，这里只需更新 UI
  });

  // 监听移动与拉伸结束事件以持久化保存坐标
  unlistenMoved = await appWindow.onMoved(() => saveState());
  unlistenResized = await appWindow.onResized(() => saveState());
});

onUnmounted(() => {
  window.removeEventListener('click', hideContextMenu);
  if (unlistenUnlock) unlistenUnlock();
  if (unlistenMoved) unlistenMoved();
  if (unlistenResized) unlistenResized();
});
</script>

<style>
/* 全局充斥，透明背景 */
html, body, #app {
  margin: 0;
  padding: 0;
  width: 100vw;
  height: 100vh;
  background: transparent;
  overflow: hidden;
  user-select: none;
}

/* 核心遮罩层 */
.mask-container {
  width: 100vw;
  height: 100vh;
  border-radius: 12px;
  position: relative;
  box-sizing: border-box;
  /* 编辑状态下显示微弱白框，提示用户可操作 */
  border: 1px dashed rgba(255, 255, 255, 0.4);
  transition: background-color 0.2s;
}

.mask-container.is-locked {
  border: none;
}

/* 拉伸边缘热区 */
.resize-handle {
  position: absolute;
}
.top { top: 0; left: 15px; right: 15px; height: 8px; cursor: n-resize; }
.bottom { bottom: 0; left: 15px; right: 15px; height: 8px; cursor: s-resize; }
.left { left: 0; top: 15px; bottom: 15px; width: 8px; cursor: w-resize; }
.right { right: 0; top: 15px; bottom: 15px; width: 8px; cursor: e-resize; }

.top-left { top: 0; left: 0; width: 15px; height: 15px; cursor: nw-resize; }
.top-right { top: 0; right: 0; width: 15px; height: 15px; cursor: ne-resize; }
.bottom-left { bottom: 0; left: 0; width: 15px; height: 15px; cursor: sw-resize; }
.bottom-right { bottom: 0; right: 0; width: 15px; height: 15px; cursor: se-resize; }

/* 悬浮控制按钮 */
.controls {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 6px;
  opacity: 0;
  transition: opacity 0.2s;
  z-index: 10;
}
.mask-container:hover .controls {
  opacity: 1;
}
.icon-btn {
  background: rgba(255, 255, 255, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: white;
  border-radius: 6px;
  width: 28px;
  height: 28px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
}
.icon-btn:hover {
  background: rgba(255, 255, 255, 0.4);
}
.close-btn:hover {
  background: rgba(255, 50, 50, 0.8);
}

/* 右键菜单 */
.context-menu {
  position: absolute;
  background: rgba(30, 30, 30, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 8px 0;
  min-width: 160px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.5);
  color: white;
  font-family: sans-serif;
  font-size: 13px;
  z-index: 20;
}
.menu-item {
  padding: 8px 16px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.menu-item.danger {
  color: #ff5555;
  cursor: pointer;
}
.menu-item.danger:hover {
  background: rgba(255, 255, 255, 0.1);
}
hr {
  border: none;
  border-top: 1px solid rgba(255,255,255,0.1);
  margin: 4px 0;
}
</style>
