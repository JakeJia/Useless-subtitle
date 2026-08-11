<template>
  <div class="mask-container" data-tauri-drag-region>
    <!-- 拖拽拉伸边框/四角手柄 -->
    <div class="resize-handle top" @mousedown="startResize('TOP')" />
    <div class="resize-handle bottom" @mousedown="startResize('BOTTOM')" />
    <div class="resize-handle left" @mousedown="startResize('LEFT')" />
    <div class="resize-handle right" @mousedown="startResize('RIGHT')" />
    
    <div class="resize-handle top-left" @mousedown="startResize('TOP_LEFT')" />
    <div class="resize-handle top-right" @mousedown="startResize('TOP_RIGHT')" />
    <div class="resize-handle bottom-left" @mousedown="startResize('BOTTOM_LEFT')" />
    <div class="resize-handle bottom-right" @mousedown="startResize('BOTTOM_RIGHT')" />
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

// 使用 any 避免类型检查严格报错
function startResize(direction: string) {
  appWindow.startResizing(direction as any);
}
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
}

/* 核心遮罩层，圆角与纯色，带内置拖拽指令 */
.mask-container {
  width: 100vw;
  height: 100vh;
  background-color: rgba(0, 0, 0, 0.9);
  border-radius: 12px;
  position: relative;
  box-sizing: border-box;
  /* 编辑状态下显示微弱白框，提示用户可操作 */
  border: 1px solid rgba(255, 255, 255, 0.3);
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
</style>
