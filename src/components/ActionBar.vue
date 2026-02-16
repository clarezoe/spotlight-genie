<script setup lang="ts">
import type { SearchResult } from "../types";

defineProps<{
  selectedResult: SearchResult | null;
}>();

const isMac = navigator.platform.toUpperCase().includes("MAC");
const mod = isMac ? "⌘" : "Ctrl";
</script>

<template>
  <div
    v-if="selectedResult"
    class="flex w-full items-center justify-between px-5 py-2 font-body text-[11px] text-genie-text-muted"
    style="border-top: 1px solid rgba(255, 255, 255, 0.06)"
  >
    <span>Open (Enter)</span>
    <span>Reveal ({{ mod }}+Enter) &nbsp; Copy ({{ mod }}+C)</span>
  </div>
</template>
