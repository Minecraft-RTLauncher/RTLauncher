<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { ref } from 'vue';

const username = ref('风吹裤裆蛋蛋凉');

// 登录
async function login() {
  const login = await invoke('get_code');
  console.log('登录: ', login);
}

// 下载
async function downloadMinecraft() {
  const download = await invoke('dwl_version_manifest', {
    url: 'https://piston-meta.mojang.com/v1/packages/c440b9ef34fec9d69388de8650cd55b465116587/1.21.4.json',
  });
  console.log('下载文件: ', download);
}

// 启动游戏
async function startGame(username: string) {
  try {
    const result = await invoke('stg', {
      startupParameter: '-Xmx1024m -Xms1024m',
      versionId: '1.21.4',
      javaVersion: '21',
      assetIndexId: '19',
      username: username,
    });

    console.log('游戏启动成功', result);
  } catch (error) {
    console.error('游戏启动失败:', error);
  }
}

// 获取java_home路径
async function getJavaPath() {
  const javaPath = await invoke('get_java_path');
  console.log('java_home路径: ', javaPath);
}

// 导出启动脚本
async function exportScript() {
  const script = await invoke('export_bat', {
    startupParameter: '-Xmx1024m -Xms1024m',
    versionId: '1.21.4',
    javaVersion: '21',
    outputPath: 'D:\\Desktop\\start_game.bat',
  });
  console.log('启动脚本: ', script);
}
</script>

<template>
  <button @click="login">登录</button>
  <button @click="downloadMinecraft">下载我的世界</button>
  <button @click="startGame(username)">启动游戏</button>

  <button @click="getJavaPath">获取java_home路径</button>
  <button @click="exportScript">导出启动脚本</button>
  <br />
  <div>
    <label for="username">用户名</label>
    <input type="text" id="username" v-model="username" />
  </div>
</template>


<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}
</style>
