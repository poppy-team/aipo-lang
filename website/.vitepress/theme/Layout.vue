<script setup lang="ts">
import DefaultTheme from 'vitepress/theme'
import { ref, onMounted } from 'vue'
const focus = ref(false)
const spacious = ref(false)
function sync() {
  if (typeof document === 'undefined') return
  document.documentElement.classList.toggle('aipo-focus', focus.value)
  document.documentElement.classList.toggle('aipo-spacious', spacious.value)
}
onMounted(() => {
  try {
    focus.value = localStorage.getItem('aipo-reading-focus') === '1'
    spacious.value = localStorage.getItem('aipo-reading-spacious') === '1'
  } catch {}
  sync()
})
function toggleFocus() {
  focus.value = !focus.value
  sync()
  try { localStorage.setItem('aipo-reading-focus', focus.value ? '1' : '0') } catch {}
}
function toggleSpacious() {
  spacious.value = !spacious.value
  sync()
  try { localStorage.setItem('aipo-reading-spacious', spacious.value ? '1' : '0') } catch {}
}
</script>
<template>
  <DefaultTheme.Layout>
    <template #nav-bar-content-after>
      <div class="aipo-reading-settings" role="group" aria-label="Preferências de leitura">
        <button type="button" :aria-pressed="focus" @click="toggleFocus">{{ focus ? 'Sair do foco' : 'Modo foco' }}</button>
        <button type="button" :aria-pressed="spacious" @click="toggleSpacious">{{ spacious ? 'Espaço padrão' : 'Mais espaço' }}</button>
      </div>
    </template>
  </DefaultTheme.Layout>
</template>
