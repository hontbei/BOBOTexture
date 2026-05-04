<script setup lang="ts">
import PanelLeft from './PanelLeft.vue'
import PanelCenter from './PanelCenter.vue'
import PanelRight from './PanelRight.vue'
import StatusBar from './StatusBar.vue'
import Toolbar from './Toolbar.vue'
import ExportBar from './ExportBar.vue'
import WindowChrome from './WindowChrome.vue'

defineEmits<{
  save: []
  'save-as': []
  new: []
  open: []
  publish: []
  'request-close': []
  'settings-open': []
}>()
</script>

<template>
  <div class="shell-root">
    <div class="shell-chrome">
      <WindowChrome @request-close="$emit('request-close')" />
    </div>
    <div class="shell-toolbar">
      <Toolbar
        @save="$emit('save')"
        @save-as="$emit('save-as')"
        @new="$emit('new')"
        @open="$emit('open')"
        @settings-open="$emit('settings-open')"
      />
    </div>
    <div class="shell-export">
      <ExportBar @publish="$emit('publish')" />
    </div>
    <div class="shell-body" @contextmenu.prevent>
      <PanelLeft class="shell-left" />
      <div class="panel-divider" />
      <PanelCenter class="shell-center" />
      <div class="panel-divider" />
      <PanelRight class="shell-right" />
    </div>
    <div class="shell-statusbar">
      <StatusBar />
    </div>
  </div>
</template>

<style scoped>
.shell-root {
  display: grid;
  grid-template-rows: 32px var(--toolbar-height) auto 1fr var(--statusbar-height);
  grid-template-columns: 1fr;
  width: 100%;
  height: 100%;
  background: var(--bg-app);
}

.shell-chrome {
  border-bottom: 1px solid var(--divider);
}

.shell-toolbar {
  border-bottom: 1px solid var(--divider);
}

.shell-body {
  display: flex;
  overflow: hidden;
  min-height: 0;
}

.shell-left {
  width: 300px; min-width: 240px; flex-shrink: 0;
  display: flex; flex-direction: column; overflow: hidden;
  height: 100%;
}

.shell-center {
  flex: 1; min-width: 200px;
  display: flex; flex-direction: column; overflow: hidden;
  background: var(--bg-app); height: 100%;
}

.shell-right {
  width: 320px; min-width: 280px; flex-shrink: 0;
  display: flex; flex-direction: column; overflow: hidden;
  height: 100%;
}

.shell-center {
  flex: 1;
  min-width: 200px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-app);
}

.shell-right {
  width: 320px;
  min-width: 280px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.shell-statusbar {
  border-top: 1px solid var(--divider);
}

.panel-divider {
  width: 1px;
  background: var(--divider);
  flex-shrink: 0;
}
</style>
