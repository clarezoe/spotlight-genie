<script setup lang="ts">
import type { ResultCategory } from "../types";

defineProps<{
  category: ResultCategory;
}>();

const badgeColors: Record<string, string> = {
  SPOTIFY: "#1DB954",
  CURRENCY: "#3B82F6",
  CLIP: "#8B5CF6",
};
</script>

<template>
  <span
    class="inline-flex items-center rounded-full border px-2 py-0.5 font-heading text-[10px] font-semibold uppercase tracking-wide"
    :style="{
      borderColor: badgeColors[category] || '#4B5563',
      color: badgeColors[category] || '#94A3B8',
    }"
  >
    {{ category }}
  </span>
</template>
