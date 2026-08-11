<template>
  <div class="mask-container" data-tauri-drag-region>
  </div>
</template>

<script setup lang="ts">
// MVP: 仅使用原生 data-tauri-drag-region
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
</style>
